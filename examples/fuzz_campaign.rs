// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (C) 2026 Peter Krenesky

//! Bounded fuzz smoke runner (FR-005-AC-2).
//!
//! Stages the checked-in seeds for one fuzz target, runs a short cargo-fuzz
//! campaign under a wall-clock deadline with LeakSanitizer enabled, and exits
//! nonzero when the fuzzer fails, produces a crash artifact, or cannot run.

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use command_group::{CommandGroup, GroupChild};

const RUNS: u32 = 64;
const DEADLINE: Duration = Duration::from_secs(300);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Target {
    Parser,
    CleanAsciiV2,
}

impl Target {
    fn parse(value: &OsStr) -> Option<Self> {
        match value.to_str() {
            Some("parser") => Some(Self::Parser),
            Some("clean_ascii_v2") => Some(Self::CleanAsciiV2),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Parser => "parser",
            Self::CleanAsciiV2 => "clean_ascii_v2",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProcessState {
    Exited(i32),
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Outcome {
    Pass,
    Fail,
    Unavailable,
}

impl Outcome {
    const fn exit_code(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Fail => 1,
            Self::Unavailable => 2,
        }
    }
}

/// A clean exit that left crash artifacts behind is not a pass.
fn classify(process: ProcessState, artifacts_present: bool) -> Outcome {
    match process {
        ProcessState::TimedOut => Outcome::Unavailable,
        ProcessState::Exited(0) if !artifacts_present => Outcome::Pass,
        ProcessState::Exited(_) => Outcome::Fail,
    }
}

fn ambient_sanitizer_override(
    asan: Option<&OsStr>,
    disable_leaks: Option<&OsStr>,
    lsan: Option<&OsStr>,
) -> bool {
    asan.is_some() || disable_leaks.is_some() || lsan.is_some()
}

fn leak_sanitizer_available() -> io::Result<bool> {
    if !cfg!(target_os = "linux") {
        return Ok(true);
    }
    let status = match fs::read_to_string("/proc/self/status") {
        Ok(status) => status,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(error),
    };
    let no_new_privileges = status
        .lines()
        .any(|line| line.split_whitespace().eq(["NoNewPrivs:", "1"]));
    let filtered_seccomp = status
        .lines()
        .any(|line| line.split_whitespace().eq(["Seccomp:", "2"]));
    Ok(!(no_new_privileges && filtered_seccomp))
}

fn target_directory(root: &Path) -> PathBuf {
    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"))
        .join("fuzz")
}

fn copy_seeds(source: &Path, destination: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let path = entry?.path();
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(io::Error::other(format!(
                "seed {} is not a regular file",
                path.display()
            )));
        }
        if let Some(name) = path.file_name() {
            fs::copy(&path, destination.join(name))?;
        }
    }
    Ok(())
}

fn supervise(mut child: GroupChild, deadline: Duration) -> io::Result<ProcessState> {
    let started = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => return Ok(ProcessState::Exited(status.code().unwrap_or(-1))),
            None if started.elapsed() < deadline => thread::sleep(Duration::from_millis(25)),
            None => {
                child.kill()?;
                child.wait()?;
                return Ok(ProcessState::TimedOut);
            }
        }
    }
}

fn run_fuzzer(
    root: &Path,
    target: Target,
    generated: &Path,
    seeds: &Path,
    artifacts: &Path,
) -> io::Result<ProcessState> {
    // TL-195: LeakSanitizer deterministically flags a 56-byte allocation made
    // by libFuzzer's own driver thread, not by tl-parse or either fuzz target
    // (see fuzz/lsan_suppressions.txt for the full stack and rationale).
    // Suppressing only that named allocation site keeps
    // ASAN_OPTIONS=detect_leaks=1 catching every real leak in the code under
    // test.
    let lsan_suppressions = root.join("fuzz").join("lsan_suppressions.txt");
    let child = Command::new("rustup")
        .args(["run", "nightly", "cargo", "fuzz", "run", target.as_str()])
        .arg("--target-dir")
        .arg(target_directory(root))
        .arg(generated)
        .arg(seeds)
        .arg("--")
        .arg(format!("-artifact_prefix={}/", artifacts.display()))
        .arg(format!("-runs={RUNS}"))
        .current_dir(root)
        .env("ASAN_OPTIONS", "detect_leaks=1")
        .env(
            "LSAN_OPTIONS",
            format!("suppressions={}", lsan_suppressions.display()),
        )
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .group_spawn()?;
    supervise(child, DEADLINE)
}

fn campaign(root: &Path, target: Target) -> Result<Outcome, String> {
    if ambient_sanitizer_override(
        env::var_os("ASAN_OPTIONS").as_deref(),
        env::var_os("TL_PARSE_FUZZ_DISABLE_LEAKS").as_deref(),
        env::var_os("LSAN_OPTIONS").as_deref(),
    ) {
        return Err(
            "ASAN_OPTIONS, LSAN_OPTIONS, and TL_PARSE_FUZZ_DISABLE_LEAKS must be absent".to_owned(),
        );
    }
    match leak_sanitizer_available() {
        Ok(true) => {}
        Ok(false) => {
            return Err("no-new-privileges plus filtered seccomp prevents LeakSanitizer".to_owned())
        }
        Err(error) => {
            return Err(format!(
                "could not inspect LeakSanitizer availability: {error}"
            ))
        }
    }

    let scratch_parent = target_directory(root).join("campaigns");
    fs::create_dir_all(&scratch_parent)
        .map_err(|error| format!("could not create {}: {error}", scratch_parent.display()))?;
    let scratch = tempfile::Builder::new()
        .prefix(&format!("{}-", target.as_str()))
        .tempdir_in(&scratch_parent)
        .map_err(|error| format!("could not allocate campaign scratch: {error}"))?;
    let generated = scratch.path().join("generated");
    let seeds = scratch.path().join("seeds");
    let artifacts = scratch.path().join("artifacts");
    for directory in [&generated, &seeds, &artifacts] {
        fs::create_dir(directory)
            .map_err(|error| format!("could not create {}: {error}", directory.display()))?;
    }
    copy_seeds(
        &root.join("fuzz").join("corpus").join(target.as_str()),
        &seeds,
    )
    .map_err(|error| format!("could not stage seeds: {error}"))?;

    let process = run_fuzzer(root, target, &generated, &seeds, &artifacts)
        .map_err(|error| format!("could not run cargo-fuzz: {error}"))?;
    let artifacts_present = fs::read_dir(&artifacts)
        .map_err(|error| format!("could not enumerate {}: {error}", artifacts.display()))?
        .next()
        .is_some();
    let outcome = classify(process, artifacts_present);
    if outcome != Outcome::Pass {
        let retained = scratch.into_path();
        eprintln!("fuzz campaign scratch retained at {}", retained.display());
    }
    Ok(outcome)
}

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let (Some(raw_target), None) = (arguments.next(), arguments.next()) else {
        eprintln!("usage: fuzz_campaign <parser|clean_ascii_v2>");
        return ExitCode::from(64);
    };
    let Some(target) = Target::parse(&raw_target) else {
        eprintln!("unknown fuzz target {raw_target:?}; expected parser or clean_ascii_v2");
        return ExitCode::from(64);
    };
    match campaign(Path::new(env!("CARGO_MANIFEST_DIR")), target) {
        Ok(outcome) => ExitCode::from(outcome.exit_code()),
        Err(error) => {
            eprintln!("fuzz campaign unavailable: {error}");
            ExitCode::from(Outcome::Unavailable.exit_code())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ambient_sanitizer_override, classify, supervise, Outcome, ProcessState, Target};
    use std::ffi::OsStr;

    use command_group::CommandGroup;

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_047_classifies_every_process_and_artifact_combination() {
        assert_eq!(classify(ProcessState::Exited(0), false), Outcome::Pass);
        assert_eq!(classify(ProcessState::Exited(9), false), Outcome::Fail);
        assert_eq!(classify(ProcessState::Exited(0), true), Outcome::Fail);
        assert_eq!(
            classify(ProcessState::TimedOut, false),
            Outcome::Unavailable
        );
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_047_refuses_unknown_targets_and_ambient_sanitizer_overrides() {
        assert_eq!(Target::parse(OsStr::new("parser")), Some(Target::Parser));
        assert_eq!(
            Target::parse(OsStr::new("clean_ascii_v2")),
            Some(Target::CleanAsciiV2)
        );
        assert_eq!(Target::parse(OsStr::new("../parser")), None);
        assert!(ambient_sanitizer_override(Some(OsStr::new("")), None, None));
        assert!(ambient_sanitizer_override(
            None,
            Some(OsStr::new("0")),
            None
        ));
        assert!(ambient_sanitizer_override(None, None, Some(OsStr::new(""))));
        assert!(!ambient_sanitizer_override(None, None, None));
    }

    // Trace: TC-047, FR-005-AC-2
    #[test]
    fn tc_047_zero_deadline_terminates_and_reaps_the_whole_process_group() {
        let mut command = std::process::Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let child = command.group_spawn().expect("spawn process group");
        assert_eq!(
            supervise(child, std::time::Duration::ZERO).expect("supervise process group"),
            ProcessState::TimedOut
        );
    }
}
