use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};

use tempfile::NamedTempFile;
use tl_parse::{
    attribution_document_digest, dialect_digest, dialect_document_digest, TL_SYNTAX_REVISION,
};

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
        // The compiled revision, which moves when the crate repins.
        TL_SYNTAX_REVISION,
        // The authorship basis, which is historical and must not be silently
        // rewritten to match the compiled revision when the two diverge.
        "740182f13b84858008d6f176f75136737d405c1b",
        "tl-parse.clean-ascii/v1",
    ] {
        assert!(dialect.contains(required), "dialect omits {required}");
    }
    assert_eq!(
        dialect_digest(),
        "22959d4df6c7a1230172289903f1c31f36859b6f2a0e4556e886bdb7ebc9ae11"
    );
    assert_eq!(
        dialect_document_digest(),
        "ae891b5c7e0784e90d2a5f869d8de26ae824cc3b9138792295bb836915ab932d"
    );
    assert_eq!(
        attribution_document_digest(),
        "f3461df5bda9ee96ec4f132229845f2f1be6cc4a6050091696e8eb58ebecd728"
    );
    let attribution = fs::read_to_string(format!("{root}/docs/ATTRIBUTION.md")).unwrap();
    // The authorship basis at 740182f1, which is historical and does not move,
    // and the compiled revision at 953ee825, which is a different fact. The
    // per-file digest tables that used to be asserted here were dropped under
    // issue #15: 740182f1 is on a deleted branch, so half of them could never be
    // re-derived by anyone, and Cargo.lock is what enforces the compiled pin.
    for required in [
        "independently authored",
        "MIT OR Apache-2.0",
        TL_SYNTAX_REVISION,
        "740182f13b84858008d6f176f75136737d405c1b",
    ] {
        assert!(
            attribution.contains(required),
            "attribution omits {required}"
        );
    }

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
    assert_eq!(output.stdout, b"p0U[1,2]true\n");

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

    let format_json = stdin_run(&["format", "--json", "-"], "p0 & p1");
    assert!(format_json.status.success());
    let value: serde_json::Value = serde_json::from_slice(&format_json.stdout).unwrap();
    assert_eq!(value["text"], "p0&p1");

    let unknown = binary().args(["validate", "--bogus"]).output().unwrap();
    assert_eq!(unknown.status.code(), Some(2));
    assert!(String::from_utf8(unknown.stderr)
        .unwrap()
        .contains("unknown option"));

    let duplicate = binary()
        .args(["validate", "first", "second"])
        .output()
        .unwrap();
    assert_eq!(duplicate.status.code(), Some(2));
    assert!(String::from_utf8(duplicate.stderr)
        .unwrap()
        .contains("only one input path"));

    let truncated = stdin_run(&["validate", "-"], &"@".repeat(100));
    assert_eq!(truncated.status.code(), Some(1));
    assert!(String::from_utf8(truncated.stderr)
        .unwrap()
        .contains("error[diagnostic_limit]"));

    let mut oversized = NamedTempFile::new().unwrap();
    oversized.write_all(&vec![b' '; 1_048_576]).unwrap();
    oversized.write_all("λ".as_bytes()).unwrap();
    let source_limit = binary()
        .args(["validate", oversized.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(source_limit.status.code(), Some(1));
    assert!(String::from_utf8(source_limit.stderr)
        .unwrap()
        .contains("source_limit"));

    let source_limit_json = binary()
        .args(["validate", "--json", oversized.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(source_limit_json.status.code(), Some(1));
    let value: serde_json::Value = serde_json::from_slice(&source_limit_json.stdout).unwrap();
    assert_eq!(value["stats"]["source_bytes"], 1_048_578);
    assert_eq!(value["diagnostics"][0]["code"], "source_limit");
    assert_eq!(value["diagnostics"][0]["span"]["start"], 1_048_576);
    assert_eq!(value["diagnostics"][0]["span"]["end"], 1_048_577);

    let source = "p0";
    let mut child = binary()
        .args(["format", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "broken stdout pipe caused a panic");
}

// Trace: TC-021, FR-005-AC-3, NFR-001-AC-1
#[test]
fn cli_dispatch_read_and_render_failures_are_classified() {
    for (arguments, expected) in [
        (vec![], "usage: tl-parse"),
        (
            vec!["validate", "--profile"],
            "--profile requires closed or online",
        ),
        (
            vec!["validate", "--profile", "future"],
            "--profile requires closed or online",
        ),
    ] {
        let output = binary().args(arguments).output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
    }

    let default_stdin = stdin_run(&["validate"], "p0");
    assert!(default_stdin.status.success());
    assert_eq!(default_stdin.stdout, b"valid\n");

    let closed = stdin_run(&["validate", "--profile", "closed", "--json"], "p0");
    assert!(closed.status.success());
    let value: serde_json::Value = serde_json::from_slice(&closed.stdout).unwrap();
    assert_eq!(value["semantic_profile"], "mltl.closed-trace/v1");

    let duplicate_stdin = binary().args(["validate", "-", "-"]).output().unwrap();
    assert_eq!(duplicate_stdin.status.code(), Some(2));
    assert!(String::from_utf8(duplicate_stdin.stderr)
        .unwrap()
        .contains("only one input path"));

    let missing = binary()
        .args(["validate", "/definitely/not/a/tl-parse-input"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8(missing.stderr)
        .unwrap()
        .contains("cannot open"));

    let mut invalid_utf8 = NamedTempFile::new().unwrap();
    invalid_utf8.write_all(&[0xff, 0xfe]).unwrap();
    let unreadable = binary()
        .args(["validate", invalid_utf8.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(unreadable.status.code(), Some(2));
    assert!(String::from_utf8(unreadable.stderr)
        .unwrap()
        .contains("invalid utf-8 sequence"));

    let mut child = binary()
        .arg("validate")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&[0xff, 0xfe])
        .unwrap();
    let invalid_stdin = child.wait_with_output().unwrap();
    assert_eq!(invalid_stdin.status.code(), Some(2));
    assert!(String::from_utf8(invalid_stdin.stderr)
        .unwrap()
        .contains("cannot read stdin"));

    let streamed_limit = stdin_run(&["validate"], &" ".repeat(1_048_577));
    assert_eq!(streamed_limit.status.code(), Some(1));
    let stderr = String::from_utf8(streamed_limit.stderr).unwrap();
    assert!(stderr.contains("error[source_limit]"));
    assert!(stderr.contains("source length is at least 1048577"));

    let rendered = stdin_run(&["format", "-"], "F[3,1]p0");
    assert_eq!(rendered.status.code(), Some(1));
    assert!(rendered.stdout.is_empty());
    let stderr = String::from_utf8(rendered.stderr).unwrap();
    assert!(stderr.starts_with("error[invalid_interval] 1..6: "));
    assert!(stderr.contains("interval"));
}
