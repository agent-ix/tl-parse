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
        TL_SYNTAX_REVISION,
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
        "2004663070c53bda7aa1bf54f0057d83835c3afdbccb91de7ddb312e1e6a6f24"
    );
    assert_eq!(
        attribution_document_digest(),
        "58abff30ae6fa05528159f2d9aaf2ef8ee5dac4146d7ba0080ac288a20e36d8e"
    );
    let attribution = fs::read_to_string(format!("{root}/docs/ATTRIBUTION.md")).unwrap();
    for digest in [
        "04e6a46e697444df8e6764dd0e5e5227b1271199ffc0e9d24f77720c979eb14e",
        "f97005479f1f12511f1fceb2f9a85b94b482170e606c5735758e11aa2e4580f2",
        "97ead12ddb151fc37ffb1c623ab42b9814e21629dee252ff23dc7205f1df9f05",
        "62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a",
    ] {
        assert!(attribution.contains(digest));
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
}
