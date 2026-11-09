//! TL-371: behavior of `scripts/mutation_gate.py` on synthetic cargo-mutants outcomes. The
//! script is a stopgap until its port to Rust (TL-370); these tests keep it honest meanwhile.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The gate's exit status and standard output for one synthetic run.
struct Run {
    status: i32,
    stdout: String,
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tl-mutation-gate-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn outcome(summary: &str, name: &str) -> String {
    format!(
        r#"{{"scenario":{{"Mutant":{{"file":"src/lib.rs","name":"src/lib.rs:1:1: {name}"}}}},"summary":"{summary}"}}"#
    )
}

fn write_outcomes(dir: &Path, outcomes: &[String]) {
    let baseline = r#"{"scenario":"Baseline","summary":"Success"}"#.to_owned();
    let all = std::iter::once(baseline)
        .chain(outcomes.iter().cloned())
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        dir.join("outcomes.json"),
        format!(r#"{{"outcomes":[{all}]}}"#),
    )
    .unwrap();
}

fn thresholds(dir: &Path, floor: f64, survivors: &str) -> PathBuf {
    let path = dir.join("thresholds.toml");
    fs::write(
        &path,
        format!(
            "diff_min_kill_percent = 100.0\n[[group]]\nname = \"crate\"\nglobs = [\"src/*.rs\"]\n\
             min_kill_percent = {floor}\n{survivors}"
        ),
    )
    .unwrap();
    path
}

fn gate(diff: bool, dir: &Path, thresholds: &Path) -> Run {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/mutation_gate.py");
    let mut command = Command::new("python3");
    command.arg(script);
    if diff {
        command.arg("--diff");
    }
    let output = command.arg(dir).arg(thresholds).output().unwrap();
    Run {
        status: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
    }
}

const LISTED_MISS: &str = "[[survivor]]\nfile = \"src/lib.rs\"\nreason = \"equivalent\"\nnames = [\"replace a with b in f\"]\n";

#[test]
fn an_unlisted_missed_mutant_fails_the_full_and_the_diff_gate() {
    let dir = scratch("unlisted");
    write_outcomes(
        &dir,
        &[
            outcome("CaughtMutant", "x"),
            outcome("MissedMutant", "replace a with b in f"),
        ],
    );
    let toml = thresholds(&dir, 0.0, "");
    for diff in [false, true] {
        let run = gate(diff, &dir, &toml);
        assert_eq!(run.status, 1, "diff={diff}");
        assert!(
            run.stdout.contains("survivor without a reason"),
            "{}",
            run.stdout
        );
    }
}

#[test]
fn a_listed_survivor_passes_and_one_with_an_empty_reason_fails() {
    let dir = scratch("listed");
    write_outcomes(
        &dir,
        &[
            outcome("CaughtMutant", "x"),
            outcome("MissedMutant", "replace a with b in f"),
        ],
    );
    let listed = thresholds(&dir, 50.0, LISTED_MISS);
    assert_eq!(gate(false, &dir, &listed).status, 0);
    let blank = thresholds(&dir, 50.0, &LISTED_MISS.replace("equivalent", " "));
    assert_eq!(gate(false, &dir, &blank).status, 1);
}

#[test]
fn a_listed_timeout_counts_against_the_kill_rate() {
    let dir = scratch("timeout");
    write_outcomes(
        &dir,
        &[
            outcome("CaughtMutant", "x"),
            outcome("Timeout", "replace a with b in f"),
        ],
    );
    let met = thresholds(&dir, 50.0, LISTED_MISS);
    assert_eq!(gate(false, &dir, &met).status, 0);
    let above = thresholds(&dir, 60.0, LISTED_MISS);
    let run = gate(false, &dir, &above);
    assert_eq!(run.status, 1);
    assert!(run.stdout.contains("timeout=1"), "{}", run.stdout);
}

#[test]
fn an_empty_full_population_fails_but_an_empty_diff_passes() {
    let dir = scratch("empty");
    write_outcomes(&dir, &[outcome("Unviable", "x")]);
    let toml = thresholds(&dir, 0.0, "");
    assert_eq!(gate(false, &dir, &toml).status, 1);
    assert_eq!(gate(true, &dir, &toml).status, 0);
}

#[test]
fn missing_outcomes_are_an_input_error_in_a_full_run_and_an_empty_diff_otherwise() {
    let dir = scratch("missing");
    let toml = thresholds(&dir, 0.0, "");
    assert_eq!(gate(false, &dir, &toml).status, 2);
    assert_eq!(gate(true, &dir, &toml).status, 0);
}
