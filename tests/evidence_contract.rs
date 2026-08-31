use std::{fs, process::Command};

use serde_json::Value;

fn root_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

// Trace: TC-022, TC-023, TC-024, TC-025, FR-005-AC-4, NFR-002-AC-2
// Trace: NFR-003-AC-1, NFR-003-AC-2, NFR-003-AC-3, NFR-003-AC-4, SUITE-001, SUITE-002, SUITE-003
// Trace: SUITE-004, SUITE-005, SUITE-006
// Trace: SUITE-007, SUITE-008, SUITE-009
#[test]
fn evidence_gates_and_manual_ci_boundary_are_machine_checkable() {
    let makefile = fs::read_to_string(root_path("Makefile")).unwrap();
    let runner = fs::read_to_string(root_path("scripts/run_local_ci.py")).unwrap();
    for gate in [
        "check-failure-propagation",
        "fmt-check",
        "lint",
        "test",
        "check-corpus",
        "fuzz-build",
        "fuzz-smoke",
        "deny",
        "audit-unsafe",
        "evidence-tool",
        "spec",
        "msrv",
        "rustdoc",
        "verify-evidence",
    ] {
        assert!(
            runner.contains("propagation.PROBES"),
            "local runner omits the gate census"
        );
        assert!(
            makefile.contains(&format!("{gate}:")),
            "Makefile omits {gate}"
        );
    }
    assert!(runner.contains("positive_ci_census"));
    assert!(makefile.contains("scripts/run_local_ci.py --include-verify"));
    assert!(makefile.contains("ci-for-evidence:\n\t/usr/bin/python3 scripts/run_local_ci.py"));
    for command in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test --all-targets --all-features",
        "python3 scripts/check_checksum_manifest.py corpus/v1",
        "python3 scripts/check_checksum_manifest.py fuzz/corpus/parser",
        "cargo +nightly fuzz build parser",
        "bash scripts/run_fuzz_smoke.sh",
        "cargo deny check licenses",
        "cargo deny check sources",
        "cargo deny check advisories",
        "cargo deny check bans",
        "bash scripts/check_unsafe_comments.sh",
        "python3 scripts/run_policy_tests.py",
        "quire validate --scope . 'spec/**/*.md' 'docs/*.md'",
        "python3 scripts/check_traceability_coverage.py",
        "cargo +1.75.0 check --all-targets --all-features",
        "RUSTDOCFLAGS=-Dwarnings",
        "doc --no-deps --all-features",
        "bash scripts/verify_evidence.sh",
    ] {
        assert!(
            makefile.contains(command),
            "local gate omits exact command {command}"
        );
    }

    let sentinel = Command::new("make")
        .args(["--no-print-directory", "MAKEFLAGS=", "check-failure-propagation"])
        .env_remove("MAKEFLAGS")
        .env_remove("MAKELEVEL")
        .env_remove("MFLAGS")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        sentinel.status.success(),
        "failure-propagation sentinel failed: {}",
        String::from_utf8_lossy(&sentinel.stderr)
    );

    let workflow = fs::read_to_string(root_path(".github/workflows/ci.yml")).unwrap();
    let trigger = workflow
        .split_once("on:\n")
        .and_then(|(_, rest)| rest.split_once("\njobs:"))
        .map(|(value, _)| value)
        .expect("workflow has an explicit trigger block");
    assert!(trigger.contains("workflow_dispatch:"));
    assert!(!trigger.contains("push:"));
    assert!(!trigger.contains("pull_request:"));

    for schema in [
        "schemas/tl-parse-evidence-input-v1.schema.json",
        "schemas/tl-parse-evidence-manifest-v1.schema.json",
    ] {
        let value: Value =
            serde_json::from_str(&fs::read_to_string(root_path(schema)).unwrap()).unwrap();
        assert_eq!(value["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(value["additionalProperties"], false);
    }
    assert!(fs::metadata(root_path("scripts/verify_evidence_manifest.py")).is_ok());
    let collector = fs::read_to_string(root_path("scripts/collect_evidence.sh")).unwrap();
    assert!(collector.contains("git diff --check"));
    assert!(collector.contains("':(exclude)evidence/**'"));
    assert!(collector.contains("clean_env=(/usr/bin/env -i PATH="));
    assert!(collector.contains("for tool in bash cargo git make python3 quire rustc sha256sum"));
    assert!(collector.contains("scripts/tool_identity.py --verify-live"));
    assert!(collector.contains("pgm01_validator_digest="));
    assert!(collector.contains("make ci-for-evidence"));
    let verifier = fs::read_to_string(root_path("scripts/verify_evidence.sh")).unwrap();
    for required in [
        "sha256sum --check evidence/ANCHORS",
        "assurance argument and evidence anchors disagree",
        "retained evidence summary is missing",
    ] {
        assert!(verifier.contains(required));
    }

    let behavior = Command::new("/usr/bin/python3")
        .arg(root_path("scripts/test_evidence_tool.py"))
        .output()
        .unwrap();
    assert!(
        behavior.status.success(),
        "evidence behavior test failed: {}",
        String::from_utf8_lossy(&behavior.stderr)
    );

    let assurance = fs::read_to_string(root_path("spec/assurance/AA-001.md")).unwrap();
    let assurance_words = assurance.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(assurance.contains("status: open"));
    assert!(assurance_words.contains("only the named human release owner"));
}
