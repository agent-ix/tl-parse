#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from contextlib import contextmanager
from pathlib import Path

from jsonschema import Draft7Validator


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "build_evidence_envelope", ROOT / "scripts" / "build_evidence_envelope.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
FINALIZER_SPEC = importlib.util.spec_from_file_location(
    "finalize_collection", ROOT / "scripts" / "finalize_collection.py"
)
assert FINALIZER_SPEC is not None and FINALIZER_SPEC.loader is not None
FINALIZER = importlib.util.module_from_spec(FINALIZER_SPEC)
FINALIZER_SPEC.loader.exec_module(FINALIZER)
VERIFIER_SPEC = importlib.util.spec_from_file_location(
    "verify_evidence_manifest", ROOT / "scripts" / "verify_evidence_manifest.py"
)
assert VERIFIER_SPEC is not None and VERIFIER_SPEC.loader is not None
VERIFIER = importlib.util.module_from_spec(VERIFIER_SPEC)
VERIFIER_SPEC.loader.exec_module(VERIFIER)


@contextmanager
def temporary_worktree():
    with tempfile.TemporaryDirectory() as directory:
        checkout = Path(directory) / "checkout"
        subprocess.run(
            ["/usr/bin/git", "worktree", "add", "--detach", "-q", str(checkout), "HEAD"],
            cwd=ROOT, check=True,
        )
        try:
            overlay = [ROOT / "tools.lock", *sorted((ROOT / "scripts").glob("*.py"))]
            overlay.extend(sorted((ROOT / "scripts").glob("*.sh")))
            relative_overlay = []
            for source in overlay:
                relative = source.relative_to(ROOT)
                destination = checkout / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(source, destination)
                relative_overlay.append(str(relative))
            subprocess.run(
                ["/usr/bin/git", "add", *relative_overlay], cwd=checkout, check=True
            )
            subprocess.run(
                [
                    "/usr/bin/git", "-c", "user.name=Policy Test",
                    "-c", "user.email=policy@example.invalid", "commit",
                    "--allow-empty", "-qm", "overlay evidence policy under test",
                ],
                cwd=checkout,
                check=True,
            )
            yield checkout
        finally:
            subprocess.run(
                ["/usr/bin/git", "worktree", "remove", "--force", str(checkout)],
                cwd=ROOT, check=True,
            )


def healthy_ci_output() -> str:
    tests = "test result: ok. 1 passed; 0 failed; 0 ignored\n" * 7
    tests += "test result: ok. 21 passed; 0 failed; 0 ignored\n"
    corpus = "\n".join(sorted(FINALIZER.REQUIRED_CORPUS_LINES)) + "\n"
    signatures = (
        "qualification profile: peter-linux-x86_64-v1\n"
        "all 14 mandatory local-CI targets propagate failures\n"
        "all 6 policy behavior tests passed\n"
        "strict traceability coverage is complete: 62/62\n"
        "clean-room attribution digests match the pinned Cargo source\n"
        "LeakSanitizer enabled\n"
        "fmt-check gate passed\n"
        "lint gate passed\n"
        "Rust test gate passed\n"
        "compiled Rust test census passed: 28 requirement-tagged tests\n"
        "rust-test-census gate passed\n"
        "corpus-integrity gate passed\n"
        "fuzz-build gate passed\n"
        "fuzz-smoke gate passed\n"
        "deny gate passed\n"
        "audit-unsafe gate passed\n"
        "evidence-tool gate passed\n"
        "spec gate passed\n"
        "msrv gate passed\n"
        "rustdoc gate passed\n"
    )
    return tests + corpus + signatures


def healthy_retained_output(name: str) -> str:
    if name == "make-ci":
        return healthy_ci_output()
    if name in {"make-spec", "quire-coverage"}:
        return "strict traceability coverage is complete: 62/62\n"
    if name == "rustdoc":
        return "Generated /tmp/doc/tl_parse/index.html\n"
    if name == "msrv":
        return "msrv gate passed\n"
    if name == "default-dependencies":
        return "tl-parse v0.1.0 (/tmp/tl-parse)\n"
    if name == "corpus-integrity":
        return "\n".join(sorted(FINALIZER.REQUIRED_CORPUS_LINES)) + "\n"
    if name in {
        "input-schema", "manifest-schema", "pgm01-schema", "pgm01-validator",
        "sealed-pgm01-schema", "sealed-pgm01-validator",
    }:
        return '{"errors": [], "valid": true}\n'
    return "verified\n"


def assert_schema_contracts() -> None:
    digest = {"algorithm": "sha256", "value": "a" * 64}
    source = "b" * 40
    input_schema = json.loads(
        (ROOT / "schemas" / "tl-parse-evidence-input-v1.schema.json").read_text()
    )
    input_value = {
        "schemaVersion": "tl-parse.evidence-input/v1",
        "sourceRevision": source,
        "sourceState": "clean",
        "commands": ["make ci"],
        "tools": {name: "v1" for name in ["cargo", "jsonschema", "python", "quire", "rustc"]},
        "pgm01": {
            "policy": "ix://agent-ix/quire-contract-ir/PGM-01",
            "candidateRevision": source,
            "envelopeSchema": "quire.derivation-evidence/v1",
            "envelopeSchemaDigest": digest,
        },
        "dependency": {"tlSyntaxRevision": source, "cargoLockDigest": digest},
        "dialect": {
            "revision": "tl-parse.clean-ascii/v1",
            "recordDigest": digest,
            "documentDigest": digest,
        },
        "corpus": {
            "revision": "tl-parse-corpus/v1",
            "manifestDigest": digest,
            "checksumDigest": digest,
            "fuzzChecksumDigest": digest,
        },
        "limits": {
            "sourceBytes": 1,
            "tokens": 1,
            "nodes": 1,
            "depth": 1,
            "diagnostics": 1,
            "parseWork": 1,
            "outputBytes": 1,
            "formatWork": 1,
        },
    }
    validator = Draft7Validator(input_schema)
    assert not list(validator.iter_errors(input_value))
    qualified_input = json.loads(json.dumps(input_value))
    qualified_input["qualificationProfile"] = "tl-parse.evidence-qualification/v2"
    qualified_input["toolProfile"] = "reviewed-runner-v1"
    qualified_input["tools"]["identities"] = {
        name: {"path": f"/reviewed/bin/{name}", "sha256": "d" * 64}
        for name in FINALIZER.tool_identity.REQUIRED
    }
    assert not list(validator.iter_errors(qualified_input))
    del qualified_input["toolProfile"]
    assert list(validator.iter_errors(qualified_input)), (
        "qualification-v2 input omitted its selected tool profile"
    )
    for mutate in [
        lambda value: value.update({"fabricated": True}),
        lambda value: value["dependency"].update({"fabricated": True}),
        lambda value: value["dependency"]["cargoLockDigest"].update({"algorithm": "sha1"}),
        lambda value: value.update({"sourceState": "dirty"}),
        lambda value: value.update({"commands": []}),
    ]:
        candidate = json.loads(json.dumps(input_value))
        mutate(candidate)
        assert list(validator.iter_errors(candidate)), candidate

    manifest_schema = json.loads(
        (ROOT / "schemas" / "tl-parse-evidence-manifest-v1.schema.json").read_text()
    )
    manifest_value = {
        "schemaVersion": "tl-parse.evidence-manifest/v1",
        "sourceRevision": source,
        "collectedAt": "2026-08-31T00:00:00Z",
        "outcomes": [{"name": "make-ci", "status": "passed", "exitCode": 0}],
        "artifacts": [{"path": "make-ci.stdout", "sha256": "c" * 64, "size": 1}],
        "limitations": ["human review pending"],
    }
    validator = Draft7Validator(manifest_schema)
    assert not list(validator.iter_errors(manifest_value))
    for mutate in [
        lambda value: value["outcomes"][0].update({"status": "fabricated"}),
        lambda value: value["artifacts"][0].update({"sha256": "short"}),
        lambda value: value["artifacts"][0].update({"fabricated": True}),
    ]:
        candidate = json.loads(json.dumps(manifest_value))
        mutate(candidate)
        assert list(validator.iter_errors(candidate)), candidate


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    assert_schema_contracts()
    registry = json.loads((ROOT / "evidence" / "RETRACTIONS.json").read_text())
    retracted_name = sorted(registry["records"])[0]
    assert FINALIZER.evidence_profile.resolve_profile(
        ROOT / "evidence" / retracted_name
    ) == "retracted"
    retracted_check = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts" / "finalize_collection.py"),
            "--check",
            str(ROOT / "evidence" / retracted_name),
        ],
        check=False,
        capture_output=True,
    )
    assert retracted_check.returncode == 3, (
        "explicitly retracted evidence did not retain its distinct non-passing exit"
    )
    assured = ROOT / "evidence" / "tl-parse-v01-2b295d21fef6-20260831T233256Z"
    assert FINALIZER.evidence_profile.resolve_profile(assured) == "unsupported-lock-schema", (
        "the assurance-bound v1 lock was not distinguished from checked failure"
    )
    unsupported_check = subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts" / "finalize_collection.py"),
            "--check",
            str(assured),
        ],
        check=False,
        capture_output=True,
    )
    assert unsupported_check.returncode == 4, (
        "unsupported source tool-lock schema lacked its distinct verification exit"
    )
    with tempfile.TemporaryDirectory(prefix="tl-parse-inconclusive-") as directory:
        inconclusive = Path(directory)
        (inconclusive / "collection-input.json").write_text("{}\n", encoding="utf-8")
        assert FINALIZER.evidence_profile.resolve_profile(inconclusive) == "inconclusive"
    collector = (ROOT / "scripts" / "collect_evidence.sh").read_text(encoding="utf-8")
    retained_calls = [
        line.strip() for line in collector.splitlines()
        if line.strip().startswith("run_and_retain ")
    ]
    assert len(retained_calls) == 14
    assert all('"${clean_env[@]}"' in line for line in retained_calls), (
        "one or more retained commands bypass the clean environment"
    )
    with tempfile.TemporaryDirectory() as directory:
        evidence_dir = Path(directory)
        (evidence_dir / "make-ci.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        (evidence_dir / "pgm01-schema.status.txt").write_text("125\n", encoding="utf-8")
        (evidence_dir / "pgm01-schema.stdout").write_text(
            "ordinary-output\n", encoding="utf-8"
        )
        (evidence_dir / "pgm01-validator.status.txt").write_text("3\n", encoding="utf-8")
        outcomes = {item["name"]: item for item in MODULE.command_outcomes(evidence_dir)}
        assert outcomes["make-ci"] == {
            "name": "make-ci",
            "status": "passed",
            "exitCode": 0,
        }
        assert outcomes["pgm01-schema"] == {
            "name": "pgm01-schema",
            "status": "skipped-unavailable",
            "exitCode": 125,
        }
        assert outcomes["pgm01-validator"] == {
            "name": "pgm01-validator",
            "status": "failed",
            "exitCode": 3,
        }
        assert outcomes["make-spec"] == {
            "name": "make-spec",
            "status": "inconclusive",
            "exitCode": None,
        }
        assert MODULE.classify_result("final", [outcomes["make-ci"]])[0] == "inconclusive"
        assert MODULE.classify_result("provisional", [outcomes["make-ci"]])[0] == "inconclusive"
        assert MODULE.classify_result("sealed-failed", [outcomes["make-ci"]])[0] == "error"
        assert MODULE.classify_result("final", [outcomes["pgm01-schema"]])[0] == "inconclusive"
        assert MODULE.classify_result("final", [outcomes["pgm01-validator"]])[0] == "error"

        (evidence_dir / "evidence-envelope.json").write_text(
            "{}\n", encoding="utf-8"
        )
        revision = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"], cwd=ROOT, check=True,
            capture_output=True, text=True,
        ).stdout.strip()
        source_builder = FINALIZER.git_bytes(revision, MODULE.BUILDER)
        parameters = FINALIZER.historical_parameters_digest(revision, source_builder)
        (evidence_dir / "source-revision.txt").write_text(revision + "\n", encoding="utf-8")
        source_lock = json.loads(FINALIZER.git_bytes(revision, MODULE.TOOLS_LOCK))
        profile_name = source_lock["defaultProfile"]
        profile_value = source_lock["profiles"][profile_name]
        source_names = set(profile_value["tools"])
        required = (
            FINALIZER.tool_identity.REQUIRED
            if source_names == set(FINALIZER.tool_identity.REQUIRED)
            else FINALIZER.tool_identity.LEGACY_REQUIRED
        )
        _, locked_tools = FINALIZER.tool_identity.validate_profile(
            profile_name, profile_value, required=required
        )
        (evidence_dir / "qualification-profile.txt").write_text(
            profile_name + "\n", encoding="utf-8"
        )
        collection_input = {
            "qualificationProfile": "tl-parse.evidence-qualification/v2",
            "toolProfile": profile_name,
            "tools": {"identities": locked_tools},
        }
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(collection_input) + "\n", encoding="utf-8",
        )
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps({
                "result": {"status": "inconclusive"},
                "parametersDigest": {"value": parameters},
            }) + "\n",
            encoding="utf-8",
        )
        for name in FINALIZER.CHECKS:
            (evidence_dir / f"{name}.status.txt").write_text("0\n", encoding="utf-8")
            (evidence_dir / f"{name}.stdout").write_text(
                healthy_retained_output(name), encoding="utf-8"
            )
            (evidence_dir / f"{name}.stderr").write_text("", encoding="utf-8")
        retained = FINALIZER.summary(evidence_dir)
        assert retained["overallStatus"] == "passed"
        for name in FINALIZER.CHECKS:
            if name == "diff-integrity":
                continue
            stdout_path = evidence_dir / f"{name}.stdout"
            original_output = stdout_path.read_text(encoding="utf-8")
            stdout_path.write_text("", encoding="utf-8")
            assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed", (
                f"zero-exit {name} evidence passed without its positive output contract"
            )
            stdout_path.write_text(original_output, encoding="utf-8")
        (evidence_dir / "msrv.status.txt").unlink()
        (evidence_dir / "msrv.stdout").unlink()
        (evidence_dir / "msrv.stderr").unlink()
        missing_msrv = FINALIZER.summary(evidence_dir)
        assert next(
            item for item in missing_msrv["outcomes"] if item["name"] == "msrv"
        )["status"] == "inconclusive"
        for suffix, content in (
            ("status.txt", "0\n"),
            ("stdout", healthy_retained_output("msrv")),
            ("stderr", ""),
        ):
            (evidence_dir / f"msrv.{suffix}").write_text(content, encoding="utf-8")
        expected_tests = len(FINALIZER.rust_test_census.git_tagged_test_names(ROOT, revision))
        assert expected_tests == 28, "reviewed source Rust-test known answer drifted"
        assert FINALIZER.positive_ci_census(
            healthy_ci_output(),
            expected_rust_tests=expected_tests,
            expected_profile=profile_name,
        )
        assert not FINALIZER.positive_ci_census(
            healthy_ci_output().replace(
                "qualification profile: peter-linux-x86_64-v1",
                "qualification profile: undeclared-profile-v1",
            ),
            expected_rust_tests=expected_tests,
            expected_profile=profile_name,
        ), "a transcript attributed to the wrong qualification profile passed"
        assert not FINALIZER.positive_ci_census(
            healthy_ci_output().replace(
                "compiled Rust test census passed: 28",
                "compiled Rust test census passed: 27",
            ),
            expected_rust_tests=expected_tests,
            expected_profile=profile_name,
        ), "compiled Rust-test census drift passed"
        assert not FINALIZER.positive_ci_census("cargo 1.94.1\n")
        zero_tests = "test result: ok. 0 passed; 0 failed; 0 ignored\n" * 8
        zero_tests += "\n".join(sorted(FINALIZER.REQUIRED_CORPUS_LINES)) + "\n"
        assert not FINALIZER.positive_ci_census(zero_tests), (
            "zero-test result groups satisfied the positive CI census"
        )
        (evidence_dir / "make-ci.stdout").write_text("", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed", (
            "the positive census was not applied through summary()"
        )
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")
        assert FINALIZER.validate_envelope_result(evidence_dir, retained) == []
        reordered = json.loads(json.dumps(retained))
        msrv = next(item for item in reordered["outcomes"] if item["name"] == "msrv")
        reordered["outcomes"] = [
            item for item in reordered["outcomes"] if item["name"] != "msrv"
        ] + [msrv]
        (evidence_dir / "collection-summary.json").write_text(
            json.dumps(reordered, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        order_check = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "finalize_collection.py"),
             "--check", str(evidence_dir)], check=False, capture_output=True,
        )
        assert order_check.returncode == 0, (
            "semantically identical retained outcomes were rejected solely by list order"
        )
        (evidence_dir / "qualification-profile.txt").write_text(
            "undeclared-profile-v1\n", encoding="utf-8"
        )
        assert FINALIZER.validate_tool_identity(evidence_dir, revision), (
            "an undeclared retained tool profile escaped source-lock re-derivation"
        )
        (evidence_dir / "qualification-profile.txt").write_text(
            profile_name + "\n", encoding="utf-8"
        )
        forged_profile = json.loads(json.dumps(collection_input))
        forged_profile["toolProfile"] = "different-profile-v1"
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(forged_profile) + "\n", encoding="utf-8"
        )
        assert FINALIZER.validate_tool_identity(evidence_dir, revision), (
            "a collection-input profile mismatch escaped source-lock re-derivation"
        )
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(collection_input) + "\n", encoding="utf-8"
        )
        forged_collection = json.loads(json.dumps(collection_input))
        forged_collection["tools"]["identities"]["cargo"]["sha256"] = "0" * 64
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(forged_collection) + "\n", encoding="utf-8",
        )
        assert FINALIZER.validate_tool_identity(evidence_dir, revision), (
            "a forged retained tool digest escaped source-lock re-derivation"
        )
        (evidence_dir / "collection-input.json").write_text(
            json.dumps(collection_input) + "\n", encoding="utf-8",
        )
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps({"result": {"status": "conclusive"}}) + "\n", encoding="utf-8"
        )
        assert FINALIZER.validate_envelope_result(evidence_dir, retained), (
            "a forged conclusive envelope result was accepted"
        )
        (evidence_dir / "collection-summary.json").write_text(
            json.dumps(retained, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        rejected = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "finalize_collection.py"),
             "--check", str(evidence_dir)], check=False, capture_output=True,
        )
        assert rejected.returncode != 0, "finalizer exit contract accepted a forged result"
        (evidence_dir / "evidence-envelope.json").write_text(
            json.dumps({"result": {"status": "inconclusive"}}) + "\n", encoding="utf-8"
        )
        (evidence_dir / "pgm01-validator.stderr").write_text(
            "governance validation error: fabricated pass\n", encoding="utf-8"
        )
        contradicted = FINALIZER.summary(evidence_dir)
        assert contradicted["overallStatus"] == "failed"
        assert next(
            item for item in contradicted["outcomes"] if item["name"] == "pgm01-validator"
        )["status"] == "failed"
        (evidence_dir / "pgm01-validator.stderr").write_text("", encoding="utf-8")
        (evidence_dir / "rustdoc.status.txt").write_text("1\n", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "msrv.status.txt").write_text("1\n", encoding="utf-8")
        censused = FINALIZER.summary(evidence_dir)
        assert any(item["name"] == "msrv" for item in censused["outcomes"])
        assert censused["overallStatus"] == "failed"

        (evidence_dir / "rustdoc.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "msrv.status.txt").write_text("0\n", encoding="utf-8")
        (evidence_dir / "msrv.stdout").write_text("", encoding="utf-8")
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed", (
            "zero-exit MSRV evidence passed without its positive signature"
        )
        (evidence_dir / "msrv.stdout").write_text(
            healthy_retained_output("msrv"), encoding="utf-8"
        )
        (evidence_dir / "make-ci.stdout").write_text(
            "test result: FAILED. 5 passed; 1 failed; 0 ignored\n", encoding="utf-8"
        )
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "make-ci.stdout").write_text(
            "test result: ok. 24 passed; 0 failed; 1 ignored\n", encoding="utf-8"
        )
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "make-ci.stdout").write_text(
            "make: [Makefile:51: test] Error 101 (ignored)\n", encoding="utf-8"
        )
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "make-ci.stdout").write_text(
            "LeakSanitizer explicitly disabled by TL_PARSE_FUZZ_DISABLE_LEAKS=1\n",
            encoding="utf-8",
        )
        assert FINALIZER.summary(evidence_dir)["overallStatus"] == "failed"
        (evidence_dir / "make-ci.stdout").write_text(healthy_ci_output(), encoding="utf-8")

        artifact = evidence_dir / "make-ci.stdout"
        artifact.write_text("passed\n", encoding="utf-8")
        manifest = {
            "artifacts": [
                {
                    "path": artifact.name,
                    "sha256": VERIFIER.sha256(artifact),
                    "size": artifact.stat().st_size,
                }
            ]
        }
        (evidence_dir / "evidence-manifest.json").write_text(
            json.dumps(manifest), encoding="utf-8"
        )
        evidence_dir.with_suffix(".sha256").write_text(
            "".join(
                f"{VERIFIER.sha256(path)}  {path}\n"
                for path in sorted(evidence_dir.iterdir())
                if path.is_file()
            ),
            encoding="utf-8",
        )
        assert VERIFIER.verify(evidence_dir) == []
        added = evidence_dir / "PLANTED-EXTRA.txt"
        added.write_text("FABRICATED\n", encoding="utf-8")
        assert any("unlisted" in error for error in VERIFIER.verify(evidence_dir))
        rejected = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "verify_evidence_manifest.py"),
             str(evidence_dir)], check=False, capture_output=True,
        )
        assert rejected.returncode != 0, "manifest verifier main accepted an extra artifact"
        added.unlink()
        symlink = evidence_dir / "PLANTED-LINK"
        symlink.symlink_to("make-ci.stdout")
        assert any("symlink" in error for error in VERIFIER.verify(evidence_dir))
        symlink.unlink()
        artifact.write_text("FABRICATED\n", encoding="utf-8")
        assert VERIFIER.verify(evidence_dir)

    with tempfile.TemporaryDirectory() as directory:
        test = Path(directory) / "test_fails.py"
        test.write_text("raise SystemExit(7)\n", encoding="utf-8")
        runner = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "run_policy_tests.py"),
             "--directory", directory], cwd=ROOT, check=False, capture_output=True,
        )
        assert runner.returncode != 0, "policy runner swallowed a failing discovered test"
        missing = Path(directory) / "missing"
        for script, arguments in (
            ("check_checksum_manifest.py", [str(missing)]),
            ("check_traceability_coverage.py", ["--report", str(missing)]),
            ("build_evidence_envelope.py", [str(missing), "final"]),
            ("verify_evidence_history.py", ["--root", str(missing)]),
        ):
            result = subprocess.run(
                [sys.executable, str(ROOT / "scripts" / script), *arguments],
                cwd=ROOT, check=False, capture_output=True,
            )
            assert result.returncode != 0, f"{script} main accepted a corrupt fixture"

    with temporary_worktree() as checkout:
        attribution = checkout / "docs" / "ATTRIBUTION.md"
        attribution.write_text(
            attribution.read_text(encoding="utf-8") + "\n| `fabricated` | `" + "0" * 64 + "` |\n",
            encoding="utf-8",
        )
        result = subprocess.run(
            ["/usr/bin/python3", "scripts/check_attribution.py"],
            cwd=checkout, check=False, capture_output=True,
        )
        assert result.returncode != 0, "attribution gate main accepted a fabricated file"

    with temporary_worktree() as checkout:
        planted = checkout / "evidence" / f"PLANTED-EXIT-CONTRACT-{os.getpid()}.txt"
        planted.write_text("FABRICATED\n", encoding="utf-8")
        subprocess.run(["/usr/bin/git", "add", str(planted)], cwd=checkout, check=True)
        subprocess.run(
            ["/usr/bin/git", "-c", "user.name=Policy Test", "-c",
             "user.email=policy@example.invalid", "commit", "-qm", "plant fixture"],
            cwd=checkout, check=True,
        )
        shell = subprocess.run(
            ["/usr/bin/bash", "scripts/verify_evidence.sh"], cwd=checkout, check=False,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        assert shell.returncode != 0, "evidence shell verifier exit contract was gutted"

    with temporary_worktree() as checkout:
        assurance = checkout / "spec" / "assurance" / "AA-001.md"
        assured_match = re.search(
            r"^- Record: `(evidence/[^`]+)`\.",
            assurance.read_text(encoding="utf-8"),
            re.MULTILINE,
        )
        assert assured_match is not None
        record_name = Path(assured_match.group(1)).name
        registry_path = checkout / "evidence" / "RETRACTIONS.json"
        mutated_registry = json.loads(registry_path.read_text(encoding="utf-8"))
        mutated_registry["records"][record_name] = {
            "reason": "policy probe must not retract the assurance-bound record"
        }
        registry_path.write_text(
            json.dumps(mutated_registry, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        subprocess.run(
            ["/usr/bin/git", "add", "evidence/RETRACTIONS.json"],
            cwd=checkout,
            check=True,
        )
        subprocess.run(
            ["/usr/bin/git", "-c", "user.name=Policy Test", "-c",
             "user.email=policy@example.invalid", "commit", "-qm", "retract assured fixture"],
            cwd=checkout,
            check=True,
        )
        retracted = subprocess.run(
            ["/usr/bin/bash", "scripts/verify_evidence.sh"],
            cwd=checkout,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
        )
        assert retracted.returncode != 0 and "retracted record" in retracted.stderr, (
            "assurance gate accepted an explicitly retracted bound record"
        )

    with temporary_worktree() as checkout:
        stale = subprocess.run(
            ["/usr/bin/bash", "scripts/verify_evidence.sh"], cwd=checkout, check=False,
            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
        )
        assert stale.returncode != 0 and "older than the reviewed tree" in stale.stderr, (
            "assurance gate accepted a passing pre-remediation source record"
        )

    with temporary_worktree() as checkout:
        assurance = (checkout / "spec" / "assurance" / "AA-001.md").read_text(
            encoding="utf-8"
        )
        assured_match = re.search(
            r"^- Record: `(evidence/[^`]+)`\.", assurance, re.MULTILINE
        )
        assert assured_match is not None
        assured_name = Path(assured_match.group(1)).name
        historical_name = sorted(registry["records"])[0]
        verifier_path = checkout / "scripts" / "verify_evidence.sh"
        verifier_text = verifier_path.read_text(encoding="utf-8").replace(
            "/usr/bin/git diff --quiet", "/usr/bin/true", 1
        )
        verifier_path.write_text(verifier_text, encoding="utf-8")
        (checkout / "scripts" / "evidence_profile.py").write_text(
            "#!/usr/bin/env python3\n"
            "import pathlib, sys\n"
            f"assured = {assured_name!r}\n"
            f"unsupported = {historical_name!r}\n"
            "name = pathlib.Path(sys.argv[1]).name\n"
            "print('v2' if name == assured else "
            "'unsupported-lock-schema' if name == unsupported else 'retracted')\n",
            encoding="utf-8",
        )
        (checkout / "scripts" / "finalize_collection.py").write_text(
            "#!/usr/bin/env python3\nraise SystemExit(0)\n", encoding="utf-8"
        )
        subprocess.run(
            [
                "/usr/bin/git", "add", "scripts/verify_evidence.sh",
                "scripts/evidence_profile.py", "scripts/finalize_collection.py",
            ],
            cwd=checkout, check=True,
        )
        subprocess.run(
            [
                "/usr/bin/git", "-c", "user.name=Policy Test", "-c",
                "user.email=policy@example.invalid", "commit", "-qm",
                "isolate historical profile-state policy",
            ],
            cwd=checkout, check=True,
        )
        historical = subprocess.run(
            ["/usr/bin/bash", "scripts/verify_evidence.sh"],
            cwd=checkout, check=False, capture_output=True, text=True,
        )
        assert historical.returncode == 0, historical.stderr
        assert (
            f"unsupported source tool-lock schema: evidence/{historical_name}"
            in historical.stdout
            and "1 unsupported historical" in historical.stdout
        ), "unsupported historical evidence was not checksummed and counted"

    with temporary_worktree() as checkout:
        (checkout / "DIRTY-POLICY-PROBE").write_text("dirty\n", encoding="utf-8")
        dirty = subprocess.run(
            ["/usr/bin/bash", "scripts/verify_evidence.sh"], cwd=checkout, check=False,
            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
        )
        assert dirty.returncode != 0 and "clean source tree" in dirty.stderr, (
            "evidence verification accepted a dirty candidate tree"
        )
    print("evidence outcome behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
