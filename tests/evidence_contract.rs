use std::{fs, process::Command};

use serde_json::Value;

fn root_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

// Trace: TC-022, FR-005-AC-4, NFR-002-AC-2
#[test]
fn evidence_gates_and_manual_ci_boundary_are_machine_checkable() {
    let makefile = fs::read_to_string(root_path("Makefile")).unwrap();
    let ci_line = makefile
        .lines()
        .find(|line| line.starts_with("ci:"))
        .expect("Makefile has a composite local gate");
    for gate in [
        "fmt-check",
        "lint",
        "test",
        "check-corpus",
        "deny",
        "audit-unsafe",
        "evidence-tool",
        "spec",
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
        "cargo deny check licenses",
        "cargo deny check sources",
        "bash scripts/check_unsafe_comments.sh",
        "python3 scripts/test_evidence_tool.py",
        "quire validate --scope . 'spec/**/*.md' 'docs/*.md'",
        "quire coverage --scope . --strict",
        "RUSTDOCFLAGS=-Dwarnings",
        "doc --no-deps --all-features",
        "bash scripts/verify_evidence.sh",
    ] {
        assert!(
            dry_run.contains(command),
            "local gate omits exact command {command}"
        );
    }

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
