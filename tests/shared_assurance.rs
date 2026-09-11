//! Tests for the shared assurance intake path (FR-006).
//!
//! These follow this repository's own binding idiom: a `// Trace:` comment above
//! each `#[test]`, which is what Quire's census reads. They invoke the gates
//! rather than reimplementing them, because a test that recomputes what a gate
//! computes is a second implementation that can agree with itself while both are
//! wrong.
//!
//! A missing prerequisite is a failure here, never a skip. A gate that stands
//! down when its dependency is absent reports the same green as one that ran.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use serde_json::Value;
use serde_yaml_ng::Value as YamlValue;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The interpreter `make assurance-env` builds. Its absence is an error.
fn assurance_python() -> PathBuf {
    let path = std::env::var_os("ASSURANCE_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join(".venv-assurance/bin/python"));
    assert!(
        path.is_file(),
        "the pinned assurance interpreter is missing at {}. Run `make assurance-env`. \
         This is a failure and not a skip: a gate that stands down when its dependency \
         is absent reports the same green as one that ran.",
        path.display()
    );
    path
}

fn run(program: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root())
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()));
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn workflow_run_scripts(source: &str) -> Result<Vec<String>, Vec<String>> {
    let document: YamlValue = serde_yaml_ng::from_str(source)
        .map_err(|error| vec![format!("invalid workflow YAML: {error}")])?;
    let mut scripts = Vec::new();
    let mut errors = Vec::new();

    let key = |name: &str| YamlValue::String(name.to_owned());
    let Some(jobs) = document
        .as_mapping()
        .and_then(|root| root.get(key("jobs")))
        .and_then(YamlValue::as_mapping)
    else {
        return Err(vec!["workflow has no jobs mapping".to_owned()]);
    };

    for (job_name, job) in jobs {
        let Some(job) = job.as_mapping() else {
            errors.push(format!("workflow job {job_name:?} is not a mapping"));
            continue;
        };
        let Some(steps) = job.get(key("steps")) else {
            continue;
        };
        let Some(steps) = steps.as_sequence() else {
            errors.push(format!(
                "workflow job {job_name:?} steps are not a sequence"
            ));
            continue;
        };
        for (index, step) in steps.iter().enumerate() {
            let Some(step) = step.as_mapping() else {
                errors.push(format!(
                    "workflow job {job_name:?} step {index} is not a mapping"
                ));
                continue;
            };
            let Some(run) = step.get(key("run")) else {
                continue;
            };
            match run.as_str() {
                Some(script) => scripts.push(script.to_owned()),
                None => errors.push(format!(
                    "workflow job {job_name:?} step {index} run value is not a scalar string"
                )),
            }
        }
    }
    if errors.is_empty() {
        Ok(scripts)
    } else {
        Err(errors)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShellToken {
    Word(String),
    Boundary,
}

fn shell_tokens(script: &str) -> Result<Vec<ShellToken>, String> {
    let mut tokens = Vec::new();
    let mut word = String::new();
    let mut word_started = false;
    let mut quote = None;
    let mut escaped = false;
    let mut characters = script.chars().peekable();
    let flush = |tokens: &mut Vec<ShellToken>, word: &mut String, word_started: &mut bool| {
        if *word_started {
            tokens.push(ShellToken::Word(std::mem::take(word)));
            *word_started = false;
        }
    };
    while let Some(character) = characters.next() {
        if escaped {
            word_started = true;
            if character != '\n' {
                word.push(character);
            }
            escaped = false;
            continue;
        }
        if quote == Some('\'') {
            if character == '\'' {
                quote = None;
            } else {
                word_started = true;
                word.push(character);
            }
            continue;
        }
        if quote == Some('"') {
            match character {
                '"' => quote = None,
                '\\' => escaped = true,
                _ => {
                    word_started = true;
                    word.push(character);
                }
            }
            continue;
        }
        match character {
            '\'' | '"' => {
                word_started = true;
                quote = Some(character);
            }
            '\\' => {
                word_started = true;
                escaped = true;
            }
            ' ' | '\t' | '\r' => flush(&mut tokens, &mut word, &mut word_started),
            '#' if !word_started => {
                for comment_character in characters.by_ref() {
                    if comment_character == '\n' {
                        if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                            tokens.push(ShellToken::Boundary);
                        }
                        break;
                    }
                }
            }
            '\n' | ';' | '|' | '&' => {
                flush(&mut tokens, &mut word, &mut word_started);
                if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                    tokens.push(ShellToken::Boundary);
                }
                if matches!(character, '|' | '&') && characters.peek() == Some(&character) {
                    characters.next();
                }
            }
            '(' | '{' if !word_started => {
                if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                    tokens.push(ShellToken::Boundary);
                }
            }
            ')' | '}' => {
                flush(&mut tokens, &mut word, &mut word_started);
                if !matches!(tokens.last(), Some(ShellToken::Boundary)) {
                    tokens.push(ShellToken::Boundary);
                }
            }
            _ => {
                word_started = true;
                word.push(character);
            }
        }
    }
    if escaped || quote.is_some() {
        return Err(format!(
            "shell script has an unterminated token near {word:?}"
        ));
    }
    flush(&mut tokens, &mut word, &mut word_started);
    Ok(tokens)
}

const NPM_INSTALL_ALIASES: &[&str] = &[
    "install", "add", "i", "in", "ins", "inst", "insta", "instal", "isnt", "isnta", "isntal",
    "isntall",
];

fn is_npm_install_alias(word: &str) -> bool {
    NPM_INSTALL_ALIASES.contains(&word)
}

fn is_shell_interpreter(word: &str) -> bool {
    matches!(word.rsplit('/').next(), Some("sh" | "bash"))
}

fn is_npm_executable(word: &str) -> bool {
    word.rsplit('/').next() == Some("npm")
}

fn is_shell_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let mut characters = name.chars();
    matches!(characters.next(), Some(first) if first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn shell_redirection_word_span(word: &str) -> Option<usize> {
    let remainder = word.trim_start_matches(|character: char| character.is_ascii_digit());
    let operator = ["<<<", "<<-", ">>", "<<", "<>", "<&", ">&", ">|", ">", "<"]
        .into_iter()
        .find(|operator| remainder.starts_with(*operator))?;
    Some(usize::from(remainder.len() == operator.len()) + 1)
}

fn command_executable_index(words: &[&str]) -> Option<usize> {
    let mut index = 0;
    while let Some(word) = words.get(index).copied() {
        if is_shell_assignment(word) {
            index += 1;
        } else if let Some(span) = shell_redirection_word_span(word) {
            index += span;
        } else {
            break;
        }
    }
    if words.get(index).copied() == Some("env") {
        index += 1;
        while let Some(word) = words.get(index).copied() {
            if matches!(
                word,
                "-u" | "--unset" | "-C" | "--chdir" | "-S" | "--split-string"
            ) {
                index += 2;
            } else if word.starts_with('-') || is_shell_assignment(word) {
                index += 1;
            } else if let Some(span) = shell_redirection_word_span(word) {
                index += span;
            } else {
                break;
            }
        }
    }
    words.get(index).map(|_| index)
}

fn is_shell_command_option(word: &str) -> bool {
    word.starts_with('-') && !word.starts_with("--") && word[1..].contains('c')
}

fn scan_ix_flow_packages(
    script: &str,
    depth: usize,
    packages: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    if depth > 8 {
        errors.push("nested shell command depth exceeds 8".to_owned());
        return;
    }
    let tokens = match shell_tokens(script) {
        Ok(tokens) => tokens,
        Err(error) => {
            errors.push(error);
            return;
        }
    };
    for command in tokens.split(|token| *token == ShellToken::Boundary) {
        let words: Vec<&str> = command
            .iter()
            .filter_map(|token| match token {
                ShellToken::Word(word) => Some(word.as_str()),
                ShellToken::Boundary => None,
            })
            .collect();

        let Some(executable_index) = command_executable_index(&words) else {
            continue;
        };
        let executable = words[executable_index];
        if is_shell_interpreter(executable) {
            let Some(command_option) = words[executable_index + 1..]
                .iter()
                .position(|word| is_shell_command_option(word))
                .map(|offset| executable_index + 1 + offset)
            else {
                continue;
            };
            let nested = words[command_option + 1..]
                .iter()
                .find(|word| **word != "--");
            match nested {
                Some(nested) => scan_ix_flow_packages(nested, depth + 1, packages, errors),
                None => {
                    errors.push("shell -c option has no statically classifiable script".to_owned())
                }
            }
        }

        if is_npm_executable(executable) {
            let Some((install_offset, _)) = words[executable_index + 1..]
                .iter()
                .enumerate()
                .find(|(_, word)| is_npm_install_alias(word))
            else {
                continue;
            };
            let install_index = executable_index + 1 + install_offset;
            for argument in &words[install_index + 1..] {
                if *argument == "--" || argument.starts_with('-') {
                    continue;
                }
                if argument.to_ascii_lowercase().contains("ix-flow") {
                    packages.push((*argument).to_owned());
                }
            }
        }
    }
}

fn workflow_ix_flow_packages(source: &str) -> (Vec<String>, Vec<String>) {
    let scripts = match workflow_run_scripts(source) {
        Ok(scripts) => scripts,
        Err(errors) => return (Vec::new(), errors),
    };
    let mut packages = Vec::new();
    let mut errors = Vec::new();
    for script in scripts {
        scan_ix_flow_packages(&script, 0, &mut packages, &mut errors);
    }
    (packages, errors)
}

fn workflow_trigger_names(source: &str) -> Result<Vec<String>, String> {
    let document: YamlValue = serde_yaml_ng::from_str(source)
        .map_err(|error| format!("invalid workflow YAML: {error}"))?;
    let root = document
        .as_mapping()
        .ok_or_else(|| "workflow document is not a mapping".to_owned())?;
    let on = root
        .get(YamlValue::String("on".to_owned()))
        .ok_or_else(|| "workflow has no on key".to_owned())?;
    match on {
        YamlValue::String(trigger) => Ok(vec![trigger.clone()]),
        YamlValue::Sequence(triggers) => triggers
            .iter()
            .map(|trigger| {
                trigger
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "workflow on sequence contains a non-string trigger".to_owned())
            })
            .collect(),
        YamlValue::Mapping(triggers) => triggers
            .keys()
            .map(|trigger| {
                trigger
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "workflow on mapping contains a non-string trigger".to_owned())
            })
            .collect(),
        _ => Err("workflow on value is not a trigger, sequence, or mapping".to_owned()),
    }
}

fn hosted_workflow_control_errors(source: &str) -> Vec<String> {
    // These literals are authored independently from the workflow. Review of
    // `.github/workflows/ci.yml` is the second control against a coordinated
    // edit of this census and its expected side.
    const EXPECTED_PACKAGE: &str = "@agent-ix/ix-flow@0.0.4";
    const EXPECTED_TRIGGER: &str = "workflow_dispatch";

    let (packages, mut errors) = workflow_ix_flow_packages(source);
    if packages != [EXPECTED_PACKAGE] {
        errors.push(format!(
            "executable ix-flow packages must be exactly [{EXPECTED_PACKAGE:?}], observed {packages:?}"
        ));
    }
    let triggers = match workflow_trigger_names(source) {
        Ok(triggers) => triggers,
        Err(error) => {
            errors.push(error);
            Vec::new()
        }
    };
    if triggers != [EXPECTED_TRIGGER] {
        errors.push(format!(
            "hosted triggers must be exactly [{EXPECTED_TRIGGER:?}], observed {triggers:?}"
        ));
    }
    errors
}

fn replace_first_install_invocation(source: &str, replacement: &str) -> String {
    for alias in NPM_INSTALL_ALIASES {
        let needle = format!("npm {alias} --global");
        if source.contains(&needle) {
            return source.replacen(&needle, replacement, 1);
        }
    }
    panic!("workflow contains no supported npm install invocation")
}

fn json_gate(program: &Path, arguments: &[&str]) -> Value {
    let (code, stdout, stderr) = run(program, arguments);
    assert_eq!(code, 0, "{arguments:?} exited {code}\n{stdout}\n{stderr}");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{arguments:?} did not emit JSON: {error}\n{stdout}"))
}

fn head_revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root())
        .output()
        .expect("git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// The chain is expensive and several tests read it. It runs once per test
/// binary, and every reader sees the same run rather than a different one.
static CHAIN: OnceLock<Value> = OnceLock::new();

/// Serializes the two tests that read or temporarily mutate shared pin inputs.
/// The guard is deliberately private to this test binary: it protects the
/// repository fixture, not an assurance runtime concern.
static SHARED_PIN_INPUTS: Mutex<()> = Mutex::new(());

fn shared_pin_inputs() -> MutexGuard<'static, ()> {
    SHARED_PIN_INPUTS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct TrackedFileRestore {
    path: PathBuf,
    original: Vec<u8>,
    restored: bool,
}

impl TrackedFileRestore {
    fn new(path: PathBuf) -> Self {
        let original = fs::read(&path)
            .unwrap_or_else(|error| panic!("read tracked input {}: {error}", path.display()));
        Self {
            path,
            original,
            restored: false,
        }
    }

    fn original(&self) -> &[u8] {
        &self.original
    }

    fn restore(&mut self) {
        fs::write(&self.path, &self.original).unwrap_or_else(|error| {
            panic!(
                "restore tracked input {} after mutation: {error}",
                self.path.display()
            )
        });
        self.restored = true;
    }
}

impl Drop for TrackedFileRestore {
    fn drop(&mut self) {
        if self.restored {
            return;
        }
        if let Err(error) = fs::write(&self.path, &self.original) {
            if std::thread::panicking() {
                eprintln!(
                    "failed to restore tracked input {} while unwinding: {error}",
                    self.path.display()
                );
            } else {
                panic!(
                    "restore tracked input {} after mutation: {error}",
                    self.path.display()
                );
            }
        }
    }
}

fn normalized_frontmatter_scalar(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn review_id(contents: &str, path: &str) -> String {
    let mut lines = contents.lines();
    assert_eq!(
        lines.next(),
        Some("---"),
        "tracked review {path} has no YAML frontmatter"
    );
    let mut id = None;
    let mut artifact_type = None;
    for line in lines {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("id:") {
            id = Some(normalized_frontmatter_scalar(value).to_owned());
        }
        if let Some(value) = line.strip_prefix("type:") {
            artifact_type = Some(normalized_frontmatter_scalar(value).to_owned());
        }
    }
    assert_eq!(
        artifact_type.as_deref(),
        Some("SpecReview"),
        "tracked review {path} is not a SpecReview artifact"
    );
    id.unwrap_or_else(|| panic!("tracked review {path} has no frontmatter id"))
}

fn duplicate_review_ids(
    reviews: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, Vec<String>> {
    let mut paths_by_id = BTreeMap::<String, Vec<String>>::new();
    for (path, contents) in reviews {
        paths_by_id
            .entry(review_id(&contents, &path))
            .or_default()
            .push(path);
    }
    assert!(
        !paths_by_id.is_empty(),
        "tracked SpecReview census is empty; uniqueness would be vacuous"
    );
    paths_by_id
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect()
}

// Trace: TC-029, NFR-003-AC-4
#[test]
fn every_tracked_spec_review_id_is_unique() {
    let output = Command::new("git")
        .args(["ls-files", "-z", "spec/reviews"])
        .current_dir(root())
        .output()
        .expect("git ls-files could not enumerate tracked reviews");
    assert!(
        output.status.success(),
        "git ls-files could not enumerate tracked reviews: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let paths = String::from_utf8(output.stdout).expect("tracked review paths are not UTF-8");
    let reviews = paths
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(|path| {
            let contents = fs::read_to_string(root().join(path))
                .unwrap_or_else(|error| panic!("could not read tracked review {path}: {error}"));
            (path.to_owned(), contents)
        });
    let duplicates = duplicate_review_ids(reviews);
    assert!(
        duplicates.is_empty(),
        "duplicate tracked SpecReview ids: {duplicates:?}"
    );
}

// Trace: TC-029, NFR-003-AC-4
#[test]
fn quoted_review_identity_collides_with_its_plain_yaml_value() {
    let duplicates = duplicate_review_ids([
        (
            "plain.md".to_owned(),
            "---\nid: SR-091\ntype: SpecReview\n---\n".to_owned(),
        ),
        (
            "quoted.md".to_owned(),
            "---\nid: \"SR-091\"\ntype: SpecReview\n---\n".to_owned(),
        ),
    ]);
    assert_eq!(
        duplicates.get("SR-091"),
        Some(&vec!["plain.md".to_owned(), "quoted.md".to_owned()])
    );
}

// Trace: TC-029, NFR-003-AC-4
#[test]
#[should_panic(expected = "tracked SpecReview census is empty")]
fn review_identity_census_refuses_an_empty_set() {
    let _ = duplicate_review_ids(std::iter::empty::<(String, String)>());
}

fn chain_report() -> &'static Value {
    CHAIN.get_or_init(|| {
        // The chain runs under the system interpreter: it only shells out to
        // quoin and never imports engineering-assurance.
        let revision = head_revision();
        let (code, stdout, stderr) = run(
            Path::new("python3"),
            &[
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
                "--json",
            ],
        );
        assert_eq!(code, 0, "the assurance chain exited {code}\n{stderr}");
        serde_json::from_str(&stdout).expect("the assurance chain did not emit JSON")
    })
}

// Trace: TC-022, FR-006-AC-1
#[test]
fn every_shared_pin_is_classified_by_the_packaged_matrix() {
    let _shared_inputs = shared_pin_inputs();
    let python = assurance_python();
    let report = json_gate(&python, &["scripts/check_shared_pins.py", "--json"]);

    let components = report["components"].as_array().expect("components array");
    assert_eq!(
        components.len(),
        4,
        "the matrix pins four components; this run classified {}",
        components.len()
    );
    for component in components {
        assert_eq!(
            component["verdict"], "compatible",
            "{} is {} ({})",
            component["component"], component["verdict"], component["reason"]
        );
    }
    assert_eq!(report["accepted"], true);
    assert!(report["artifact_mismatches"].as_array().unwrap().is_empty());
    assert!(report["mirror_references"].as_array().unwrap().is_empty());
    assert!(
        report["upstream_pin_mismatches"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the tl-syntax pin disagrees across the files that name it: {}",
        report["upstream_pin_mismatches"]
    );

    // Acceptance is reported and never gated on: the pinned release records
    // `pending_human_acceptance` and ships no predicate for it
    // (agent-ix/engineering-assurance#20). Reading an absent field as approval,
    // in either direction, is the mistake this asserts against.
    assert_eq!(report["acceptance_recorded_here"], false);
    assert!(report["acceptance_state"].is_string());

    // The mirror check must be seen to refuse. Without this it is indistinguishable
    // from a check that matches nothing.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['engineering_assurance']['requirement']+=' --registry=https://npm.ix/';\
             print(json.dumps(m.mirror_references(pins)))",
        ],
    );
    assert_eq!(code, 0, "the mirror probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        !offenders.is_empty(),
        "a mirror registry reference was not detected; the check matches nothing"
    );
}

// Trace: TC-023, FR-006-AC-2, NFR-003-AC-1, SUITE-006, SUITE-007, SUITE-009
#[test]
fn the_chain_reaches_quoin_without_quoin_or_quire_executing_a_producer() {
    let report = chain_report();
    assert_eq!(report["matched"], true, "{report:#}");

    for group in ["scenarios", "controls", "adapter_probes"] {
        let items = report[group]
            .as_array()
            .unwrap_or_else(|| panic!("{group}"));
        assert!(!items.is_empty(), "{group} is empty");
        for item in items {
            assert_eq!(
                item["matched"], true,
                "{group} entry did not match: {item:#}"
            );
        }
    }

    // Every attested result is read out of the producer's bytes. Asserting the
    // values here means a chain that reverted to sealing a literal "passed"
    // would still have to agree with what the producers actually wrote.
    let attested = report["attested_results"]
        .as_object()
        .expect("attested_results");
    assert_eq!(
        attested.len(),
        5,
        "five proof obligations are declared; {} were attested",
        attested.len()
    );
    for (proof, result) in attested {
        assert_eq!(result, "passed", "{proof} was attested {result}");
    }

    // The adapter transcribes one named protocol and refuses another, rather than
    // guessing. A verdict recovered from an unrecognised stream is a verdict
    // recovered from nothing.
    let probes = report["adapter_probes"].as_array().unwrap();
    for required in [
        "refuses-a-foreign-protocol",
        "refuses-an-unnamed-outcome",
        "refuses-an-empty-stream",
        "accepts-the-real-run",
    ] {
        assert!(
            probes.iter().any(|probe| probe["probe"] == required),
            "adapter probe {required} is missing"
        );
    }
}

/// Write an executable shim for each name that records every invocation.
///
/// The log is the point. A shim that is never consulted and a producer that is
/// never run look identical from the outside, so the shims write down every call
/// and the test reads the file rather than assuming.
///
/// A version query is answered rather than refused, and deliberately so. Asking
/// a tool its version is an observation — it is what the compatibility matrix's
/// own `observe` column does — and it is not the thing this test forbids. What
/// is forbidden is asking a tool to build, compile, test, parse, or replay
/// anything. Every such invocation is logged and the log must be empty.
///
/// `--version` is matched anywhere in the argv, not just in `$1`, because the
/// MSRV attestation observes `rustup run 1.75.0 cargo --version`: its declared
/// command runs cargo through the pinned toolchain, so the version sealed into
/// the attestation has to come from that toolchain rather than from ambient
/// cargo. That is still a version observation. Anything without a version flag
/// — `cargo build`, `cargo check`, `rustup run … check` — is logged and fails
/// the test, which is what keeps it able to fail.
fn producer_shims(directory: &Path, names: &[&str]) -> PathBuf {
    fs::create_dir_all(directory).unwrap();
    let log = directory.join("invocations.log");
    let _ = fs::remove_file(&log);
    for name in names {
        let path = directory.join(name);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 for argument in \"$@\"; do\n\
                 case \"$argument\" in\n\
                 --version|-V) echo \"{name} 9.9.9 (shim)\"; exit 0 ;;\n\
                 esac\n\
                 done\n\
                 echo \"$0 $@\" >> {}\n\
                 exit 97\n",
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    log
}

fn run_chain_with_path(shims: &Path) -> std::process::Output {
    let inherited = std::env::var("PATH").unwrap_or_default();
    let revision = head_revision();
    Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(root())
        .env("PATH", format!("{}:{inherited}", shims.display()))
        .output()
        .expect("failed to run the assurance chain")
}

// Trace: TC-023, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_chain_never_executes_a_producer_and_the_probe_can_prove_it() {
    // Two runs, because one proves nothing.
    //
    // Run A replaces every producer — cargo, rustup, rustc — with a stub that
    // logs and fails. The chain must finish, and the log must be empty: not one
    // producer was invoked.
    //
    // Run B is the control. It stubs `quoin`, which the chain is supposed to run,
    // and requires the chain to fail and the log to be non-empty. Without it, an
    // empty log in run A would be equally consistent with PATH never being
    // consulted at all.
    let producers = root().join("target/producer-shims");
    let producer_log = producer_shims(&producers, &["cargo", "rustup", "rustc"]);
    let output = run_chain_with_path(&producers);
    let logged = fs::read_to_string(&producer_log).unwrap_or_default();
    assert!(
        output.status.success(),
        "the assurance chain failed with producers stubbed, which means it ran one:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        logged.trim().is_empty(),
        "the assurance driver asked a producer to do work, not just to name its version:\n{logged}"
    );

    let tools = root().join("target/tool-shims");
    let tool_log = producer_shims(&tools, &["quoin"]);
    let control = run_chain_with_path(&tools);
    let tool_logged = fs::read_to_string(&tool_log).unwrap_or_default();
    assert!(
        !tool_logged.trim().is_empty(),
        "stubbing quoin produced no invocation, so PATH is not being consulted by \
         the subprocess and the run above proves nothing"
    );
    assert!(
        !control.status.success(),
        "the chain succeeded with quoin stubbed out, so it is not actually using it"
    );
}

// Trace: TC-024, FR-006-AC-3, SUITE-003
#[test]
fn the_sealed_records_impact_snapshot_is_the_quire_export() {
    let report = chain_report();
    let export = root().join(report["quire_export"].as_str().expect("quire_export"));
    let bytes =
        fs::read(&export).unwrap_or_else(|error| panic!("{} is absent: {error}", export.display()));

    let digest = {
        let output = Command::new("sha256sum")
            .arg(&export)
            .output()
            .expect("sha256sum failed");
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .expect("sha256sum output")
            .to_owned()
    };
    assert_eq!(
        report["impact_snapshot_digest"], digest,
        "the sealed record's impact snapshot does not name the Quire export it claims"
    );
    // An empty object has a digest too. The snapshot is only worth its content,
    // so the export is required to actually carry the coverage facts the record
    // claims it snapshotted, and to name every requirement this repository has.
    let parsed: Value = serde_json::from_slice(&bytes).expect("the Quire export is JSON");
    let text = String::from_utf8_lossy(&bytes);
    for requirement in [
        "FR-001", "FR-002", "FR-003", "FR-004", "FR-005", "FR-006", "NFR-001", "NFR-002",
        "NFR-003", "StR-001", "StR-002",
    ] {
        assert!(
            text.contains(requirement),
            "the Quire export does not mention {requirement}; it is not a coverage \
             export of this repository"
        );
    }
    assert!(
        parsed.is_object() && !parsed.as_object().unwrap().is_empty(),
        "the Quire export is not a populated document"
    );

    // The measured coverage, pinned. `derive_result` refuses an export that
    // measured nothing or carries a status lie, but a partially-backed export is
    // legitimately not a failure here — four suite rows are deliberately
    // unbacked and SR-007 says why. So the figures themselves are asserted: an
    // export reporting different totals has to move a number in this file.
    let totals = &parsed["totals"];
    // 67 baseline rows + NFR-003-AC-4 + review-identity TC-029 +
    // FR-002-AC-4 + grouping-span TC-030 + NFR-002-AC-2 + TC-031 +
    // NFR-003-AC-5 + TC-032 = 75 total. The same four suite registry rows
    // remain deliberately unbacked, so 71 are backed.
    assert_eq!(totals["total"], 75, "matrix row count changed: {totals}");
    assert_eq!(
        totals["backed"], 71,
        "backed-row count changed: {totals}. Four suite rows are unbacked on \
         purpose; if that number moved, update spec/evidence/suites.md and SR-007 \
         deliberately rather than adjusting this assertion."
    );
    assert!(
        parsed["status_lies"].as_array().unwrap().is_empty(),
        "Quire reported a row whose declared status disagrees with its evidence: {}",
        parsed["status_lies"]
    );

    // And the chain must have read it as such rather than as a not-computed run.
    assert_eq!(
        report["attested_results"]["PROOF-quire-static-export"], "passed",
        "the Quire export was attested as {}",
        report["attested_results"]["PROOF-quire-static-export"]
    );
}

// Trace: TC-026, FR-006-AC-5, NFR-003-AC-3
#[test]
fn all_twelve_verification_outcomes_are_demonstrated_and_paired_with_controls() {
    // The twelve states this migration must keep distinguishable, and the gate
    // that owns each. A state nobody demonstrates is a state nobody would notice
    // the loss of.
    //
    // `malformed` is owned by the parser corpus rather than by an evidence lane,
    // which is where tl-syntax demonstrated it. That is this repository's domain
    // behaviour: six of its seven corpus fixtures are malformed by design.
    const REQUIRED: [(&str, &str); 12] = [
        ("pass", "chain"),
        ("fail", "chain"),
        ("unavailable", "chain"),
        ("unsupported", "chain"),
        ("inconclusive", "chain"),
        ("not-computed", "chain"),
        ("malformed", "chain/parser-corpus"),
        ("partial", "chain"),
        ("stale", "chain"),
        ("suspect", "chain"),
        ("vacuous", "chain"),
        ("tampered", "chain"),
    ];

    let report = chain_report();

    // Only MEASURED outcomes count. The chain's `states_demonstrated` is built
    // from scenarios and adapter probes that ran and matched, never from a
    // free-text label a fixture declares about itself. Counting a label would
    // let a state stop being demonstrated while this test stayed green, which is
    // the exact failure mode this test exists to rule out.
    //
    // All twelve now come from the chain alone. They used to be split with the
    // retained-evidence compatibility lane, which was deleted along with the
    // records it read; the chain already demonstrated every one of the twelve on
    // its own, so nothing moved out of reach.
    let demonstrated: BTreeSet<String> = report["states_demonstrated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();

    let missing: Vec<&str> = REQUIRED
        .iter()
        .filter(|(state, _)| !demonstrated.contains(*state))
        .map(|(state, _)| *state)
        .collect();
    assert!(
        missing.is_empty(),
        "these verification outcomes were never demonstrated: {missing:?}; \
         demonstrated: {demonstrated:?}"
    );

    // Every negative names the positive control that proves the step it refuses
    // is a step that works.
    let controls = report["controls"].as_array().unwrap();
    assert!(!controls.is_empty(), "no positive controls were run");
    let negatives: BTreeSet<&str> = controls
        .iter()
        .map(|control| control["pairs_with"].as_str().unwrap())
        .collect();
    for required in [
        "retained-bytes-changed-after-sealing",
        "refuse-an-edited-receipt",
        "stale-candidate-binding",
        "attested-failed",
        "malformed-input-is-reported-as-malformed",
    ] {
        assert!(
            negatives.contains(required),
            "the negative {required} has no positive control"
        );
    }
}

// Trace: TC-027, FR-006-AC-6, StR-002-VC-2
#[test]
fn malformed_input_is_reported_as_malformed_and_never_as_a_pass() {
    let report = chain_report();

    // The count comes from the corpus manifest, so a producer that stopped
    // reporting malformed rows cannot also move the number it is checked
    // against. Six of the seven fixtures declare a diagnostic code.
    let declared = report["declared_malformed_fixtures"].as_u64().unwrap();
    let reported = report["malformed_rows"].as_u64().unwrap();
    assert_eq!(
        declared, 6,
        "the corpus manifest declares {declared} malformed fixtures; if a fixture was \
         added or removed this expectation should move deliberately"
    );
    assert_eq!(
        reported, declared,
        "the producer reported {reported} malformed rows for {declared} declared \
         malformed fixtures"
    );

    // The three facts the chain asserts, each named, so that dropping any one of
    // them is visible here rather than only inside the driver.
    let scenarios = report["scenarios"].as_array().unwrap();
    for required in [
        "malformed-input-is-reported-as-malformed",
        "malformed-does-not-fail-its-proof",
        "malformed-survives-into-retained-bytes",
    ] {
        let found = scenarios
            .iter()
            .find(|item| item["scenario"] == required)
            .unwrap_or_else(|| panic!("the scenario {required} did not run"));
        assert_eq!(
            found["matched"], true,
            "{required} did not match: {found:#}"
        );
    }

    // Malformed is not a failure: the proof it belongs to is attested `passed`.
    assert_eq!(
        report["attested_results"]["PROOF-parser-conformance"], "passed",
        "a corpus that is malformed by design dragged its proof to a failure"
    );

    // And it is not a silent pass either: the producer's own rows say
    // `malformed`, and the adapter carries that word alongside Quoin's
    // three-valued entry outcome rather than discarding it.
    let python = std::env::var_os("ASSURANCE_PYTHON").is_some();
    let _ = python;
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "scripts/assurance_chain.py",
            "--adapt",
            "target/assurance/parser-conformance.jsonl",
        ],
    );
    assert_eq!(code, 0, "the adapter refused the real stream: {stderr}");
    let adapted: Value = serde_json::from_str(&stdout).expect("the adapter emits JSON");
    let carried = adapted["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["domainOutcome"] == "malformed")
        .count() as u64;
    assert_eq!(
        carried, declared,
        "the adapter dropped the malformed domain outcome; Quoin's entry vocabulary \
         is three-valued, so the twelve-state word has to survive alongside it"
    );
}

// Trace: TC-028, FR-006-AC-7
#[test]
fn no_local_evidence_framework_remains_and_none_of_its_files_came_back() {
    let root = root();

    // The generic machinery is gone, by name.
    for removed in [
        "scripts/build_evidence_envelope.py",
        "scripts/collect_evidence.sh",
        "scripts/finalize_collection.py",
        "scripts/verify_evidence.sh",
        "scripts/verify_evidence_manifest.py",
        "scripts/verify_evidence_history.py",
        "scripts/evidence_profile.py",
        "scripts/check_failure_propagation.py",
        "scripts/check_traceability_coverage.py",
        "scripts/run_local_ci.py",
        "scripts/run_policy_tests.py",
        "scripts/tool_identity.py",
        "scripts/run_cargo_toolchain.py",
        "scripts/validate_json_schema.py",
        "scripts/test_evidence_tool.py",
        "scripts/test_evidence_history.py",
        "scripts/test_collector_behavior.py",
        "scripts/test_failure_propagation.py",
        "scripts/test_json_schema_gate.py",
        "scripts/test_traceability_gate.py",
        "tools.lock",
        "tests/evidence_contract.rs",
        // Deleted under the owner's pre-stable release of the preservation
        // constraint (agent-ix/engineering-assurance#7, agent-ix/tl-parse#13).
        // Named here so a reintroduction is a test failure rather than a quiet
        // return of the machinery this repository decided not to carry.
        "evidence",
        "schemas",
        "scripts/legacy_evidence_view.py",
        "tests/fixtures/legacy-compat",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} is still present; the generic evidence machinery was not removed"
        );
    }

    // The Makefile is orchestration, not a trust root, and carries no gate that
    // polices its own execution.
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    for gone in [
        "check-failure-propagation",
        "ci-for-evidence",
        "verify-evidence",
        "evidence-tool",
        "rust-test-census",
        "compat-view",
    ] {
        assert!(
            !makefile.contains(gone),
            "the Makefile still carries the {gone} self-attestation target"
        );
    }
}

// Trace: TC-026, FR-006-AC-5, NFR-003-AC-3
#[test]
fn a_control_naming_a_scenario_that_does_not_exist_is_refused() {
    // NFR-003-AC-3 claims this guard is checked. It was not: the guard existed
    // and nothing exercised it, which is the same shape of gap the guard itself
    // is there to catch.
    //
    // The driver is copied and one `pairs_with` — and only that one — is
    // renamed. Renaming the scenario as well would leave the pairing consistent
    // and prove nothing, which is exactly how the first version of this probe
    // failed to detect anything.
    let scratch = root().join("target/dangling-probe");
    // A failed probe intentionally retains its scratch for diagnosis. The next
    // run must either remove that exact owned tree or fail with that cause;
    // swallowing the error would turn stale links into a misleading EEXIST.
    match fs::remove_dir_all(&scratch) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("failed to clear the previous dangling-probe scratch: {error}"),
    }
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();

    let control_marker =
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt\",";
    assert!(
        driver.contains(control_marker),
        "the control this probe renames is no longer present in the driver"
    );
    let mutated = driver.replacen(
        control_marker,
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt-typo\",",
        1,
    );
    assert_ne!(mutated, driver, "the mutation did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();

    // Everything else the driver reads comes from the real tree. Every root
    // entry except `scripts` and `target` is symlinked, rather than an enumerated list, so
    // that a driver which starts reading a new directory does not turn this
    // probe into one that fails for an unrelated reason. The scratch owns its
    // Quoin store and shares only the already-produced assurance inputs;
    // symlinking all of `target` coupled this probe to the real store.
    let scratch_target = scratch.join("target");
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "scripts" || name == ".git" || name == "target" {
            continue;
        }
        std::os::unix::fs::symlink(&path, scratch.join(&name))
            .unwrap_or_else(|error| panic!("failed to link {name} into the probe: {error}"));
    }
    // Check ownership before creating anything under target. If `target` drops
    // out of the skip set, this is the first reactor and no self-referential
    // link can be created through the repository target.
    match fs::symlink_metadata(&scratch_target) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Ok(_) => panic!(
            "the dangling-scenario probe must own target/ rather than inherit a root-entry link"
        ),
        Err(error) => panic!("could not establish scratch target ownership: {error}"),
    }
    fs::create_dir_all(&scratch_target).expect("create isolated probe target");
    std::os::unix::fs::symlink(
        root().join("target/assurance"),
        scratch_target.join("assurance"),
    )
    .expect("share assurance inputs with the isolated probe");
    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the mutated chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a control naming a non-existent scenario was not refused\n{stderr}"
    );
    assert!(
        stderr.contains("name a scenario that does not exist"),
        "the refusal did not name the cause: {stderr}"
    );
    let scratch_store = fs::canonicalize(scratch_target.join("assurance-store"))
        .expect("the mutated driver created its isolated Quoin store");
    let real_target =
        fs::canonicalize(root().join("target")).expect("canonical repository target directory");
    let real_store_candidate = real_target.join("assurance-store");
    let real_store = match fs::canonicalize(&real_store_candidate) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => real_store_candidate,
        Err(error) => panic!("canonical repository assurance store: {error}"),
    };
    assert!(
        !scratch_store.starts_with(&real_store),
        "the dangling-scenario probe placed its Quoin store in the real store: {scratch_store:?}"
    );

    // The same isolated environment must succeed once the deliberate dangling
    // reference is removed. This bypassed half proves the expected exit 2 is
    // caused by the validator rather than an earlier scratch-construction fault.
    fs::write(scratch.join("scripts/assurance_chain.py"), &driver).unwrap();
    let bypassed = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the unmutated chain in the isolated scratch");
    assert_eq!(
        bypassed.status.code(),
        Some(0),
        "the isolated scratch is not a valid environment for the unmutated chain:\n{}\n{}",
        String::from_utf8_lossy(&bypassed.stdout),
        String::from_utf8_lossy(&bypassed.stderr)
    );

    // `remove_dir_all` does not follow directory symlinks, but explicitly
    // unlinking every shared input keeps that safety boundary visible and stops
    // a future walk-and-delete replacement reaching repository inputs.
    fs::remove_file(scratch_target.join("assurance")).expect("unlink shared assurance inputs");
    for entry in fs::read_dir(&scratch).expect("read dangling-probe scratch") {
        let path = entry.expect("scratch entry").path();
        if fs::symlink_metadata(&path)
            .expect("scratch entry metadata")
            .file_type()
            .is_symlink()
        {
            fs::remove_file(path).expect("unlink dangling-probe repository input");
        }
    }
    fs::remove_dir_all(&scratch).expect("remove dangling-probe scratch");
}

// Trace: TC-022, FR-006-AC-1
#[test]
fn mirror_mutation_restores_on_unwind_and_a_poisoned_lock_recovers() {
    let mirror = root().join("requirements-assurance.txt");
    let original = fs::read(&mirror).expect("read shared mirror before unwind probe");
    let unwind = std::panic::catch_unwind(|| {
        let _shared_inputs = shared_pin_inputs();
        let _restoration = TrackedFileRestore::new(mirror.clone());
        fs::write(
            &mirror,
            [original.as_slice(), b"\n--registry=https://npm.ix/\n"].concat(),
        )
        .expect("write unwind restoration probe");
        panic!("deliberate unwind after tracked-file mutation");
    });
    assert!(unwind.is_err(), "the restoration probe did not unwind");

    let _recovered = shared_pin_inputs();
    assert_eq!(
        fs::read(&mirror).expect("read shared mirror after unwind probe"),
        original,
        "the RAII guard did not restore requirements-assurance.txt while unwinding"
    );
}

// Trace: TC-022, FR-006-AC-1
#[test]
fn the_mirror_scan_refuses_a_registry_reference_in_a_real_file() {
    // The structural branch of `mirror_references` (pins.json) already has a
    // control. The file-scan branch did not: it was never observed to fire, so
    // it was indistinguishable from a loop over files that never match.
    let _shared_inputs = shared_pin_inputs();
    let python = assurance_python();
    let mirror = root().join("requirements-assurance.txt");
    let mut restoration = TrackedFileRestore::new(mirror.clone());
    fs::write(
        &mirror,
        [restoration.original(), b"\n--registry=https://npm.ix/\n"].concat(),
    )
    .expect("write mirror-scan mutation");
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             found=m.mirror_references(pins);\
             print(json.dumps(found))",
        ],
    );
    restoration.restore();
    assert_eq!(code, 0, "the mirror file-scan probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        offenders
            .iter()
            .any(|entry| entry.starts_with("requirements-assurance.txt:")),
        "a mirror reference written into a scanned FILE was not detected; the \
         file-scan branch matches nothing. Detected: {offenders:?}"
    );

    // And the file must be restored, or this test has dirtied the tree.
    assert_eq!(
        fs::read(&mirror).expect("re-read restored shared mirror"),
        restoration.original(),
        "the probe left requirements-assurance.txt changed"
    );
}

// Trace: TC-032, NFR-003-AC-5
#[test]
fn hosted_ix_flow_identity_and_manual_trigger_are_exact() {
    let workflow = fs::read_to_string(root().join(".github/workflows/ci.yml"))
        .expect("read hosted CI workflow");
    let errors = hosted_workflow_control_errors(&workflow);
    assert!(
        errors.is_empty(),
        "hosted workflow control errors: {errors:?}"
    );

    let comment_only = format!("{workflow}\n# npm i -g ix-flow@99.99.99 is explanatory only\n");
    assert!(
        hosted_workflow_control_errors(&comment_only).is_empty(),
        "a comment-only package spelling became executable"
    );

    let metadata_only = workflow.replacen(
        "name: Install specification tools and modules",
        "name: npm add github:agent-ix/ix-flow is inert metadata",
        1,
    );
    assert!(
        hosted_workflow_control_errors(&metadata_only).is_empty(),
        "an inert step name became an executable package specification"
    );

    let quoted_run_key = workflow.replacen("        run: |", "        'run': |", 1);
    assert!(
        hosted_workflow_control_errors(&quoted_run_key).is_empty(),
        "a quoted YAML run key hid its executable script"
    );

    let spaced_run_key = workflow.replacen("        run: |", "        run : |", 1);
    assert!(
        hosted_workflow_control_errors(&spaced_run_key).is_empty(),
        "semantic YAML spacing around the run-key separator hid its executable script"
    );

    let defaults_metadata = workflow.replacen(
        "\njobs:\n",
        "\ndefaults:\n  run:\n    shell: bash\n\njobs:\n",
        1,
    );
    assert!(
        hosted_workflow_control_errors(&defaults_metadata).is_empty(),
        "defaults.run metadata became an executable step script"
    );

    let word_internal_hash = replace_first_install_invocation(
        &workflow,
        "echo marker#not-a-comment; npm add --global github:agent-ix/ix-flow#v9.9.9; npm install --global",
    );
    let hash_errors = hosted_workflow_control_errors(&word_internal_hash);
    assert!(
        hash_errors
            .iter()
            .any(|error| error.contains("github:agent-ix/ix-flow#v9.9.9")),
        "a word-internal shell hash hid an alternate npm package: {hash_errors:?}"
    );

    let empty_word_hash = replace_first_install_invocation(
        &workflow,
        "true \"\"#not-a-comment && npm add --global github:agent-ix/ix-flow#empty-word; npm install --global",
    );
    let empty_hash_errors = hosted_workflow_control_errors(&empty_word_hash);
    assert!(
        empty_hash_errors
            .iter()
            .any(|error| error.contains("github:agent-ix/ix-flow#empty-word")),
        "an empty quoted shell word reopened the word-internal hash bypass: {empty_hash_errors:?}"
    );

    let nested_shell = replace_first_install_invocation(
        &workflow,
        "bash -c 'npm in --global ix-flow@npm:@agent-ix/ix-flow@9.9.9'; npm install --global",
    );
    let nested_errors = hosted_workflow_control_errors(&nested_shell);
    assert!(
        nested_errors
            .iter()
            .any(|error| error.contains("ix-flow@npm:@agent-ix/ix-flow@9.9.9")),
        "a nested shell invocation hid an alternate npm alias install: {nested_errors:?}"
    );

    let grouped_path_install = replace_first_install_invocation(
        &workflow,
        "( /usr/bin/npm in --global github:agent-ix/ix-flow#grouped ); npm install --global",
    );
    let grouped_errors = hosted_workflow_control_errors(&grouped_path_install);
    assert!(
        grouped_errors
            .iter()
            .any(|error| error.contains("github:agent-ix/ix-flow#grouped")),
        "a grouped path-qualified npm command hid an alternate install: {grouped_errors:?}"
    );

    let redirected_path_install = replace_first_install_invocation(
        &workflow,
        ">/tmp/reviewer-log /usr/bin/npm add --global github:agent-ix/ix-flow#redirected-attached; > /tmp/reviewer-log-2 /usr/bin/npm in --global github:agent-ix/ix-flow#redirected-separate; npm install --global",
    );
    let redirected_errors = hosted_workflow_control_errors(&redirected_path_install);
    assert!(
        redirected_errors.iter().any(|error| error
            .contains("github:agent-ix/ix-flow#redirected-attached")
            && error.contains("github:agent-ix/ix-flow#redirected-separate")),
        "a leading shell redirection hid a path-qualified npm command: {redirected_errors:?}"
    );

    let preceding_shell = replace_first_install_invocation(
        &workflow,
        "bash --version; /usr/bin/npm add --global github:agent-ix/ix-flow#after-shell; npm install --global",
    );
    let preceding_shell_errors = hosted_workflow_control_errors(&preceding_shell);
    assert!(
        preceding_shell_errors
            .iter()
            .any(|error| error.contains("github:agent-ix/ix-flow#after-shell")),
        "a non--c shell invocation suppressed later commands: {preceding_shell_errors:?}"
    );

    let long_shell_option = replace_first_install_invocation(
        &workflow,
        "bash --norc -c 'npm in --global github:agent-ix/ix-flow#nested-long'; npm install --global",
    );
    let long_option_errors = hosted_workflow_control_errors(&long_shell_option);
    assert!(
        long_option_errors
            .iter()
            .any(|error| error.contains("github:agent-ix/ix-flow#nested-long")),
        "a long shell option obscured the actual -c script: {long_option_errors:?}"
    );

    let inert_arguments = replace_first_install_invocation(
        &workflow,
        "printf '%s\\n' npm add github:agent-ix/ix-flow#inert bash -c npm in ix-flow@9.9.9; npm install --global",
    );
    assert!(
        hosted_workflow_control_errors(&inert_arguments).is_empty(),
        "npm- or shell-shaped inert command arguments entered the executable population"
    );

    let shell_comment = replace_first_install_invocation(
        &workflow,
        "# npm add --global github:agent-ix/ix-flow#comment-only\n          npm install --global",
    );
    assert!(
        hosted_workflow_control_errors(&shell_comment).is_empty(),
        "a shell comment inside a literal run block became executable"
    );

    for alias in NPM_INSTALL_ALIASES {
        let alias_workflow =
            replace_first_install_invocation(&workflow, &format!("npm {alias} --global"));
        assert!(
            hosted_workflow_control_errors(&alias_workflow).is_empty(),
            "documented npm install alias {alias:?} changed the package population"
        );
    }

    let unscoped = workflow.replacen("@agent-ix/ix-flow@0.0.4", "ix-flow@0.0.4", 1);
    assert!(
        !hosted_workflow_control_errors(&unscoped).is_empty(),
        "an unscoped package replacement was accepted"
    );
    let alias_duplicate = workflow.replacen(
        "'@agent-ix/ix-flow@0.0.4'",
        "'@agent-ix/ix-flow@0.0.4' 'ix-flow@npm:@agent-ix/ix-flow@0.0.4'",
        1,
    );
    assert!(
        !hosted_workflow_control_errors(&alias_duplicate).is_empty(),
        "an executable npm-alias duplicate was accepted"
    );
    let unversioned_duplicate = workflow.replacen(
        "'@agent-ix/ix-flow@0.0.4'",
        "'@agent-ix/ix-flow@0.0.4' '@agent-ix/ix-flow'",
        1,
    );
    assert!(
        !hosted_workflow_control_errors(&unversioned_duplicate).is_empty(),
        "an executable unversioned duplicate was accepted"
    );
    let automatic = workflow.replacen(
        "  workflow_dispatch:\n",
        "  workflow_dispatch:\n  push:\n",
        1,
    );
    assert!(
        !hosted_workflow_control_errors(&automatic).is_empty(),
        "an automatic push trigger was accepted"
    );
    let inline_automatic = workflow.replacen(
        "  workflow_dispatch:\n",
        "  workflow_dispatch:\n  pull_request: {}\n",
        1,
    );
    assert!(
        !hosted_workflow_control_errors(&inline_automatic).is_empty(),
        "an inline-map pull-request trigger was accepted"
    );

    let (code, stdout, stderr) = run(Path::new("ix-flow"), &["--version"]);
    assert_eq!(code, 0, "ix-flow --version failed: {stderr}");
    assert_eq!(
        stdout.trim(),
        "0.0.4",
        "the released local ix-flow executable is not the pinned version"
    );
}
