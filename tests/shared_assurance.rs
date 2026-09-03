//! Tests for the shared assurance intake path (FR-006).
//!
//! These follow this repository's own binding idiom: a `// Trace:` comment above
//! each `#[test]`, which is what Quire's census reads. They invoke the gates
//! rather than reimplementing them, because a test that recomputes what a gate
//! computes is a second implementation that can agree with itself while both are
//! wrong.
//!
//! A missing prerequisite is a failure here, never a skip. A gate that stands
//! down when its dependency is absent reports the same green as one that ran.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The interpreter `make assurance-env` builds. Its absence is an error.
fn assurance_python() -> PathBuf {
    let path = std::env::var_os("ASSURANCE_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join(".venv-assurance/bin/python"));
    assert!(
        path.is_file(),
        "the pinned assurance interpreter is missing at {}. Run `make assurance-env`. \
         This is a failure and not a skip: a gate that stands down when its dependency \
         is absent reports the same green as one that ran.",
        path.display()
    );
    path
}

fn run(program: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root())
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()));
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn json_gate(program: &Path, arguments: &[&str]) -> Value {
    let (code, stdout, stderr) = run(program, arguments);
    assert_eq!(code, 0, "{arguments:?} exited {code}\n{stdout}\n{stderr}");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{arguments:?} did not emit JSON: {error}\n{stdout}"))
}

fn head_revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root())
        .output()
        .expect("git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

/// The chain is expensive and several tests read it. It runs once per test
/// binary, and every reader sees the same run rather than a different one.
static CHAIN: OnceLock<Value> = OnceLock::new();

fn chain_report() -> &'static Value {
    CHAIN.get_or_init(|| {
        // The chain runs under the system interpreter: it only shells out to
        // quoin and never imports engineering-assurance.
        let revision = head_revision();
        let (code, stdout, stderr) = run(
            Path::new("python3"),
            &[
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
                "--json",
            ],
        );
        assert_eq!(code, 0, "the assurance chain exited {code}\n{stderr}");
        serde_json::from_str(&stdout).expect("the assurance chain did not emit JSON")
    })
}

// Trace: TC-022, FR-006-AC-1
#[test]
fn every_shared_pin_is_classified_by_the_packaged_matrix() {
    let python = assurance_python();
    let report = json_gate(&python, &["scripts/check_shared_pins.py", "--json"]);

    let components = report["components"].as_array().expect("components array");
    assert_eq!(
        components.len(),
        4,
        "the matrix pins four components; this run classified {}",
        components.len()
    );
    for component in components {
        assert_eq!(
            component["verdict"], "compatible",
            "{} is {} ({})",
            component["component"], component["verdict"], component["reason"]
        );
    }
    assert_eq!(report["accepted"], true);
    assert!(report["artifact_mismatches"].as_array().unwrap().is_empty());
    assert!(report["mirror_references"].as_array().unwrap().is_empty());
    assert!(
        report["upstream_pin_mismatches"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the tl-syntax pin disagrees across the files that name it: {}",
        report["upstream_pin_mismatches"]
    );

    // Acceptance is reported and never gated on: the pinned release records
    // `pending_human_acceptance` and ships no predicate for it
    // (agent-ix/engineering-assurance#20). Reading an absent field as approval,
    // in either direction, is the mistake this asserts against.
    assert_eq!(report["acceptance_recorded_here"], false);
    assert!(report["acceptance_state"].is_string());

    // The mirror check must be seen to refuse. Without this it is indistinguishable
    // from a check that matches nothing.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['engineering_assurance']['requirement']+=' --registry=https://npm.ix/';\
             print(json.dumps(m.mirror_references(pins)))",
        ],
    );
    assert_eq!(code, 0, "the mirror probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        !offenders.is_empty(),
        "a mirror registry reference was not detected; the check matches nothing"
    );
}

// Trace: TC-023, FR-006-AC-2, NFR-003-AC-1, SUITE-006, SUITE-007, SUITE-009
#[test]
fn the_chain_reaches_quoin_without_quoin_or_quire_executing_a_producer() {
    let report = chain_report();
    assert_eq!(report["matched"], true, "{report:#}");

    for group in ["scenarios", "controls", "adapter_probes"] {
        let items = report[group]
            .as_array()
            .unwrap_or_else(|| panic!("{group}"));
        assert!(!items.is_empty(), "{group} is empty");
        for item in items {
            assert_eq!(
                item["matched"], true,
                "{group} entry did not match: {item:#}"
            );
        }
    }

    // Every attested result is read out of the producer's bytes. Asserting the
    // values here means a chain that reverted to sealing a literal "passed"
    // would still have to agree with what the producers actually wrote.
    let attested = report["attested_results"]
        .as_object()
        .expect("attested_results");
    assert_eq!(
        attested.len(),
        5,
        "five proof obligations are declared; {} were attested",
        attested.len()
    );
    for (proof, result) in attested {
        assert_eq!(result, "passed", "{proof} was attested {result}");
    }

    // The adapter transcribes one named protocol and refuses another, rather than
    // guessing. A verdict recovered from an unrecognised stream is a verdict
    // recovered from nothing.
    let probes = report["adapter_probes"].as_array().unwrap();
    for required in [
        "refuses-a-foreign-protocol",
        "refuses-an-unnamed-outcome",
        "refuses-an-empty-stream",
        "accepts-the-real-run",
    ] {
        assert!(
            probes.iter().any(|probe| probe["probe"] == required),
            "adapter probe {required} is missing"
        );
    }
}

/// Write an executable shim for each name that records every invocation.
///
/// The log is the point. A shim that is never consulted and a producer that is
/// never run look identical from the outside, so the shims write down every call
/// and the test reads the file rather than assuming.
///
/// A version query is answered rather than refused, and deliberately so. Asking
/// a tool its version is an observation — it is what the compatibility matrix's
/// own `observe` column does — and it is not the thing this test forbids. What
/// is forbidden is asking a tool to build, compile, test, parse, or replay
/// anything. Every such invocation is logged and the log must be empty.
///
/// `--version` is matched anywhere in the argv, not just in `$1`, because the
/// MSRV attestation observes `rustup run 1.75.0 cargo --version`: its declared
/// command runs cargo through the pinned toolchain, so the version sealed into
/// the attestation has to come from that toolchain rather than from ambient
/// cargo. That is still a version observation. Anything without a version flag
/// — `cargo build`, `cargo check`, `rustup run … check` — is logged and fails
/// the test, which is what keeps it able to fail.
fn producer_shims(directory: &Path, names: &[&str]) -> PathBuf {
    fs::create_dir_all(directory).unwrap();
    let log = directory.join("invocations.log");
    let _ = fs::remove_file(&log);
    for name in names {
        let path = directory.join(name);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 for argument in \"$@\"; do\n\
                 case \"$argument\" in\n\
                 --version|-V) echo \"{name} 9.9.9 (shim)\"; exit 0 ;;\n\
                 esac\n\
                 done\n\
                 echo \"$0 $@\" >> {}\n\
                 exit 97\n",
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    log
}

fn run_chain_with_path(shims: &Path) -> std::process::Output {
    let inherited = std::env::var("PATH").unwrap_or_default();
    let revision = head_revision();
    Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(root())
        .env("PATH", format!("{}:{inherited}", shims.display()))
        .output()
        .expect("failed to run the assurance chain")
}

// Trace: TC-023, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_chain_never_executes_a_producer_and_the_probe_can_prove_it() {
    // Two runs, because one proves nothing.
    //
    // Run A replaces every producer — cargo, rustup, rustc — with a stub that
    // logs and fails. The chain must finish, and the log must be empty: not one
    // producer was invoked.
    //
    // Run B is the control. It stubs `quoin`, which the chain is supposed to run,
    // and requires the chain to fail and the log to be non-empty. Without it, an
    // empty log in run A would be equally consistent with PATH never being
    // consulted at all.
    let producers = root().join("target/producer-shims");
    let producer_log = producer_shims(&producers, &["cargo", "rustup", "rustc"]);
    let output = run_chain_with_path(&producers);
    let logged = fs::read_to_string(&producer_log).unwrap_or_default();
    assert!(
        output.status.success(),
        "the assurance chain failed with producers stubbed, which means it ran one:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        logged.trim().is_empty(),
        "the assurance driver asked a producer to do work, not just to name its version:\n{logged}"
    );

    let tools = root().join("target/tool-shims");
    let tool_log = producer_shims(&tools, &["quoin"]);
    let control = run_chain_with_path(&tools);
    let tool_logged = fs::read_to_string(&tool_log).unwrap_or_default();
    assert!(
        !tool_logged.trim().is_empty(),
        "stubbing quoin produced no invocation, so PATH is not being consulted by \
         the subprocess and the run above proves nothing"
    );
    assert!(
        !control.status.success(),
        "the chain succeeded with quoin stubbed out, so it is not actually using it"
    );
}

// Trace: TC-024, FR-006-AC-3, SUITE-003
#[test]
fn the_sealed_records_impact_snapshot_is_the_quire_export() {
    let report = chain_report();
    let export = root().join(report["quire_export"].as_str().expect("quire_export"));
    let bytes =
        fs::read(&export).unwrap_or_else(|error| panic!("{} is absent: {error}", export.display()));

    let digest = {
        let output = Command::new("sha256sum")
            .arg(&export)
            .output()
            .expect("sha256sum failed");
        String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .next()
            .expect("sha256sum output")
            .to_owned()
    };
    assert_eq!(
        report["impact_snapshot_digest"], digest,
        "the sealed record's impact snapshot does not name the Quire export it claims"
    );
    // An empty object has a digest too. The snapshot is only worth its content,
    // so the export is required to actually carry the coverage facts the record
    // claims it snapshotted, and to name every requirement this repository has.
    let parsed: Value = serde_json::from_slice(&bytes).expect("the Quire export is JSON");
    let text = String::from_utf8_lossy(&bytes);
    for requirement in [
        "FR-001", "FR-002", "FR-003", "FR-004", "FR-005", "FR-006", "NFR-001", "NFR-002",
        "NFR-003", "StR-001", "StR-002",
    ] {
        assert!(
            text.contains(requirement),
            "the Quire export does not mention {requirement}; it is not a coverage \
             export of this repository"
        );
    }
    assert!(
        parsed.is_object() && !parsed.as_object().unwrap().is_empty(),
        "the Quire export is not a populated document"
    );

    // The measured coverage, pinned. `derive_result` refuses an export that
    // measured nothing or carries a status lie, but a partially-backed export is
    // legitimately not a failure here — four suite rows are deliberately
    // unbacked and SR-007 says why. So the figures themselves are asserted: an
    // export reporting different totals has to move a number in this file.
    let totals = &parsed["totals"];
    assert_eq!(totals["total"], 67, "matrix row count changed: {totals}");
    assert_eq!(
        totals["backed"], 63,
        "backed-row count changed: {totals}. Four suite rows are unbacked on \
         purpose; if that number moved, update spec/evidence/suites.md and SR-007 \
         deliberately rather than adjusting this assertion."
    );
    assert!(
        parsed["status_lies"].as_array().unwrap().is_empty(),
        "Quire reported a row whose declared status disagrees with its evidence: {}",
        parsed["status_lies"]
    );

    // And the chain must have read it as such rather than as a not-computed run.
    assert_eq!(
        report["attested_results"]["PROOF-quire-static-export"], "passed",
        "the Quire export was attested as {}",
        report["attested_results"]["PROOF-quire-static-export"]
    );
}

// Trace: TC-026, FR-006-AC-5, NFR-003-AC-3
#[test]
fn all_twelve_verification_outcomes_are_demonstrated_and_paired_with_controls() {
    // The twelve states this migration must keep distinguishable, and the gate
    // that owns each. A state nobody demonstrates is a state nobody would notice
    // the loss of.
    //
    // `malformed` is owned by the parser corpus rather than by an evidence lane,
    // which is where tl-syntax demonstrated it. That is this repository's domain
    // behaviour: six of its seven corpus fixtures are malformed by design.
    const REQUIRED: [(&str, &str); 12] = [
        ("pass", "chain"),
        ("fail", "chain"),
        ("unavailable", "chain"),
        ("unsupported", "chain"),
        ("inconclusive", "chain"),
        ("not-computed", "chain"),
        ("malformed", "chain/parser-corpus"),
        ("partial", "chain"),
        ("stale", "chain"),
        ("suspect", "chain"),
        ("vacuous", "chain"),
        ("tampered", "chain"),
    ];

    let report = chain_report();

    // Only MEASURED outcomes count. The chain's `states_demonstrated` is built
    // from scenarios and adapter probes that ran and matched, never from a
    // free-text label a fixture declares about itself. Counting a label would
    // let a state stop being demonstrated while this test stayed green, which is
    // the exact failure mode this test exists to rule out.
    //
    // All twelve now come from the chain alone. They used to be split with the
    // retained-evidence compatibility lane, which was deleted along with the
    // records it read; the chain already demonstrated every one of the twelve on
    // its own, so nothing moved out of reach.
    let demonstrated: BTreeSet<String> = report["states_demonstrated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();

    let missing: Vec<&str> = REQUIRED
        .iter()
        .filter(|(state, _)| !demonstrated.contains(*state))
        .map(|(state, _)| *state)
        .collect();
    assert!(
        missing.is_empty(),
        "these verification outcomes were never demonstrated: {missing:?}; \
         demonstrated: {demonstrated:?}"
    );

    // Every negative names the positive control that proves the step it refuses
    // is a step that works.
    let controls = report["controls"].as_array().unwrap();
    assert!(!controls.is_empty(), "no positive controls were run");
    let negatives: BTreeSet<&str> = controls
        .iter()
        .map(|control| control["pairs_with"].as_str().unwrap())
        .collect();
    for required in [
        "retained-bytes-changed-after-sealing",
        "refuse-an-edited-receipt",
        "stale-candidate-binding",
        "attested-failed",
        "malformed-input-is-reported-as-malformed",
    ] {
        assert!(
            negatives.contains(required),
            "the negative {required} has no positive control"
        );
    }
}

// Trace: TC-027, FR-006-AC-6, StR-002-VC-2
#[test]
fn malformed_input_is_reported_as_malformed_and_never_as_a_pass() {
    let report = chain_report();

    // The count comes from the corpus manifest, so a producer that stopped
    // reporting malformed rows cannot also move the number it is checked
    // against. Six of the seven fixtures declare a diagnostic code.
    let declared = report["declared_malformed_fixtures"].as_u64().unwrap();
    let reported = report["malformed_rows"].as_u64().unwrap();
    assert_eq!(
        declared, 6,
        "the corpus manifest declares {declared} malformed fixtures; if a fixture was \
         added or removed this expectation should move deliberately"
    );
    assert_eq!(
        reported, declared,
        "the producer reported {reported} malformed rows for {declared} declared \
         malformed fixtures"
    );

    // The three facts the chain asserts, each named, so that dropping any one of
    // them is visible here rather than only inside the driver.
    let scenarios = report["scenarios"].as_array().unwrap();
    for required in [
        "malformed-input-is-reported-as-malformed",
        "malformed-does-not-fail-its-proof",
        "malformed-survives-into-retained-bytes",
    ] {
        let found = scenarios
            .iter()
            .find(|item| item["scenario"] == required)
            .unwrap_or_else(|| panic!("the scenario {required} did not run"));
        assert_eq!(
            found["matched"], true,
            "{required} did not match: {found:#}"
        );
    }

    // Malformed is not a failure: the proof it belongs to is attested `passed`.
    assert_eq!(
        report["attested_results"]["PROOF-parser-conformance"], "passed",
        "a corpus that is malformed by design dragged its proof to a failure"
    );

    // And it is not a silent pass either: the producer's own rows say
    // `malformed`, and the adapter carries that word alongside Quoin's
    // three-valued entry outcome rather than discarding it.
    let python = std::env::var_os("ASSURANCE_PYTHON").is_some();
    let _ = python;
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "scripts/assurance_chain.py",
            "--adapt",
            "target/assurance/parser-conformance.jsonl",
        ],
    );
    assert_eq!(code, 0, "the adapter refused the real stream: {stderr}");
    let adapted: Value = serde_json::from_str(&stdout).expect("the adapter emits JSON");
    let carried = adapted["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["domainOutcome"] == "malformed")
        .count() as u64;
    assert_eq!(
        carried, declared,
        "the adapter dropped the malformed domain outcome; Quoin's entry vocabulary \
         is three-valued, so the twelve-state word has to survive alongside it"
    );
}

// Trace: TC-028, FR-006-AC-7
#[test]
fn no_local_evidence_framework_remains_and_none_of_its_files_came_back() {
    let root = root();

    // The generic machinery is gone, by name.
    for removed in [
        "scripts/build_evidence_envelope.py",
        "scripts/collect_evidence.sh",
        "scripts/finalize_collection.py",
        "scripts/verify_evidence.sh",
        "scripts/verify_evidence_manifest.py",
        "scripts/verify_evidence_history.py",
        "scripts/evidence_profile.py",
        "scripts/check_failure_propagation.py",
        "scripts/check_traceability_coverage.py",
        "scripts/run_local_ci.py",
        "scripts/run_policy_tests.py",
        "scripts/tool_identity.py",
        "scripts/run_cargo_toolchain.py",
        "scripts/validate_json_schema.py",
        "scripts/test_evidence_tool.py",
        "scripts/test_evidence_history.py",
        "scripts/test_collector_behavior.py",
        "scripts/test_failure_propagation.py",
        "scripts/test_json_schema_gate.py",
        "scripts/test_traceability_gate.py",
        "tools.lock",
        "tests/evidence_contract.rs",
        // Deleted under the owner's pre-stable release of the preservation
        // constraint (agent-ix/engineering-assurance#7, agent-ix/tl-parse#13).
        // Named here so a reintroduction is a test failure rather than a quiet
        // return of the machinery this repository decided not to carry.
        "evidence",
        "schemas",
        "scripts/legacy_evidence_view.py",
        "tests/fixtures/legacy-compat",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} is still present; the generic evidence machinery was not removed"
        );
    }

    // The Makefile is orchestration, not a trust root, and carries no gate that
    // polices its own execution.
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    for gone in [
        "check-failure-propagation",
        "ci-for-evidence",
        "verify-evidence",
        "evidence-tool",
        "rust-test-census",
        "compat-view",
    ] {
        assert!(
            !makefile.contains(gone),
            "the Makefile still carries the {gone} self-attestation target"
        );
    }
}

// Trace: TC-026, FR-006-AC-5, NFR-003-AC-3
#[test]
fn a_control_naming_a_scenario_that_does_not_exist_is_refused() {
    // NFR-003-AC-3 claims this guard is checked. It was not: the guard existed
    // and nothing exercised it, which is the same shape of gap the guard itself
    // is there to catch.
    //
    // The driver is copied and one `pairs_with` — and only that one — is
    // renamed. Renaming the scenario as well would leave the pairing consistent
    // and prove nothing, which is exactly how the first version of this probe
    // failed to detect anything.
    let scratch = root().join("target/dangling-probe");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();

    let control_marker =
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt\",";
    assert!(
        driver.contains(control_marker),
        "the control this probe renames is no longer present in the driver"
    );
    let mutated = driver.replacen(
        control_marker,
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt-typo\",",
        1,
    );
    assert_ne!(mutated, driver, "the mutation did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();

    // Everything else the driver reads comes from the real tree. Every root
    // entry except `scripts` and `target` is symlinked, rather than an enumerated list, so
    // that a driver which starts reading a new directory does not turn this
    // probe into one that fails for an unrelated reason. The scratch owns its
    // Quoin store and shares only the already-produced assurance inputs;
    // symlinking all of `target` coupled this probe to the real store.
    let scratch_target = scratch.join("target");
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "scripts" || name == ".git" || name == "target" {
            continue;
        }
        std::os::unix::fs::symlink(&path, scratch.join(&name))
            .unwrap_or_else(|error| panic!("failed to link {name} into the probe: {error}"));
    }
    // Create this after the loop so dropping `target` from the skip set creates
    // a symlink that the ownership assertion below can see and reject.
    fs::create_dir_all(&scratch_target).expect("create isolated probe target");
    std::os::unix::fs::symlink(
        root().join("target/assurance"),
        scratch_target.join("assurance"),
    )
    .expect("share assurance inputs with the isolated probe");
    assert!(
        !fs::symlink_metadata(&scratch_target)
            .expect("scratch target metadata")
            .file_type()
            .is_symlink(),
        "the dangling-scenario probe must own target/ so its Quoin store is isolated"
    );

    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the mutated chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a control naming a non-existent scenario was not refused\n{stderr}"
    );
    assert!(
        stderr.contains("name a scenario that does not exist"),
        "the refusal did not name the cause: {stderr}"
    );
    let scratch_store = fs::canonicalize(scratch_target.join("assurance-store"))
        .expect("the mutated driver created its isolated Quoin store");
    let real_store = fs::canonicalize(root().join("target/assurance-store"))
        .expect("canonical real Quoin store");
    assert_ne!(
        scratch_store, real_store,
        "the dangling-scenario probe resolved its Quoin store into the real tree"
    );

    // The same isolated environment must succeed once the deliberate dangling
    // reference is removed. This bypassed half proves the expected exit 2 is
    // caused by the validator rather than an earlier scratch-construction fault.
    fs::write(scratch.join("scripts/assurance_chain.py"), &driver).unwrap();
    let bypassed = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the unmutated chain in the isolated scratch");
    assert_eq!(
        bypassed.status.code(),
        Some(0),
        "the isolated scratch is not a valid environment for the unmutated chain:\n{}\n{}",
        String::from_utf8_lossy(&bypassed.stdout),
        String::from_utf8_lossy(&bypassed.stderr)
    );

    // Unlink shared repository inputs explicitly before recursively removing
    // only the real scratch directories.
    fs::remove_file(scratch_target.join("assurance")).expect("unlink shared assurance inputs");
    for entry in fs::read_dir(&scratch).expect("read dangling-probe scratch") {
        let path = entry.expect("scratch entry").path();
        if fs::symlink_metadata(&path)
            .expect("scratch entry metadata")
            .file_type()
            .is_symlink()
        {
            fs::remove_file(path).expect("unlink dangling-probe repository input");
        }
    }
    fs::remove_dir_all(&scratch).expect("remove dangling-probe scratch");
}

// Trace: TC-022, FR-006-AC-1
#[test]
fn the_mirror_scan_refuses_a_registry_reference_in_a_real_file() {
    // The structural branch of `mirror_references` (pins.json) already has a
    // control. The file-scan branch did not: it was never observed to fire, so
    // it was indistinguishable from a loop over files that never match.
    let python = assurance_python();
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys,tempfile,pathlib;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             original=pathlib.Path('requirements-assurance.txt').read_text();\
             pathlib.Path('requirements-assurance.txt').write_text(\
             original+'\\n--registry=https://npm.ix/\\n');\
             pins=json.load(open('assurance/pins.json'));\
             found=m.mirror_references(pins);\
             pathlib.Path('requirements-assurance.txt').write_text(original);\
             print(json.dumps(found))",
        ],
    );
    assert_eq!(code, 0, "the mirror file-scan probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        offenders
            .iter()
            .any(|entry| entry.starts_with("requirements-assurance.txt:")),
        "a mirror reference written into a scanned FILE was not detected; the \
         file-scan branch matches nothing. Detected: {offenders:?}"
    );

    // And the file must be restored, or this test has dirtied the tree.
    let restored = fs::read_to_string(root().join("requirements-assurance.txt")).unwrap();
    assert!(
        !restored.contains("npm.ix/"),
        "the probe left a mirror reference in requirements-assurance.txt"
    );
}
