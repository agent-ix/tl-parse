// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Peter Krenesky

//! Structured bounded-fuzz campaign producer (FR-005-AC-2, FR-006-AC-2).
//!
//! This Rust-owned boundary verifies checked seeds, supervises cargo-fuzz, and
//! emits one typed result that shared assurance can retain without scraping a
//! transcript or executing the producer itself.

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use command_group::{CommandGroup, GroupChild};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const PROTOCOL: &str = "tl-parse.fuzz-campaign/v1";
const RUNS: u32 = 64;
const DEADLINE: Duration = Duration::from_secs(300);
const MAX_SEEDS: usize = 256;
const MAX_ARTIFACTS: usize = 16;
const MAX_FILE_BYTES: u64 = 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const TOOL_PROBE_DEADLINE: Duration = Duration::from_secs(10);
const MAX_TOOL_OUTPUT_BYTES: usize = 8 * 1024;
const ATTACHMENT_LIMITATION: &str =
    "lossless crash-artifact attachment is deferred to agent-ix/quoin#363";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Target {
    Parser,
    CleanAsciiV2,
}

impl Target {
    fn parse(value: &OsStr) -> Option<Self> {
        match value.to_str() {
            Some("parser") => Some(Self::Parser),
            Some("clean_ascii_v2") => Some(Self::CleanAsciiV2),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Parser => "parser",
            Self::CleanAsciiV2 => "clean_ascii_v2",
        }
    }

    const fn symbol(self) -> &'static str {
        match self {
            Self::Parser => "fuzz:parser",
            Self::CleanAsciiV2 => "fuzz:clean_ascii_v2",
        }
    }

    fn from_proof_id(value: &OsStr) -> Option<Self> {
        match value.to_str() {
            Some("PROOF-parser-fuzz-campaign") => Some(Self::Parser),
            Some("PROOF-clean-ascii-v2-fuzz-campaign") => Some(Self::CleanAsciiV2),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum DomainOutcome {
    Pass,
    Fail,
    Unavailable,
    Suspect,
}

impl DomainOutcome {
    const fn normalized(self) -> NormalizedOutcome {
        match self {
            Self::Pass => NormalizedOutcome::Pass,
            Self::Fail | Self::Suspect => NormalizedOutcome::Fail,
            Self::Unavailable => NormalizedOutcome::Skip,
        }
    }

    const fn exit_code(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Fail => 1,
            Self::Unavailable | Self::Suspect => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum NormalizedOutcome {
    Pass,
    Fail,
    Skip,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ElapsedClass {
    NotRun,
    WithinDeadline,
    DeadlineExceeded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessState {
    NotStarted,
    Exited(i32),
    TimedOut,
    SupervisionFailed,
}

fn classify(process: ProcessState, artifacts_present: bool) -> (DomainOutcome, ElapsedClass) {
    match process {
        ProcessState::NotStarted => (DomainOutcome::Unavailable, ElapsedClass::NotRun),
        ProcessState::TimedOut => (DomainOutcome::Unavailable, ElapsedClass::DeadlineExceeded),
        ProcessState::SupervisionFailed => {
            (DomainOutcome::Unavailable, ElapsedClass::WithinDeadline)
        }
        ProcessState::Exited(0) if artifacts_present => {
            (DomainOutcome::Suspect, ElapsedClass::WithinDeadline)
        }
        ProcessState::Exited(0) => (DomainOutcome::Pass, ElapsedClass::WithinDeadline),
        ProcessState::Exited(_) => (DomainOutcome::Fail, ElapsedClass::WithinDeadline),
    }
}

#[derive(Debug)]
enum ErrorCode {
    AmbientSanitizerOverride,
    LeakSanitizerUnavailable,
    ManifestUnreadable,
    ManifestMalformed,
    SeedPopulationExceeded,
    SeedInvalid,
    SeedDigestMismatch,
    ToolUnavailable,
    ScratchUnavailable,
    FuzzerUnavailable,
    SupervisionFailed,
}

impl ErrorCode {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::AmbientSanitizerOverride => "ambient_sanitizer_override",
            Self::LeakSanitizerUnavailable => "leak_sanitizer_unavailable",
            Self::ManifestUnreadable => "manifest_unreadable",
            Self::ManifestMalformed => "manifest_malformed",
            Self::SeedPopulationExceeded => "seed_population_exceeded",
            Self::SeedInvalid => "seed_invalid",
            Self::SeedDigestMismatch => "seed_digest_mismatch",
            Self::ToolUnavailable => "tool_unavailable",
            Self::ScratchUnavailable => "scratch_unavailable",
            Self::FuzzerUnavailable => "fuzzer_unavailable",
            Self::SupervisionFailed => "supervision_failed",
        }
    }
}

#[derive(Debug)]
struct CampaignError {
    code: ErrorCode,
    message: String,
}

impl CampaignError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    fn limitation(&self) -> String {
        format!("{}: {}", self.code.as_str(), self.message)
    }

    const fn process_state(&self) -> ProcessState {
        match self.code {
            ErrorCode::SupervisionFailed => ProcessState::SupervisionFailed,
            ErrorCode::AmbientSanitizerOverride
            | ErrorCode::LeakSanitizerUnavailable
            | ErrorCode::ManifestUnreadable
            | ErrorCode::ManifestMalformed
            | ErrorCode::SeedPopulationExceeded
            | ErrorCode::SeedInvalid
            | ErrorCode::SeedDigestMismatch
            | ErrorCode::ToolUnavailable
            | ErrorCode::ScratchUnavailable
            | ErrorCode::FuzzerUnavailable => ProcessState::NotStarted,
        }
    }
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct CampaignDocument {
    protocol: &'static str,
    entries: [NormalizedEntry; 1],
    campaign: Campaign,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NormalizedEntry {
    symbol: &'static str,
    outcome: NormalizedOutcome,
    trace_ids: [&'static str; 5],
    domain_outcome: DomainOutcome,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Campaign {
    target: &'static str,
    requested_runs: u32,
    deadline_seconds: u64,
    seed_manifest: SeedManifestReport,
    tools: ToolReport,
    sanitizer: SanitizerReport,
    process: ProcessReport,
    elapsed_time_class: ElapsedClass,
    domain_outcome: DomainOutcome,
    limitations: Vec<String>,
    artifacts: Vec<ArtifactReport>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeedManifestReport {
    path: String,
    sha256: Option<String>,
    count: Option<u32>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ToolReport {
    nightly_rust: Option<String>,
    cargo_fuzz: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SanitizerReport {
    leak_sanitizer: SanitizerState,
    asan_options: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SanitizerState {
    NotChecked,
    Enabled,
    Unavailable,
    RefusedAmbientOverride,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProcessReport {
    state: ProcessStateName,
    exit_code: Option<i32>,
}

impl From<ProcessState> for ProcessReport {
    fn from(value: ProcessState) -> Self {
        match value {
            ProcessState::NotStarted => Self {
                state: ProcessStateName::NotStarted,
                exit_code: None,
            },
            ProcessState::Exited(code) => Self {
                state: ProcessStateName::Exited,
                exit_code: Some(code),
            },
            ProcessState::TimedOut => Self {
                state: ProcessStateName::TimedOut,
                exit_code: None,
            },
            ProcessState::SupervisionFailed => Self {
                state: ProcessStateName::SupervisionFailed,
                exit_code: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ProcessStateName {
    NotStarted,
    Exited,
    TimedOut,
    SupervisionFailed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadCampaignDocument {
    protocol: String,
    entries: Vec<ReadNormalizedEntry>,
    campaign: ReadCampaign,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadNormalizedEntry {
    symbol: String,
    outcome: NormalizedOutcome,
    trace_ids: Vec<String>,
    domain_outcome: DomainOutcome,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadCampaign {
    target: String,
    requested_runs: u32,
    deadline_seconds: u64,
    seed_manifest: ReadSeedManifest,
    tools: ReadTools,
    sanitizer: ReadSanitizer,
    process: ReadProcess,
    elapsed_time_class: ElapsedClass,
    domain_outcome: DomainOutcome,
    limitations: Vec<String>,
    artifacts: Vec<ReadArtifact>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadSeedManifest {
    path: String,
    sha256: Option<String>,
    count: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadTools {
    nightly_rust: Option<String>,
    cargo_fuzz: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadSanitizer {
    leak_sanitizer: SanitizerState,
    asan_options: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadProcess {
    state: ProcessStateName,
    exit_code: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadArtifact {
    name: String,
    media_type: String,
    byte_length: u64,
    sha256: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArtifactReport {
    name: String,
    media_type: &'static str,
    byte_length: u64,
    sha256: String,
}

#[derive(Debug)]
struct Seed {
    name: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct SeedManifest {
    seeds: Vec<Seed>,
}

struct Execution {
    process: ProcessState,
    artifacts: Vec<ArtifactReport>,
    artifact_population_suspect: bool,
    limitations: Vec<String>,
    scratch: Option<TempDir>,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn exact_object_fields(value: &Value, fields: &[&str], name: &str) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{name} is not an object"))?;
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = fields.iter().copied().collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(format!("{name} fields are not the closed protocol fields"));
    }
    Ok(())
}

fn validate_result_shape(value: &Value) -> Result<(), String> {
    exact_object_fields(value, &["protocol", "entries", "campaign"], "document")?;
    let entries = value["entries"]
        .as_array()
        .ok_or_else(|| "entries is not an array".to_owned())?;
    if entries.len() != 1 {
        return Err("entries must contain exactly one result".to_owned());
    }
    exact_object_fields(
        &entries[0],
        &["symbol", "outcome", "traceIds", "domainOutcome"],
        "entry",
    )?;
    let campaign = &value["campaign"];
    exact_object_fields(
        campaign,
        &[
            "target",
            "requestedRuns",
            "deadlineSeconds",
            "seedManifest",
            "tools",
            "sanitizer",
            "process",
            "elapsedTimeClass",
            "domainOutcome",
            "limitations",
            "artifacts",
        ],
        "campaign",
    )?;
    exact_object_fields(
        &campaign["seedManifest"],
        &["path", "sha256", "count"],
        "seedManifest",
    )?;
    exact_object_fields(&campaign["tools"], &["nightlyRust", "cargoFuzz"], "tools")?;
    exact_object_fields(
        &campaign["sanitizer"],
        &["leakSanitizer", "asanOptions"],
        "sanitizer",
    )?;
    exact_object_fields(&campaign["process"], &["state", "exitCode"], "process")?;
    let artifacts = campaign["artifacts"]
        .as_array()
        .ok_or_else(|| "artifacts is not an array".to_owned())?;
    for artifact in artifacts {
        exact_object_fields(
            artifact,
            &["name", "mediaType", "byteLength", "sha256"],
            "artifact",
        )?;
    }
    Ok(())
}

fn bounded_result_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("could not inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_FILE_BYTES {
        return Err(format!(
            "{} is not a bounded regular, non-symlink result",
            path.display()
        ));
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(format!(
            "{} grew beyond its result byte limit",
            path.display()
        ));
    }
    Ok(bytes)
}

fn lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_campaign_result(
    root: &Path,
    target: Target,
    raw: &[u8],
) -> Result<&'static str, String> {
    let value: Value = serde_json::from_slice(raw)
        .map_err(|error| format!("campaign result is not readable JSON: {error}"))?;
    validate_result_shape(&value)?;
    let document: ReadCampaignDocument = serde_json::from_value(value)
        .map_err(|error| format!("campaign result is not the typed protocol: {error}"))?;
    if document.protocol != PROTOCOL {
        return Err(format!("campaign result is not {PROTOCOL}"));
    }
    let [entry] = document.entries.as_slice() else {
        return Err("campaign result must contain exactly one entry".to_owned());
    };
    let campaign = &document.campaign;
    if campaign.target != target.as_str() || entry.symbol != target.symbol() {
        return Err("campaign target, proof, and symbol identities disagree".to_owned());
    }
    if entry.trace_ids
        != [
            "TC-047",
            "FR-005-AC-2",
            "FR-006-AC-2",
            "NFR-003-AC-1",
            "SUITE-010",
        ]
    {
        return Err("campaign result does not bind the declared trace identities".to_owned());
    }
    if campaign.requested_runs != RUNS || campaign.deadline_seconds != DEADLINE.as_secs() {
        return Err("campaign result does not bind the fixed run/deadline bounds".to_owned());
    }

    let expected_manifest = format!("fuzz/corpus/{}/SHA256SUMS", target.as_str());
    if campaign.seed_manifest.path != expected_manifest {
        return Err("campaign result does not bind the target seed manifest".to_owned());
    }
    match (
        campaign.seed_manifest.sha256.as_deref(),
        campaign.seed_manifest.count,
    ) {
        (None, None) => {}
        (Some(digest), Some(count)) => {
            let manifest_bytes = bounded_result_bytes(&root.join(&expected_manifest))?;
            let declared_count = manifest_bytes
                .split(|byte| *byte == b'\n')
                .filter(|line| line.iter().any(|byte| !byte.is_ascii_whitespace()))
                .count();
            if !lower_sha256(digest)
                || digest != sha256_hex(&manifest_bytes)
                || count == 0
                || count > u32::try_from(MAX_SEEDS).unwrap_or(u32::MAX)
                || usize::try_from(count).ok() != Some(declared_count)
            {
                return Err(
                    "campaign seed-manifest identity does not match the repository".to_owned(),
                );
            }
        }
        _ => return Err("campaign result has a partial seed-manifest identity".to_owned()),
    }

    if campaign
        .tools
        .nightly_rust
        .as_deref()
        .is_some_and(|value| !tool_identity_matches("nightly rustc", value))
        || campaign
            .tools
            .cargo_fuzz
            .as_deref()
            .is_some_and(|value| !tool_identity_matches("cargo-fuzz", value))
    {
        return Err("campaign result carries an invalid tool identity".to_owned());
    }
    let sanitizer_enabled = campaign.sanitizer.leak_sanitizer == SanitizerState::Enabled;
    if sanitizer_enabled != (campaign.sanitizer.asan_options.as_deref() == Some("detect_leaks=1")) {
        return Err("campaign sanitizer state and ASAN_OPTIONS disagree".to_owned());
    }
    match campaign.process.state {
        ProcessStateName::Exited if campaign.process.exit_code.is_none() => {
            return Err("exited campaign result has no exit code".to_owned());
        }
        ProcessStateName::NotStarted
        | ProcessStateName::TimedOut
        | ProcessStateName::SupervisionFailed
            if campaign.process.exit_code.is_some() =>
        {
            return Err("non-exited campaign result carries an exit code".to_owned());
        }
        _ => {}
    }
    if campaign.limitations.iter().any(String::is_empty) {
        return Err("campaign result carries an empty limitation".to_owned());
    }
    if campaign.artifacts.len() > MAX_ARTIFACTS {
        return Err("campaign artifact population exceeds its bound".to_owned());
    }
    for artifact in &campaign.artifacts {
        if !normalized_file_name(&artifact.name)
            || artifact.media_type != "application/octet-stream"
            || artifact.byte_length > MAX_FILE_BYTES
            || !lower_sha256(&artifact.sha256)
        {
            return Err("campaign result carries an invalid artifact identity".to_owned());
        }
    }

    let domain = campaign.domain_outcome;
    if entry.domain_outcome != domain || entry.outcome != domain.normalized() {
        return Err("campaign entry and domain outcomes disagree".to_owned());
    }
    let launched = matches!(
        domain,
        DomainOutcome::Pass | DomainOutcome::Fail | DomainOutcome::Suspect
    );
    if launched
        && (!matches!(campaign.process.state, ProcessStateName::Exited)
            || campaign.elapsed_time_class != ElapsedClass::WithinDeadline
            || !sanitizer_enabled
            || campaign.tools.nightly_rust.is_none()
            || campaign.tools.cargo_fuzz.is_none()
            || campaign.seed_manifest.sha256.is_none())
    {
        return Err("launched campaign outcome lacks execution prerequisites".to_owned());
    }
    match domain {
        DomainOutcome::Pass
            if campaign.process.exit_code != Some(0) || !campaign.artifacts.is_empty() =>
        {
            Err("passing campaign is contradicted by process/artifact state".to_owned())
        }
        DomainOutcome::Pass => Ok("passed"),
        DomainOutcome::Fail if campaign.process.exit_code == Some(0) => {
            Err("failed campaign is contradicted by a zero exit code".to_owned())
        }
        DomainOutcome::Fail => Ok("failed"),
        DomainOutcome::Suspect
            if campaign.process.exit_code != Some(0)
                || campaign.artifacts.is_empty()
                    && !campaign
                        .limitations
                        .iter()
                        .any(|item| item.starts_with("artifact_population_suspect:")) =>
        {
            Err("suspect campaign has no suspect artifact population".to_owned())
        }
        DomainOutcome::Suspect => Ok("failed"),
        DomainOutcome::Unavailable => {
            let elapsed_matches = matches!(
                (campaign.process.state, campaign.elapsed_time_class),
                (ProcessStateName::NotStarted, ElapsedClass::NotRun)
                    | (ProcessStateName::TimedOut, ElapsedClass::DeadlineExceeded)
                    | (
                        ProcessStateName::SupervisionFailed,
                        ElapsedClass::WithinDeadline
                    )
            );
            if !elapsed_matches {
                return Err("unavailable campaign process/elapsed states disagree".to_owned());
            }
            Ok("unavailable")
        }
    }
}

fn normalized_file_name(value: &str) -> bool {
    let path = Path::new(value);
    let mut components = path.components();
    matches!(components.next(), Some(Component::Normal(name)) if name == OsStr::new(value))
        && components.next().is_none()
}

fn bounded_seed_bytes(path: &Path) -> Result<Vec<u8>, CampaignError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        CampaignError::new(
            ErrorCode::SeedInvalid,
            format!("{}: {error}", path.display()),
        )
    })?;
    if !metadata.file_type().is_file() {
        return Err(CampaignError::new(
            ErrorCode::SeedInvalid,
            format!("{} is not a regular, non-symlink file", path.display()),
        ));
    }
    if metadata.len() > MAX_FILE_BYTES {
        return Err(CampaignError::new(
            ErrorCode::SeedInvalid,
            format!("{} exceeds the {MAX_FILE_BYTES}-byte limit", path.display()),
        ));
    }
    let capacity = usize::try_from(metadata.len()).map_err(|_| {
        CampaignError::new(
            ErrorCode::SeedInvalid,
            format!("{} length cannot be represented", path.display()),
        )
    })?;
    let mut bytes = Vec::with_capacity(capacity);
    File::open(path)
        .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|error| {
            CampaignError::new(
                ErrorCode::SeedInvalid,
                format!("{}: {error}", path.display()),
            )
        })?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
        return Err(CampaignError::new(
            ErrorCode::SeedInvalid,
            format!("{} grew beyond its byte limit while read", path.display()),
        ));
    }
    Ok(bytes)
}

fn load_seed_manifest(
    root: &Path,
    target: Target,
    report: &mut SeedManifestReport,
) -> Result<SeedManifest, CampaignError> {
    let directory = root.join("fuzz/corpus").join(target.as_str());
    let path = directory.join("SHA256SUMS");
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        CampaignError::new(
            ErrorCode::ManifestUnreadable,
            format!("{}: {error}", path.display()),
        )
    })?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_MANIFEST_BYTES {
        return Err(CampaignError::new(
            ErrorCode::ManifestMalformed,
            format!("{} is not a bounded regular file", path.display()),
        ));
    }
    let mut raw = Vec::new();
    File::open(&path)
        .and_then(|file| file.take(MAX_MANIFEST_BYTES + 1).read_to_end(&mut raw))
        .map_err(|error| {
            CampaignError::new(
                ErrorCode::ManifestUnreadable,
                format!("{}: {error}", path.display()),
            )
        })?;
    if u64::try_from(raw.len()).unwrap_or(u64::MAX) > MAX_MANIFEST_BYTES {
        return Err(CampaignError::new(
            ErrorCode::ManifestMalformed,
            format!("{} grew beyond its byte limit while read", path.display()),
        ));
    }
    report.sha256 = Some(sha256_hex(&raw));
    let text = std::str::from_utf8(&raw).map_err(|error| {
        CampaignError::new(
            ErrorCode::ManifestMalformed,
            format!("{} is not UTF-8: {error}", path.display()),
        )
    })?;

    let mut declarations = Vec::new();
    let mut names = BTreeSet::new();
    for (index, line) in text.lines().enumerate() {
        let Some((digest, name)) = line.split_once("  ") else {
            return Err(CampaignError::new(
                ErrorCode::ManifestMalformed,
                format!(
                    "{} line {} is not SHA256SUMS format",
                    path.display(),
                    index + 1
                ),
            ));
        };
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || !normalized_file_name(name)
        {
            return Err(CampaignError::new(
                ErrorCode::ManifestMalformed,
                format!(
                    "{} line {} has an invalid digest or name",
                    path.display(),
                    index + 1
                ),
            ));
        }
        if !names.insert(name.to_owned()) {
            return Err(CampaignError::new(
                ErrorCode::ManifestMalformed,
                format!("{} declares duplicate seed {name:?}", path.display()),
            ));
        }
        declarations.push((digest, name));
        if declarations.len() > MAX_SEEDS {
            return Err(CampaignError::new(
                ErrorCode::SeedPopulationExceeded,
                format!("{} declares more than {MAX_SEEDS} seeds", path.display()),
            ));
        }
    }
    if declarations.is_empty() {
        return Err(CampaignError::new(
            ErrorCode::ManifestMalformed,
            format!("{} declares no seeds", path.display()),
        ));
    }
    report.count = Some(u32::try_from(declarations.len()).map_err(|_| {
        CampaignError::new(
            ErrorCode::SeedPopulationExceeded,
            "seed count cannot be represented in the campaign protocol",
        )
    })?);

    let mut seeds = Vec::with_capacity(declarations.len());
    for (expected, name) in declarations {
        let seed_path = directory.join(name);
        let bytes = bounded_seed_bytes(&seed_path)?;
        let observed = sha256_hex(&bytes);
        if observed != expected {
            return Err(CampaignError::new(
                ErrorCode::SeedDigestMismatch,
                format!(
                    "{} declares {expected} but contains {observed}",
                    seed_path.display()
                ),
            ));
        }
        seeds.push(Seed {
            name: name.to_owned(),
            bytes,
        });
    }
    Ok(SeedManifest { seeds })
}

fn copy_seeds(manifest: &SeedManifest, directory: &Path) -> Result<(), CampaignError> {
    for seed in &manifest.seeds {
        fs::write(directory.join(&seed.name), &seed.bytes).map_err(|error| {
            CampaignError::new(
                ErrorCode::ScratchUnavailable,
                format!("could not stage seed {:?}: {error}", seed.name),
            )
        })?;
    }
    Ok(())
}

fn tool_identity_matches(identity: &str, observed: &str) -> bool {
    match identity {
        "nightly rustc" => observed.starts_with("rustc ") && observed.contains("-nightly"),
        "cargo-fuzz" => observed.starts_with("cargo-fuzz "),
        _ => false,
    }
}

struct BoundedOutput {
    bytes: Vec<u8>,
    exceeded: bool,
    error: Option<io::Error>,
}

fn read_bounded_output(mut input: impl Read) -> BoundedOutput {
    let mut bytes = Vec::new();
    let mut exceeded = false;
    let mut buffer = [0_u8; 1024];
    loop {
        match input.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                let remaining = MAX_TOOL_OUTPUT_BYTES.saturating_sub(bytes.len());
                let retained = remaining.min(read);
                bytes.extend_from_slice(&buffer[..retained]);
                exceeded |= retained != read;
            }
            Err(error) => {
                return BoundedOutput {
                    bytes,
                    exceeded,
                    error: Some(error),
                };
            }
        }
    }
    BoundedOutput {
        bytes,
        exceeded,
        error: None,
    }
}

fn observe_version(arguments: &[&str], identity: &str) -> Result<String, CampaignError> {
    let mut child = Command::new("rustup")
        .args(arguments)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .group_spawn()
        .map_err(|error| {
            CampaignError::new(
                ErrorCode::ToolUnavailable,
                format!("could not observe {identity}: {error}"),
            )
        })?;
    let (Some(stdout), Some(stderr)) = (child.inner().stdout.take(), child.inner().stderr.take())
    else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("could not capture {identity} version output"),
        ));
    };
    let stdout_reader = thread::spawn(move || read_bounded_output(stdout));
    let stderr_reader = thread::spawn(move || read_bounded_output(stderr));

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if started.elapsed() < TOOL_PROBE_DEADLINE => {
                thread::sleep(Duration::from_millis(10));
            }
            Ok(None) => {
                let killed = child.kill();
                let reaped = child.wait();
                break match (killed, reaped) {
                    (Ok(()), Ok(_)) => Err(CampaignError::new(
                        ErrorCode::ToolUnavailable,
                        format!("{identity} version probe exceeded 10 seconds"),
                    )),
                    (kill, reap) => Err(CampaignError::new(
                        ErrorCode::ToolUnavailable,
                        format!(
                            "{identity} version probe could not be terminated: kill={kill:?}, wait={reap:?}"
                        ),
                    )),
                };
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(CampaignError::new(
                    ErrorCode::ToolUnavailable,
                    format!("could not supervise {identity} version probe: {error}"),
                ));
            }
        }
    };
    let stdout = stdout_reader.join().map_err(|_| {
        CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("{identity} stdout reader panicked"),
        )
    })?;
    let stderr = stderr_reader.join().map_err(|_| {
        CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("{identity} stderr reader panicked"),
        )
    })?;
    let status = status?;
    if stdout.exceeded || stderr.exceeded {
        return Err(CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("{identity} version output exceeded {MAX_TOOL_OUTPUT_BYTES} bytes"),
        ));
    }
    if let Some(error) = stdout.error.or(stderr.error) {
        return Err(CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("could not read {identity} version output: {error}"),
        ));
    }
    if !status.success() {
        return Err(CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!(
                "{identity} version probe exited {:?}: {}",
                status.code(),
                String::from_utf8_lossy(&stderr.bytes).trim()
            ),
        ));
    }
    let observed = String::from_utf8(stdout.bytes).map_err(|error| {
        CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("{identity} version is not UTF-8: {error}"),
        )
    })?;
    let observed = observed.trim();
    if observed.is_empty() {
        return Err(CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("{identity} version probe emitted no identity"),
        ));
    }
    if !tool_identity_matches(identity, observed) {
        return Err(CampaignError::new(
            ErrorCode::ToolUnavailable,
            format!("{identity} version probe emitted an unexpected identity: {observed:?}"),
        ));
    }
    Ok(observed.to_owned())
}

fn ambient_sanitizer_override(
    asan: Option<&OsStr>,
    disable_leaks: Option<&OsStr>,
    lsan: Option<&OsStr>,
) -> bool {
    asan.is_some() || disable_leaks.is_some() || lsan.is_some()
}

fn leak_sanitizer_available() -> io::Result<bool> {
    if !cfg!(target_os = "linux") {
        return Ok(true);
    }
    let status = match fs::read_to_string("/proc/self/status") {
        Ok(status) => status,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(error),
    };
    let no_new_privileges = status
        .lines()
        .any(|line| line.split_whitespace().eq(["NoNewPrivs:", "1"]));
    let filtered_seccomp = status
        .lines()
        .any(|line| line.split_whitespace().eq(["Seccomp:", "2"]));
    Ok(!(no_new_privileges && filtered_seccomp))
}

fn target_directory(root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("fuzz")
}

fn run_fuzzer(
    root: &Path,
    target: Target,
    generated: &Path,
    seeds: &Path,
    artifacts: &Path,
) -> Result<ProcessState, CampaignError> {
    let artifact_prefix = format!("-artifact_prefix={}/", artifacts.display());
    // TL-195: LeakSanitizer deterministically flags a 56-byte allocation made
    // by libFuzzer's own driver thread, not by tl-parse or either fuzz target
    // (see fuzz/lsan_suppressions.txt for the full stack and rationale).
    // Suppressing only that named allocation site keeps
    // ASAN_OPTIONS=detect_leaks=1 catching every real leak in the code under
    // test.
    let lsan_suppressions = root.join("fuzz").join("lsan_suppressions.txt");
    let mut command = Command::new("rustup");
    command
        .args(["run", "nightly", "cargo", "fuzz", "run", target.as_str()])
        .arg("--target-dir")
        .arg(target_directory(root))
        .arg(generated)
        .arg(seeds)
        .arg("--")
        .arg(artifact_prefix)
        .arg(format!("-runs={RUNS}"))
        .current_dir(root)
        .env("ASAN_OPTIONS", "detect_leaks=1")
        .env(
            "LSAN_OPTIONS",
            format!("suppressions={}", lsan_suppressions.display()),
        )
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    let child = command.group_spawn().map_err(|error| {
        CampaignError::new(
            ErrorCode::FuzzerUnavailable,
            format!("could not start cargo-fuzz: {error}"),
        )
    })?;

    supervise(child, DEADLINE)
}

fn supervise(mut child: GroupChild, deadline: Duration) -> Result<ProcessState, CampaignError> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(ProcessState::Exited(status.code().unwrap_or(-1))),
            Ok(None) if started.elapsed() < deadline => thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                child.kill().map_err(|error| {
                    CampaignError::new(
                        ErrorCode::SupervisionFailed,
                        format!("timed-out campaign could not be terminated: {error}"),
                    )
                })?;
                child.wait().map_err(|error| {
                    CampaignError::new(
                        ErrorCode::SupervisionFailed,
                        format!("terminated campaign could not be reaped: {error}"),
                    )
                })?;
                return Ok(ProcessState::TimedOut);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CampaignError::new(
                    ErrorCode::SupervisionFailed,
                    format!("could not observe cargo-fuzz: {error}"),
                ));
            }
        }
    }
}

fn inspect_artifacts(directory: &Path) -> Result<Vec<ArtifactReport>, String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("could not enumerate {}: {error}", directory.display()))?;
    let mut paths = entries
        .take(MAX_ARTIFACTS + 1)
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|error| format!("could not enumerate an artifact: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if paths.len() > MAX_ARTIFACTS {
        return Err(format!("artifact population exceeds {MAX_ARTIFACTS}"));
    }
    paths.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    for path in &paths {
        let Some(name) = path.file_name().and_then(OsStr::to_str) else {
            return Err("artifact name is not normalized UTF-8".to_owned());
        };
        if !normalized_file_name(name) {
            return Err(format!("artifact name {name:?} is not one component"));
        }
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("could not inspect {}: {error}", path.display()))?;
        if !metadata.file_type().is_file() {
            return Err(format!(
                "{} is not a regular, non-symlink file",
                path.display()
            ));
        }
        if metadata.len() > MAX_FILE_BYTES {
            return Err(format!(
                "{} exceeds the {MAX_FILE_BYTES}-byte artifact limit",
                path.display()
            ));
        }
    }
    paths
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or_else(|| "artifact name became invalid while reading".to_owned())?
                .to_owned();
            let mut bytes = Vec::new();
            File::open(&path)
                .and_then(|file| file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes))
                .map_err(|error| format!("could not read {}: {error}", path.display()))?;
            if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_FILE_BYTES {
                return Err(format!(
                    "{} grew beyond its byte limit while read",
                    path.display()
                ));
            }
            Ok(ArtifactReport {
                name,
                media_type: "application/octet-stream",
                byte_length: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                sha256: sha256_hex(&bytes),
            })
        })
        .collect()
}

fn unavailable(error: CampaignError) -> Execution {
    Execution {
        process: ProcessState::NotStarted,
        artifacts: Vec::new(),
        artifact_population_suspect: false,
        limitations: vec![error.limitation()],
        scratch: None,
    }
}

fn execute(
    root: &Path,
    target: Target,
    manifest_report: &mut SeedManifestReport,
    tools: &mut ToolReport,
    sanitizer: &mut SanitizerReport,
) -> Execution {
    let manifest = match load_seed_manifest(root, target, manifest_report) {
        Ok(manifest) => manifest,
        Err(error) => return unavailable(error),
    };
    if ambient_sanitizer_override(
        env::var_os("ASAN_OPTIONS").as_deref(),
        env::var_os("TL_PARSE_FUZZ_DISABLE_LEAKS").as_deref(),
        env::var_os("LSAN_OPTIONS").as_deref(),
    ) {
        sanitizer.leak_sanitizer = SanitizerState::RefusedAmbientOverride;
        return unavailable(CampaignError::new(
            ErrorCode::AmbientSanitizerOverride,
            "ASAN_OPTIONS, LSAN_OPTIONS, and TL_PARSE_FUZZ_DISABLE_LEAKS must be absent",
        ));
    }
    match leak_sanitizer_available() {
        Ok(true) => {
            sanitizer.leak_sanitizer = SanitizerState::Enabled;
            sanitizer.asan_options = Some("detect_leaks=1");
        }
        Ok(false) => {
            sanitizer.leak_sanitizer = SanitizerState::Unavailable;
            return unavailable(CampaignError::new(
                ErrorCode::LeakSanitizerUnavailable,
                "no-new-privileges plus filtered seccomp prevents LeakSanitizer",
            ));
        }
        Err(error) => {
            sanitizer.leak_sanitizer = SanitizerState::Unavailable;
            return unavailable(CampaignError::new(
                ErrorCode::LeakSanitizerUnavailable,
                format!("could not inspect LeakSanitizer availability: {error}"),
            ));
        }
    }

    tools.nightly_rust =
        match observe_version(&["run", "nightly", "rustc", "--version"], "nightly rustc") {
            Ok(version) => Some(version),
            Err(error) => return unavailable(error),
        };
    tools.cargo_fuzz = match observe_version(
        &["run", "nightly", "cargo", "fuzz", "--version"],
        "cargo-fuzz",
    ) {
        Ok(version) => Some(version),
        Err(error) => return unavailable(error),
    };

    let scratch_parent = target_directory(root).join("campaigns");
    if let Err(error) = fs::create_dir_all(&scratch_parent) {
        return unavailable(CampaignError::new(
            ErrorCode::ScratchUnavailable,
            format!("could not create {}: {error}", scratch_parent.display()),
        ));
    }
    let scratch = match tempfile::Builder::new()
        .prefix(&format!("{}-", target.as_str()))
        .tempdir_in(&scratch_parent)
    {
        Ok(directory) => directory,
        Err(error) => {
            return unavailable(CampaignError::new(
                ErrorCode::ScratchUnavailable,
                format!("could not allocate campaign scratch: {error}"),
            ));
        }
    };
    let generated = scratch.path().join("generated");
    let seeds = scratch.path().join("seeds");
    let artifacts = scratch.path().join("artifacts");
    for directory in [&generated, &seeds, &artifacts] {
        if let Err(error) = fs::create_dir(directory) {
            return Execution {
                process: ProcessState::NotStarted,
                artifacts: Vec::new(),
                artifact_population_suspect: false,
                limitations: vec![CampaignError::new(
                    ErrorCode::ScratchUnavailable,
                    format!("could not create {}: {error}", directory.display()),
                )
                .limitation()],
                scratch: Some(scratch),
            };
        }
    }
    if let Err(error) = copy_seeds(&manifest, &seeds) {
        return Execution {
            process: ProcessState::NotStarted,
            artifacts: Vec::new(),
            artifact_population_suspect: false,
            limitations: vec![error.limitation()],
            scratch: Some(scratch),
        };
    }

    let mut limitations = Vec::new();
    let process = match run_fuzzer(root, target, &generated, &seeds, &artifacts) {
        Ok(process) => process,
        Err(error) => {
            let process = error.process_state();
            limitations.push(error.limitation());
            process
        }
    };
    let (artifact_reports, artifact_population_suspect) = match inspect_artifacts(&artifacts) {
        Ok(reports) => (reports, false),
        Err(problem) => {
            limitations.push(format!("artifact_population_suspect: {problem}"));
            (Vec::new(), true)
        }
    };
    Execution {
        process,
        artifacts: artifact_reports,
        artifact_population_suspect,
        limitations,
        scratch: Some(scratch),
    }
}

fn document(root: &Path, target: Target) -> (CampaignDocument, Option<TempDir>) {
    let mut manifest = SeedManifestReport {
        path: format!("fuzz/corpus/{}/SHA256SUMS", target.as_str()),
        ..SeedManifestReport::default()
    };
    let mut tools = ToolReport::default();
    let mut sanitizer = SanitizerReport {
        leak_sanitizer: SanitizerState::NotChecked,
        asan_options: None,
    };
    let mut execution = execute(root, target, &mut manifest, &mut tools, &mut sanitizer);
    let artifacts_present =
        execution.artifact_population_suspect || !execution.artifacts.is_empty();
    let (mut outcome, elapsed) = classify(execution.process, artifacts_present);
    if execution.artifact_population_suspect {
        outcome = DomainOutcome::Suspect;
    }
    execution
        .limitations
        .insert(0, ATTACHMENT_LIMITATION.to_owned());
    let result = CampaignDocument {
        protocol: PROTOCOL,
        entries: [NormalizedEntry {
            symbol: target.symbol(),
            outcome: outcome.normalized(),
            trace_ids: [
                "TC-047",
                "FR-005-AC-2",
                "FR-006-AC-2",
                "NFR-003-AC-1",
                "SUITE-010",
            ],
            domain_outcome: outcome,
        }],
        campaign: Campaign {
            target: target.as_str(),
            requested_runs: RUNS,
            deadline_seconds: DEADLINE.as_secs(),
            seed_manifest: manifest,
            tools,
            sanitizer,
            process: execution.process.into(),
            elapsed_time_class: elapsed,
            domain_outcome: outcome,
            limitations: execution.limitations,
            artifacts: execution.artifacts,
        },
    };
    (result, execution.scratch)
}

fn emit(document: &CampaignDocument) -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, document)?;
    output.write_all(b"\n")
}

fn validate_mode(mut arguments: impl Iterator<Item = std::ffi::OsString>) -> ExitCode {
    let Some(proof_id) = arguments.next() else {
        eprintln!("usage: fuzz_campaign validate <proof-id> <result-path>");
        return ExitCode::from(64);
    };
    let Some(target) = Target::from_proof_id(&proof_id) else {
        eprintln!("unknown fuzz proof identity {proof_id:?}");
        return ExitCode::from(64);
    };
    let Some(path) = arguments.next().map(PathBuf::from) else {
        eprintln!("usage: fuzz_campaign validate <proof-id> <result-path>");
        return ExitCode::from(64);
    };
    if arguments.next().is_some() {
        eprintln!("usage: fuzz_campaign validate <proof-id> <result-path>");
        return ExitCode::from(64);
    }
    let raw = match bounded_result_bytes(&path) {
        Ok(raw) => raw,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(65);
        }
    };
    match validate_campaign_result(Path::new(env!("CARGO_MANIFEST_DIR")), target, &raw) {
        Ok(result) => {
            println!("{result}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(65)
        }
    }
}

fn main() -> ExitCode {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let Some(raw_target) = arguments.next() else {
        eprintln!(
            "usage: fuzz_campaign <parser|clean_ascii_v2> | validate <proof-id> <result-path>"
        );
        return ExitCode::from(64);
    };
    if raw_target == "validate" {
        return validate_mode(arguments);
    }
    if arguments.next().is_some() {
        eprintln!("usage: fuzz_campaign <parser|clean_ascii_v2>");
        return ExitCode::from(64);
    }
    let Some(target) = Target::parse(&raw_target) else {
        eprintln!(
            "unknown fuzz target {:?}; expected parser or clean_ascii_v2",
            raw_target
        );
        return ExitCode::from(64);
    };

    let (document, scratch) = document(Path::new(env!("CARGO_MANIFEST_DIR")), target);
    let exit_code = document.campaign.domain_outcome.exit_code();
    if exit_code != 0 {
        if let Some(scratch) = scratch {
            let retained = scratch.into_path();
            eprintln!("fuzz campaign scratch retained at {}", retained.display());
        }
    }
    if let Err(error) = emit(&document) {
        eprintln!("could not emit structured fuzz campaign result: {error}");
        return ExitCode::from(74);
    }
    ExitCode::from(exit_code)
}

#[cfg(test)]
mod tests {
    use super::{
        ambient_sanitizer_override, classify, document, inspect_artifacts, load_seed_manifest,
        read_bounded_output, sha256_hex, supervise, tool_identity_matches,
        validate_campaign_result, ArtifactReport, Campaign, CampaignDocument, DomainOutcome,
        ElapsedClass, NormalizedEntry, NormalizedOutcome, ProcessReport, ProcessState,
        ProcessStateName, SanitizerReport, SanitizerState, SeedManifestReport, Target, ToolReport,
        MAX_ARTIFACTS, MAX_FILE_BYTES, MAX_TOOL_OUTPUT_BYTES, PROTOCOL,
    };
    use std::ffi::OsStr;
    use std::fs;
    use std::os::unix::fs::symlink;

    use command_group::CommandGroup;
    use tempfile::TempDir;

    fn manifest_fixture(lines: &[(&str, &[u8])]) -> TempDir {
        let root = tempfile::tempdir().expect("fixture root");
        let corpus = root.path().join("fuzz/corpus/parser");
        fs::create_dir_all(&corpus).expect("fixture corpus");
        let mut manifest = String::new();
        for (name, bytes) in lines {
            fs::write(corpus.join(name), bytes).expect("fixture seed");
            manifest.push_str(&format!("{}  {name}\n", sha256_hex(bytes)));
        }
        fs::write(corpus.join("SHA256SUMS"), manifest).expect("fixture manifest");
        root
    }

    fn passing_result(root: &std::path::Path, target: Target) -> serde_json::Value {
        let manifest_path = format!("fuzz/corpus/{}/SHA256SUMS", target.as_str());
        let manifest = fs::read(root.join(&manifest_path)).expect("read fixture manifest");
        let count = manifest
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .count();
        serde_json::to_value(CampaignDocument {
            protocol: PROTOCOL,
            entries: [NormalizedEntry {
                symbol: target.symbol(),
                outcome: NormalizedOutcome::Pass,
                trace_ids: [
                    "TC-047",
                    "FR-005-AC-2",
                    "FR-006-AC-2",
                    "NFR-003-AC-1",
                    "SUITE-010",
                ],
                domain_outcome: DomainOutcome::Pass,
            }],
            campaign: Campaign {
                target: target.as_str(),
                requested_runs: 64,
                deadline_seconds: 300,
                seed_manifest: SeedManifestReport {
                    path: manifest_path,
                    sha256: Some(sha256_hex(&manifest)),
                    count: Some(u32::try_from(count).expect("manifest count")),
                },
                tools: ToolReport {
                    nightly_rust: Some("rustc 1.97.0-nightly (revision)".to_owned()),
                    cargo_fuzz: Some("cargo-fuzz 0.13.2".to_owned()),
                },
                sanitizer: SanitizerReport {
                    leak_sanitizer: SanitizerState::Enabled,
                    asan_options: Some("detect_leaks=1"),
                },
                process: ProcessReport {
                    state: ProcessStateName::Exited,
                    exit_code: Some(0),
                },
                elapsed_time_class: ElapsedClass::WithinDeadline,
                domain_outcome: DomainOutcome::Pass,
                limitations: vec!["binary attachment deferred".to_owned()],
                artifacts: Vec::new(),
            },
        })
        .expect("serialize passing campaign")
    }

    fn validate_value(
        root: &std::path::Path,
        target: Target,
        value: &serde_json::Value,
    ) -> Result<&'static str, String> {
        validate_campaign_result(
            root,
            target,
            &serde_json::to_vec(value).expect("serialize mutation"),
        )
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_046_classifies_every_process_and_artifact_combination() {
        assert_eq!(
            classify(ProcessState::Exited(0), false),
            (DomainOutcome::Pass, ElapsedClass::WithinDeadline)
        );
        assert_eq!(
            classify(ProcessState::Exited(9), false),
            (DomainOutcome::Fail, ElapsedClass::WithinDeadline)
        );
        assert_eq!(
            classify(ProcessState::Exited(0), true),
            (DomainOutcome::Suspect, ElapsedClass::WithinDeadline)
        );
        assert_eq!(
            classify(ProcessState::TimedOut, false),
            (DomainOutcome::Unavailable, ElapsedClass::DeadlineExceeded)
        );
        assert_eq!(
            classify(ProcessState::NotStarted, false),
            (DomainOutcome::Unavailable, ElapsedClass::NotRun)
        );
        assert_eq!(
            classify(ProcessState::SupervisionFailed, false),
            (DomainOutcome::Unavailable, ElapsedClass::WithinDeadline)
        );
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_046_refuses_target_sanitizer_and_tool_identity_mutations() {
        assert_eq!(Target::parse(OsStr::new("parser")), Some(Target::Parser));
        assert_eq!(
            Target::parse(OsStr::new("clean_ascii_v2")),
            Some(Target::CleanAsciiV2)
        );
        assert_eq!(Target::parse(OsStr::new("../parser")), None);
        assert!(ambient_sanitizer_override(Some(OsStr::new("")), None, None));
        assert!(ambient_sanitizer_override(
            None,
            Some(OsStr::new("0")),
            None
        ));
        assert!(ambient_sanitizer_override(None, None, Some(OsStr::new(""))));
        assert!(!ambient_sanitizer_override(None, None, None));
        assert!(tool_identity_matches(
            "nightly rustc",
            "rustc 1.97.0-nightly (revision)"
        ));
        assert!(tool_identity_matches("cargo-fuzz", "cargo-fuzz 0.13.2"));
        assert!(!tool_identity_matches("nightly rustc", "rustc 1.75.0"));
        assert!(!tool_identity_matches("cargo-fuzz", "cargo 1.97.0"));
        let bounded =
            read_bounded_output(std::io::Cursor::new(vec![b'x'; MAX_TOOL_OUTPUT_BYTES + 1]));
        assert_eq!(bounded.bytes.len(), MAX_TOOL_OUTPUT_BYTES);
        assert!(bounded.exceeded);
        assert!(bounded.error.is_none());
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_046_zero_deadline_terminates_and_reaps_the_whole_process_group() {
        let mut command = std::process::Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let child = command.group_spawn().expect("spawn process group");
        assert_eq!(
            supervise(child, std::time::Duration::ZERO).expect("supervise process group"),
            ProcessState::TimedOut
        );
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_046_accepts_exact_manifest_bytes_and_refuses_digest_or_name_mutations() {
        let root = manifest_fixture(&[("one.txt", b"one"), ("two.txt", b"two")]);
        let mut report = super::SeedManifestReport::default();
        let loaded =
            load_seed_manifest(root.path(), Target::Parser, &mut report).expect("valid manifest");
        assert_eq!(loaded.seeds.len(), 2);
        assert_eq!(loaded.seeds[0].bytes, b"one");
        assert_eq!(report.count, Some(2));
        assert!(report.sha256.is_some());
        fs::write(root.path().join("fuzz/corpus/parser/one.txt"), b"changed").expect("mutate seed");
        let mismatch = load_seed_manifest(root.path(), Target::Parser, &mut report)
            .expect_err("digest mutation");
        assert_eq!(mismatch.code.as_str(), "seed_digest_mismatch");
        let corpus = root.path().join("fuzz/corpus/parser");
        fs::write(
            corpus.join("SHA256SUMS"),
            format!("{}  ../escape\n", sha256_hex(b"escape")),
        )
        .expect("mutate manifest");
        let escape = load_seed_manifest(root.path(), Target::Parser, &mut report)
            .expect_err("escaping name");
        assert_eq!(escape.code.as_str(), "manifest_malformed");

        fs::write(
            corpus.join("SHA256SUMS"),
            format!(
                "{}  one.txt\n{}  one.txt\n",
                sha256_hex(b"one"),
                sha256_hex(b"one")
            ),
        )
        .expect("duplicate manifest");
        let duplicate = load_seed_manifest(root.path(), Target::Parser, &mut report)
            .expect_err("duplicate name");
        assert_eq!(duplicate.code.as_str(), "manifest_malformed");

        fs::remove_file(corpus.join("one.txt")).expect("remove mutated seed");
        symlink(corpus.join("two.txt"), corpus.join("one.txt")).expect("seed symlink");
        fs::write(
            corpus.join("SHA256SUMS"),
            format!("{}  one.txt\n", sha256_hex(b"two")),
        )
        .expect("symlink manifest");
        let linked = load_seed_manifest(root.path(), Target::Parser, &mut report)
            .expect_err("symlinked seed");
        assert_eq!(linked.code.as_str(), "seed_invalid");
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_046_bounds_artifacts_and_hashes_exact_bytes() {
        let root = tempfile::tempdir().expect("artifact root");
        fs::write(root.path().join("crash-a"), b"exact crash bytes").expect("artifact");
        let reports = inspect_artifacts(root.path()).expect("valid artifacts");
        assert_eq!(
            reports,
            [ArtifactReport {
                name: "crash-a".to_owned(),
                media_type: "application/octet-stream",
                byte_length: 17,
                sha256: sha256_hex(b"exact crash bytes")
            }]
        );
        for index in 0..=MAX_ARTIFACTS {
            fs::write(root.path().join(format!("many-{index}")), []).expect("extra artifact");
        }
        let excessive = inspect_artifacts(root.path()).expect_err("population bound");
        assert!(excessive.contains("population exceeds"), "{excessive}");
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_046_refuses_symlinked_and_oversized_artifacts() {
        let root = tempfile::tempdir().expect("artifact root");
        fs::write(root.path().join("outside"), b"outside").expect("symlink target");
        symlink(root.path().join("outside"), root.path().join("crash-link")).expect("symlink");
        let linked = inspect_artifacts(root.path()).expect_err("symlink suspect");
        assert!(linked.contains("non-symlink"), "{linked}");
        fs::remove_file(root.path().join("crash-link")).expect("remove symlink");
        fs::remove_file(root.path().join("outside")).expect("remove target");
        let oversized = root.path().join("oversized");
        let file = fs::File::create(&oversized).expect("oversized artifact");
        file.set_len(MAX_FILE_BYTES + 1)
            .expect("set oversized length");
        let excessive = inspect_artifacts(root.path()).expect_err("size suspect");
        assert!(excessive.contains("artifact limit"), "{excessive}");
    }

    // Trace: TC-047, FR-005-AC-2, FR-006-AC-2
    #[test]
    fn tc_046_unavailable_result_is_typed_versioned_and_non_passing() {
        let root = tempfile::tempdir().expect("campaign root");
        let (result, scratch) = document(root.path(), Target::Parser);
        assert!(scratch.is_none());
        assert_eq!(result.protocol, PROTOCOL);
        assert_eq!(result.entries[0].symbol, "fuzz:parser");
        assert_eq!(result.entries[0].outcome, super::NormalizedOutcome::Skip);
        assert_eq!(result.entries[0].domain_outcome, DomainOutcome::Unavailable);
        assert_eq!(result.campaign.domain_outcome.exit_code(), 2);
        assert_eq!(result.campaign.requested_runs, 64);
        assert_eq!(result.campaign.deadline_seconds, 300);
        let encoded = serde_json::to_value(&result).expect("serialize result");
        assert_eq!(encoded["entries"][0]["traceIds"][0], "TC-047");
        assert_eq!(encoded["campaign"]["domainOutcome"], "unavailable");
    }

    // Trace: TC-047, FR-005-AC-2, FR-006-AC-2, NFR-003-AC-1
    #[test]
    fn tc_046_rust_adapter_accepts_coherent_outcomes_and_refuses_identity_mutations() {
        let root = manifest_fixture(&[("one.txt", b"one")]);
        let target = Target::Parser;
        let passing = passing_result(root.path(), target);
        assert_eq!(validate_value(root.path(), target, &passing), Ok("passed"));

        let mut failing = passing.clone();
        failing["entries"][0]["outcome"] = "fail".into();
        failing["entries"][0]["domainOutcome"] = "fail".into();
        failing["campaign"]["domainOutcome"] = "fail".into();
        failing["campaign"]["process"]["exitCode"] = 9.into();
        assert_eq!(validate_value(root.path(), target, &failing), Ok("failed"));

        let mut unavailable = passing.clone();
        unavailable["entries"][0]["outcome"] = "skip".into();
        unavailable["entries"][0]["domainOutcome"] = "unavailable".into();
        unavailable["campaign"]["domainOutcome"] = "unavailable".into();
        unavailable["campaign"]["process"]["state"] = "timed_out".into();
        unavailable["campaign"]["process"]["exitCode"] = serde_json::Value::Null;
        unavailable["campaign"]["elapsedTimeClass"] = "deadline_exceeded".into();
        assert_eq!(
            validate_value(root.path(), target, &unavailable),
            Ok("unavailable")
        );

        let mut suspect = passing.clone();
        suspect["entries"][0]["outcome"] = "fail".into();
        suspect["entries"][0]["domainOutcome"] = "suspect".into();
        suspect["campaign"]["domainOutcome"] = "suspect".into();
        suspect["campaign"]["artifacts"] = serde_json::json!([{
            "name": "crash-example",
            "mediaType": "application/octet-stream",
            "byteLength": 1,
            "sha256": sha256_hex(b"x"),
        }]);
        assert_eq!(validate_value(root.path(), target, &suspect), Ok("failed"));

        let mutations: &[(&str, fn(&mut serde_json::Value))] = &[
            ("protocol", |value| {
                value["protocol"] = "other.fuzz/v1".into()
            }),
            ("target", |value| {
                value["campaign"]["target"] = "clean_ascii_v2".into()
            }),
            ("symbol", |value| {
                value["entries"][0]["symbol"] = "fuzz:clean_ascii_v2".into()
            }),
            ("runs", |value| {
                value["campaign"]["requestedRuns"] = 0.into()
            }),
            ("deadline", |value| {
                value["campaign"]["deadlineSeconds"] = 301.into()
            }),
            ("manifest", |value| {
                value["campaign"]["seedManifest"]["sha256"] = "0".repeat(64).into()
            }),
            ("tool", |value| {
                value["campaign"]["tools"]["nightlyRust"] = "rustc 1.75.0".into()
            }),
            ("sanitizer", |value| {
                value["campaign"]["sanitizer"]["asanOptions"] = serde_json::Value::Null
            }),
            ("process", |value| {
                value["campaign"]["process"]["exitCode"] = 9.into()
            }),
            ("outcome", |value| {
                value["entries"][0]["domainOutcome"] = "suspect".into()
            }),
        ];
        for (name, mutate) in mutations {
            let mut mutated = passing.clone();
            mutate(&mut mutated);
            assert!(
                validate_value(root.path(), target, &mutated).is_err(),
                "{name} mutation was accepted"
            );
        }
    }
}
