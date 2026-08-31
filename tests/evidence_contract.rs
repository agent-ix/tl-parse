use std::{fs, os::unix::fs::PermissionsExt, process::Command};

use serde_json::Value;

fn root_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

fn observed_cargo_arguments(target: &str) -> Vec<String> {
    let directory = tempfile::tempdir().unwrap();
    let probe = directory.path().join("cargo-probe");
    let log = directory.path().join("cargo.log");
    fs::write(
        &probe,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$PROBE_LOG\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&probe).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&probe, permissions).unwrap();
    let output = Command::new("make")
        .arg(target)
        .arg(format!("CARGO={}", probe.display()))
        .env("PROBE_LOG", &log)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "make {target} probe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(log)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect()
}

// Trace: TC-022, FR-005-AC-4, NFR-002-AC-2, SUITE-001, SUITE-002, SUITE-003
// Trace: SUITE-004, SUITE-005, SUITE-006
// Trace: SUITE-007, SUITE-008, SUITE-009
#[test]
fn evidence_gates_and_manual_ci_boundary_are_machine_checkable() {
    let makefile = fs::read_to_string(root_path("Makefile")).unwrap();
    let ci_line = makefile
        .lines()
        .find(|line| line.starts_with("ci:"))
        .expect("Makefile has a composite local gate");
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
            ci_line.split_whitespace().any(|item| item == gate),
            "local gate omits {gate}"
        );
    }
    let dry_run = Command::new("make")
        .args(["-n", "ci"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(dry_run.status.success());
    let dry_run = String::from_utf8(dry_run.stdout).unwrap();
    for command in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test --all-targets --all-features",
        "sha256sum --check SHA256SUMS",
        "cargo +nightly fuzz build parser",
        "bash scripts/run_fuzz_smoke.sh",
        "cargo deny check licenses",
        "cargo deny check sources",
        "cargo deny check advisories",
        "cargo deny check bans",
        "bash scripts/check_unsafe_comments.sh",
        "python3 scripts/test_evidence_tool.py",
        "python3 scripts/test_failure_propagation.py",
        "python3 scripts/test_json_schema_gate.py",
        "quire validate --scope . 'spec/**/*.md' 'docs/*.md'",
        "python3 scripts/check_traceability_coverage.py",
        "cargo +1.75.0 check --all-targets --all-features",
        "RUSTDOCFLAGS=-Dwarnings",
        "doc --no-deps --all-features",
        "bash scripts/verify_evidence.sh",
    ] {
        assert!(
            dry_run.contains(command),
            "local gate omits exact command {command}"
        );
    }

    assert_eq!(
        observed_cargo_arguments("test"),
        ["test --all-targets --all-features"]
    );
    assert_eq!(
        observed_cargo_arguments("deny"),
        [
            "deny check advisories",
            "deny check bans",
            "deny check licenses",
            "deny check sources"
        ]
    );
    let sentinel = Command::new("make")
        .args(["--no-print-directory", "check-failure-propagation"])
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
    let verifier = fs::read_to_string(root_path("scripts/verify_evidence.sh")).unwrap();
    for required in [
        "sha256sum --check evidence/ANCHORS",
        "assurance argument and evidence anchors disagree",
        "retained evidence summary is missing",
    ] {
        assert!(verifier.contains(required));
    }

    let behavior = Command::new("python3")
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
