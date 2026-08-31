#!/usr/bin/env python3
"""Write the post-envelope validation summary for a retained collection."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

import build_evidence_envelope as builder


CHECKS = (
    "make-ci",
    "make-spec",
    "quire-coverage",
    "rustdoc",
    "default-dependencies",
    "corpus-integrity",
    "diff-integrity",
    "input-schema",
    "manifest-schema",
    "pgm01-schema",
    "pgm01-validator",
    "sealed-pgm01-schema",
    "sealed-pgm01-validator",
)
CONTRADICTION = re.compile(
    r"test result: FAILED|Error [0-9]+ \(ignored\)|\b[1-9][0-9]* ignored\b|"
    r"LeakSanitizer explicitly disabled"
)
TEST_SUCCESS = re.compile(r"^test result: ok\.", re.MULTILINE)
REQUIRED_CORPUS_LINES = {
    f"corpus/v1/{name}: OK"
    for name in (
        "depth-limit.txt", "inverted-interval.txt", "leading-zero.txt",
        "missing-delimiter.txt", "token-limit.txt", "unexpected-unicode.txt",
        "valid-canonical.txt", "manifest.json",
    )
} | {
    f"fuzz/corpus/parser/{name}: OK"
    for name in ("invalid.txt", "resource.txt", "roundtrip-depth.txt", "valid.txt")
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_bytes(revision: str, path: Path) -> bytes:
    return subprocess.run(
        ["git", "show", f"{revision}:{path.relative_to(builder.ROOT)}"],
        cwd=builder.ROOT, check=True, capture_output=True,
    ).stdout


def source_has(evidence_dir: Path, marker: bytes) -> bool:
    source = evidence_dir / "source-revision.txt"
    if not source.exists():
        return True
    try:
        return marker in git_bytes(source.read_text(encoding="utf-8").strip(), builder.BUILDER)
    except (OSError, subprocess.CalledProcessError):
        return False


def positive_ci_census(output: str) -> bool:
    lines = set(output.splitlines())
    return len(TEST_SUCCESS.findall(output)) >= 8 and REQUIRED_CORPUS_LINES <= lines


def summary(evidence_dir: Path) -> dict[str, object]:
    outcomes = []
    observed = {
        path.name[: -len(".status.txt")]
        for path in evidence_dir.glob("*.status.txt")
        if path.is_file()
    }
    for name in list(CHECKS) + sorted(observed - set(CHECKS)):
        status_path = evidence_dir / f"{name}.status.txt"
        if not status_path.exists():
            outcomes.append({"name": name, "status": "inconclusive", "exitCode": None})
            continue
        exit_code = int(status_path.read_text(encoding="utf-8").strip())
        skipped = exit_code == 125
        stderr_path = evidence_dir / f"{name}.stderr"
        validator_contradiction = (
            exit_code == 0
            and name in {"pgm01-validator", "sealed-pgm01-validator"}
            and stderr_path.exists()
            and bool(stderr_path.read_text(encoding="utf-8").strip())
        )
        output_contradiction = any(
            path.exists()
            and CONTRADICTION.search(path.read_text(encoding="utf-8", errors="replace"))
            for path in (
                evidence_dir / f"{name}.stdout",
                evidence_dir / f"{name}.stderr",
            )
        )
        positive_census_missing = (
            exit_code == 0
            and name == "make-ci"
            and source_has(evidence_dir, b"REQUIRED_CORPUS_LINES")
            and not positive_ci_census(
                (evidence_dir / "make-ci.stdout").read_text(
                    encoding="utf-8", errors="replace"
                )
                if (evidence_dir / "make-ci.stdout").exists()
                else ""
            )
        )
        outcomes.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if skipped
                    else "failed"
                    if validator_contradiction or output_contradiction or positive_census_missing
                    else "passed"
                    if exit_code == 0
                    else "failed"
                ),
                "exitCode": exit_code,
            }
        )
    statuses = {item["status"] for item in outcomes}
    if "failed" in statuses:
        overall = "failed"
    elif "skipped-unavailable" in statuses or "inconclusive" in statuses:
        overall = "inconclusive"
    else:
        overall = "passed"
    envelope = evidence_dir / "evidence-envelope.json"
    return {
        "schemaVersion": "tl-parse.collection-summary/v1",
        "overallStatus": overall,
        "finalEnvelopeSha256": sha256(envelope),
        "finalEnvelopeValidated": all(
            item["status"] == "passed"
            for item in outcomes
            if item["name"].startswith("sealed-")
        ),
        "outcomes": outcomes,
    }


def validate_envelope_result(evidence_dir: Path, value: dict[str, object]) -> list[str]:
    try:
        envelope = json.loads((evidence_dir / "evidence-envelope.json").read_text(encoding="utf-8"))
        actual = envelope["result"]["status"]
        revision_path = evidence_dir / "source-revision.txt"
        revision = revision_path.read_text(encoding="utf-8").strip() if revision_path.exists() else None
    except (KeyError, OSError, json.JSONDecodeError) as error:
        return [f"cannot derive retained envelope result: {error}"]
    outcomes = value["outcomes"]
    assert isinstance(outcomes, list)
    sealed_not_passed = any(
        item["name"].startswith("sealed-") and item["status"] != "passed"
        for item in outcomes
    )
    expected = (
        "error"
        if value["overallStatus"] == "failed" or sealed_not_passed
        else "inconclusive"
    )
    errors = [] if actual == expected else [f"envelope result {actual!r} disagrees with {expected!r}"]
    try:
        source_builder = git_bytes(revision, builder.BUILDER) if revision else b""
        if revision and b"def parameter_paths" in source_builder:
            digest = builder.parameters_digest(lambda path: git_bytes(revision, path))
            if envelope.get("parametersDigest", {}).get("value") != digest:
                errors.append("envelope parameters digest disagrees with the source revision")
    except (OSError, subprocess.CalledProcessError) as error:
        errors.append(f"cannot rederive retained parameter identity: {error}")
    return errors


def main() -> int:
    check = len(sys.argv) == 3 and sys.argv[1] == "--check"
    if len(sys.argv) != 2 and not check:
        print("usage: finalize_collection.py [--check] EVIDENCE_DIR", file=sys.stderr)
        return 2
    evidence_dir = Path(sys.argv[2] if check else sys.argv[1])
    value = summary(evidence_dir)
    envelope_errors = validate_envelope_result(evidence_dir, value)
    if envelope_errors:
        for error in envelope_errors:
            print(error, file=sys.stderr)
        return 1
    summary_path = evidence_dir / "collection-summary.json"
    if check:
        actual = json.loads(summary_path.read_text(encoding="utf-8"))
        if actual != value:
            print(f"retained summary disagrees with status files: {evidence_dir}", file=sys.stderr)
            return 1
        return 0
    summary_path.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
