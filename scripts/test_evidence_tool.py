#!/usr/bin/env python3
"""Behavior tests for evidence outcome classification."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
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


def healthy_ci_output() -> str:
    tests = "test result: ok. 1 passed; 0 failed; 0 ignored\n" * 8
    corpus = "\n".join(sorted(FINALIZER.REQUIRED_CORPUS_LINES)) + "\n"
    return tests + corpus


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
    collector = (ROOT / "scripts" / "collect_evidence.sh").read_text(encoding="utf-8")
    for retained in (
        "make-ci", "make-spec", "quire-coverage", "msrv", "rustdoc",
        "default-dependencies", "corpus-integrity",
    ):
        line = next(
            (item for item in collector.splitlines() if item.startswith(f"run_and_retain {retained} ")),
            "",
        )
        assert '"${clean_env[@]}"' in line, f"{retained} bypasses the clean environment"
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
            json.dumps({"result": {"status": "inconclusive"}}) + "\n", encoding="utf-8"
        )
        for name in FINALIZER.CHECKS:
            (evidence_dir / f"{name}.status.txt").write_text("0\n", encoding="utf-8")
        retained = FINALIZER.summary(evidence_dir)
        assert retained["overallStatus"] == "passed"
        assert FINALIZER.positive_ci_census(healthy_ci_output())
        assert not FINALIZER.positive_ci_census("cargo 1.94.1\n")
        assert FINALIZER.validate_envelope_result(evidence_dir, retained) == []
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

    planted = ROOT / "evidence" / f"PLANTED-EXIT-CONTRACT-{os.getpid()}.txt"
    planted.write_text("FABRICATED\n", encoding="utf-8")
    try:
        shell = subprocess.run(
            ["bash", "scripts/verify_evidence.sh"], cwd=ROOT, check=False,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        assert shell.returncode != 0, "evidence shell verifier exit contract was gutted"
    finally:
        planted.unlink(missing_ok=True)
    print("evidence outcome behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
