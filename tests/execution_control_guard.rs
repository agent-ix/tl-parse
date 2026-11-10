//! The Makefile's own parse-time `MAKEFLAGS` check, run through the real `make`
//! (agent-ix/tl-parse#11). `help` has no prerequisites, so a run that gets past
//! the guard exits at once. MAKEFLAGS is stripped from the child's environment
//! so an outer `make` invocation cannot leak its flags into the one under test.
//! `help` is the one goal the parse-time guard does not run, so no nested cargo build
//! is started from inside `cargo test`.

use std::process::Command;

fn make(flags: &[&str]) -> (bool, String) {
    let output = Command::new("make")
        .args(["-C", env!("CARGO_MANIFEST_DIR")])
        .args(flags)
        .arg("help")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

// FND-001: a bundled -i used to bypass the exact-word MAKEFLAGS check.
#[test]
fn bundled_and_jobserver_separated_ignore_errors_are_caught() {
    for flags in [&["-ik"][..], &["-j", "-i"][..]] {
        let (ok, stderr) = make(flags);
        assert!(
            !ok && stderr.contains("ignore-errors"),
            "{flags:?}: {stderr}"
        );
    }
}

#[test]
fn flags_without_ignore_errors_are_not_blocked() {
    for flags in [&["-k"][..], &["-j4"][..], &[][..]] {
        let (ok, stderr) = make(flags);
        assert!(ok, "{flags:?}: {stderr}");
    }
}

fn guard(makefile: Option<&str>) -> (Option<i32>, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_check_make_execution_control"));
    if let Some(text) = makefile {
        let dir =
            std::env::temp_dir().join(format!("guard-bin-{}-{}", std::process::id(), text.len()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Makefile");
        std::fs::write(&path, text).unwrap();
        command.arg(path);
    }
    let output = command.output().unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn the_binary_exits_one_with_the_finding_on_a_neutered_makefile() {
    let (code, stderr) = guard(Some("all:\n\t-false\n"));
    assert_eq!(code, Some(1));
    assert!(stderr.contains(":2: recipe line uses the '-'"), "{stderr}");
}

#[test]
fn the_binary_exits_zero_on_a_clean_makefile_and_two_without_arguments() {
    assert_eq!(guard(Some("all:\n\ttrue\n")).0, Some(0));
    assert_eq!(guard(None).0, Some(2));
}
