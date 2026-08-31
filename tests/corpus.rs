use std::{fs, path::PathBuf, process::Command};

use serde::Deserialize;
use tl_parse::{format_document, parse, report_json, DiagnosticCode, FormatLimits, ParseLimits};
use tl_syntax::SemanticProfile;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: String,
    revision: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    path: String,
    profile: String,
    max_tokens: Option<usize>,
    max_depth: Option<usize>,
    expected_code: Option<String>,
    expected_canonical: Option<String>,
}

fn code_name(code: DiagnosticCode) -> &'static str {
    code.as_str()
}

// Trace: TC-018, FR-005-AC-1, StR-002-VC-2
#[test]
fn malformed_resource_corpus_is_checksummed_and_matches_its_manifest() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = root.join("corpus/v1");
    let checksum = Command::new("sha256sum")
        .args(["--check", "SHA256SUMS"])
        .current_dir(&corpus)
        .output()
        .unwrap();
    assert!(
        checksum.status.success(),
        "{}",
        String::from_utf8_lossy(&checksum.stderr)
    );
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(corpus.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.schema_version, "tl-parse.corpus/v1");
    assert_eq!(manifest.revision, tl_parse::CORPUS_REVISION);
    assert_eq!(manifest.cases.len(), 7);

    for case in manifest.cases {
        let source = fs::read_to_string(corpus.join(&case.path)).unwrap();
        let profile = match case.profile.as_str() {
            "closed" => SemanticProfile::ClosedTraceV1,
            "online" => SemanticProfile::OnlinePrefixV1,
            other => panic!("unknown profile {other}"),
        };
        let limits = ParseLimits {
            max_tokens: case.max_tokens.unwrap_or(ParseLimits::default().max_tokens),
            max_depth: case.max_depth.unwrap_or(ParseLimits::default().max_depth),
            ..ParseLimits::default()
        };
        let report = parse(&source, profile, limits);
        match (case.expected_code, case.expected_canonical) {
            (Some(expected), None) => {
                assert!(report.document.is_none(), "{}", case.path);
                assert!(
                    report
                        .diagnostics
                        .iter()
                        .any(|item| code_name(item.code) == expected),
                    "{}: {:?}",
                    case.path,
                    report.diagnostics
                );
            }
            (None, Some(expected)) => {
                let formatted = format_document(
                    report.document.as_ref().expect("valid corpus case"),
                    FormatLimits::default(),
                );
                assert_eq!(
                    formatted.text.as_deref(),
                    Some(expected.as_str()),
                    "{}",
                    case.path
                );
            }
            _ => panic!("case {} has an invalid expected outcome", case.path),
        }
    }
}

// Trace: TC-019, FR-005-AC-2, StR-002-VC-2
#[test]
fn every_checked_fuzz_seed_is_bounded_and_successes_round_trip() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let seeds = root.join("fuzz/corpus/parser");
    let checksum = Command::new("sha256sum")
        .args(["--check", "SHA256SUMS"])
        .current_dir(&seeds)
        .output()
        .unwrap();
    assert!(checksum.status.success());
    assert!(root.join("fuzz/fuzz_targets/parser.rs").is_file());

    let limits = ParseLimits {
        max_source_bytes: 4_096,
        max_tokens: 512,
        max_nodes: 256,
        max_depth: 64,
        max_diagnostics: 16,
        max_work: 16_384,
    };
    let mut paths = fs::read_dir(&seeds)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("txt"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 4);
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let report = parse(&source, SemanticProfile::ClosedTraceV1, limits);
        assert!(report.stats.work <= limits.max_work);
        report_json(&report).unwrap();
        if let Some(document) = report.document.as_ref() {
            let text = format_document(document, FormatLimits::default())
                .text
                .unwrap();
            let reparsed = parse(&text, SemanticProfile::ClosedTraceV1, limits);
            assert!(reparsed.document.is_some(), "{}", path.display());
        }
    }
}
