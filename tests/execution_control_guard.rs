//! The Makefile's own parse-time `MAKEFLAGS` check, run through the real `make`
//! (agent-ix/tl-parse#11). `help` has no prerequisites, so a run that gets past
//! the guard exits at once. MAKEFLAGS is stripped from the child's environment
//! so an outer `make` invocation cannot leak its flags into the one under test.
//! The guard binary is passed in prebuilt: the Makefile would otherwise run
//! `cargo run` from inside `cargo test` and wait on its own build lock.

use std::process::Command;

fn make(flags: &[&str]) -> (bool, String) {
    let output = Command::new("make")
        .args(["-C", env!("CARGO_MANIFEST_DIR")])
        .args(flags)
        .arg("help")
        .env_remove("MAKEFLAGS")
        .env_remove("MFLAGS")
        .env(
            "EXECUTION_CONTROL_GUARD_CMD",
            env!("CARGO_BIN_EXE_check_make_execution_control"),
        )
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
