use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;
use tl_parse::{dialect_digest, TL_SYNTAX_REVISION};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tl-parse"))
}

fn stdin_run(arguments: &[&str], source: &str) -> std::process::Output {
    let mut child = binary()
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

// Trace: TC-020, FR-001-AC-3, FR-005-AC-3, NFR-002-AC-1, StR-001-VC-1
#[test]
fn dialect_provenance_and_cli_valid_paths_are_exact() {
    let root = env!("CARGO_MANIFEST_DIR");
    let dialect =
        fs::read_to_string(format!("{root}/docs/DIALECT-001-clean-room-mltl-v1.md")).unwrap();
    for required in [
        "independently authored",
        "MIT OR Apache-2.0",
        TL_SYNTAX_REVISION,
        "tl-parse.clean-ascii/v1",
    ] {
        assert!(dialect.contains(required), "dialect omits {required}");
    }
    assert_eq!(
        dialect_digest(),
        "22959d4df6c7a1230172289903f1c31f36859b6f2a0e4556e886bdb7ebc9ae11"
    );

    let mut file = NamedTempFile::new().unwrap();
    write!(file, "G[0,1]p0").unwrap();
    let output = binary()
        .args(["validate", file.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"valid\n");
    assert!(output.stderr.is_empty());

    let output = stdin_run(&["format", "--profile", "online", "-"], "p0 U[1,2] true");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"(p0U[1,2]true)\n");

    let output = stdin_run(&["validate", "--profile", "online", "--json", "-"], "p4");
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["semantic_profile"], "mltl.online-prefix/v1");
    assert!(value["document"].is_object());
}

// Trace: TC-021, FR-005-AC-3, NFR-001-AC-1
#[test]
fn cli_invalid_usage_and_repeated_outputs_have_stable_exit_classes() {
    let run_invalid = || stdin_run(&["validate", "--json", "-"], "F[3,1]p0");
    let first = run_invalid();
    let second = run_invalid();
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["diagnostics"][0]["code"], "invalid_interval");

    let usage = binary().arg("unknown").output().unwrap();
    assert_eq!(usage.status.code(), Some(2));
    assert!(String::from_utf8(usage.stderr)
        .unwrap()
        .contains("usage: tl-parse"));
}
