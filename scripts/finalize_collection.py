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
import evidence_profile
import tool_identity


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
TEST_SUCCESS = re.compile(
    r"^test result: ok\. ([0-9]+) passed; 0 failed; 0 ignored", re.MULTILINE
)
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
        ["/usr/bin/git", "show", f"{revision}:{path.relative_to(builder.ROOT)}"],
        cwd=builder.ROOT, check=True, capture_output=True,
    ).stdout


def positive_ci_census(output: str, require_verify: bool = False) -> bool:
    lines = set(output.splitlines())
    passed = [int(value) for value in TEST_SUCCESS.findall(output)]
    required_signatures = (
        r"all 13 mandatory local-CI targets propagate failures",
        r"all [1-9][0-9]* policy behavior tests passed",
        r"strict traceability coverage is complete: 62/62",
        r"clean-room attribution digests match the pinned Cargo source",
        r"LeakSanitizer enabled",
        r"fmt-check gate passed",
        r"lint gate passed",
        r"Rust test gate passed",
        r"corpus-integrity gate passed",
        r"fuzz-build gate passed",
        r"fuzz-smoke gate passed",
        r"deny gate passed",
        r"audit-unsafe gate passed",
        r"evidence-tool gate passed",
        r"spec gate passed",
        r"msrv gate passed",
        r"rustdoc gate passed",
    )
    return (
        len(passed) >= 8
        and sum(passed) >= 25
        and REQUIRED_CORPUS_LINES <= lines
        and all(re.search(pattern, output) for pattern in required_signatures)
        and (not require_verify or "verify-evidence gate passed" in output)
    )


def qualification_profile(evidence_dir: Path) -> str | None:
    try:
        value = json.loads((evidence_dir / "collection-input.json").read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    profile = value.get("qualificationProfile")
    return profile if isinstance(profile, str) else None


def source_requires_v2(evidence_dir: Path) -> bool:
    try:
        revision = (evidence_dir / "source-revision.txt").read_text(encoding="utf-8").strip()
        return b"tl-parse.evidence-qualification/v2" in git_bytes(revision, builder.FINALIZER)
    except (OSError, subprocess.CalledProcessError):
        return True


def positive_output(evidence_dir: Path, name: str) -> bool:
    stdout = evidence_dir / f"{name}.stdout"
    stderr = evidence_dir / f"{name}.stderr"
    combined = "\n".join(
        path.read_text(encoding="utf-8", errors="replace")
        for path in (stdout, stderr)
        if path.exists()
    )
    if name == "make-ci":
        return positive_ci_census(combined)
    if name == "diff-integrity":
        return True
    if name in {"make-spec", "quire-coverage"}:
        return "strict traceability coverage is complete: 62/62" in combined
    if name == "rustdoc":
        return "Generated " in combined and "/doc/tl_parse/index.html" in combined
    if name == "default-dependencies":
        return "tl-parse v0.1.0" in combined
    if name == "corpus-integrity":
        return REQUIRED_CORPUS_LINES <= set(combined.splitlines())
    if name in {
        "input-schema", "manifest-schema", "pgm01-schema", "pgm01-validator",
        "sealed-pgm01-schema", "sealed-pgm01-validator",
    }:
        return bool(re.search(r'"errors"\s*:\s*\[\]\s*,?\s*"valid"\s*:\s*true', combined))
    return False


def summary(evidence_dir: Path) -> dict[str, object]:
    profile_state = evidence_profile.resolve_profile(evidence_dir)
    if profile_state != "v2":
        raise ValueError("active evidence lacks the qualified v2 profile and source tool lock")
    profile = qualification_profile(evidence_dir)
    require_positive = True
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
            and require_positive
            and not positive_output(evidence_dir, name)
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
    if profile != "tl-parse.evidence-qualification/v2":
        statuses.add("failed")
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


def historical_parameters_digest(revision: str, source_builder: bytes) -> str:
    tree = set(
        subprocess.run(
            ["/usr/bin/git", "ls-tree", "-r", "--name-only", revision],
            cwd=builder.ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    )
    start = (
        source_builder.find(b"def parameter_paths")
        if b"def parameter_paths" in source_builder
        else source_builder.find(b"def parameters_digest")
    )
    if start < 0:
        raise OSError("historical builder has no parameter digest function")
    end = source_builder.find(b"\ndef build", start)
    function = source_builder[start:end]
    ordered = [
        (b'ROOT / "Cargo.toml"', "Cargo.toml"),
        (b'ROOT / "Cargo.lock"', "Cargo.lock"),
        (b'ROOT / "Makefile"', "Makefile"),
        (b'ROOT / "deny.toml"', "deny.toml"),
        (b'ROOT / "rust-toolchain.toml"', "rust-toolchain.toml"),
        (b"TOOLS_LOCK", "tools.lock"),
        (b'ROOT / ".github" / "workflows" / "ci.yml"', ".github/workflows/ci.yml"),
        (b'ROOT / "src" / "diagnostic.rs"', "src/diagnostic.rs"),
        (b'ROOT / "src" / "format.rs"', "src/format.rs"),
        (b'ROOT / "src" / "lib.rs"', "src/lib.rs"),
        (b'ROOT / "src" / "lexer.rs"', "src/lexer.rs"),
        (b'ROOT / "src" / "parser.rs"', "src/parser.rs"),
        (b'ROOT / "src" / "bin" / "tl-parse.rs"', "src/bin/tl-parse.rs"),
        (b'ROOT / "docs" / "DIALECT-001-clean-room-mltl-v1.md"', "docs/DIALECT-001-clean-room-mltl-v1.md"),
        (b'ROOT / "docs" / "ATTRIBUTION.md"', "docs/ATTRIBUTION.md"),
        (b'ROOT / "corpus" / "README.md"', "corpus/README.md"),
        (b'ROOT / "corpus" / "v1" / "SHA256SUMS"', "corpus/v1/SHA256SUMS"),
        (b'ROOT / "corpus" / "v1" / "manifest.json"', "corpus/v1/manifest.json"),
        (b'ROOT / "fuzz" / "README.md"', "fuzz/README.md"),
        (b'ROOT / "fuzz" / "Cargo.toml"', "fuzz/Cargo.toml"),
        (b'ROOT / "fuzz" / "Cargo.lock"', "fuzz/Cargo.lock"),
        (b'ROOT / "fuzz" / "corpus" / "parser" / "SHA256SUMS"', "fuzz/corpus/parser/SHA256SUMS"),
        (b'ROOT / "fuzz" / "fuzz_targets" / "parser.rs"', "fuzz/fuzz_targets/parser.rs"),
        (b'ROOT / "scripts" / "run_fuzz_smoke.sh"', "scripts/run_fuzz_smoke.sh"),
        (b'ROOT / "evidence" / "README.md"', "evidence/README.md"),
        (b"COLLECTOR", "scripts/collect_evidence.sh"),
        (b"BUILDER", "scripts/build_evidence_envelope.py"),
        (b"VALIDATOR", "scripts/validate_json_schema.py"),
        (b"FINALIZER", "scripts/finalize_collection.py"),
        (b"EVIDENCE_VERIFIER", "scripts/verify_evidence_manifest.py"),
        (b"TRACEABILITY_VALIDATOR", "scripts/check_traceability_coverage.py"),
        (b"EVIDENCE_SHELL_VERIFIER", "scripts/verify_evidence.sh"),
        (b"EVIDENCE_ANCHORS", "evidence/ANCHORS"),
        (b"EVIDENCE_RETRACTIONS", "evidence/RETRACTIONS.json"),
        (b"ASSURANCE_ARGUMENT", "spec/assurance/AA-001.md"),
        (b"INPUT_SCHEMA", "schemas/tl-parse-evidence-input-v1.schema.json"),
        (b"MANIFEST_SCHEMA", "schemas/tl-parse-evidence-manifest-v1.schema.json"),
    ]
    paths = [path for marker, path in ordered if marker in function]
    if b'(ROOT / "scripts").iterdir()' in function:
        paths = list(set(paths) | {
            path for path in tree
            if path.startswith("scripts/") and Path(path).suffix in {".py", ".sh"}
        })
    if b"fixed_paths = {" in function or b"def parameter_paths" in function:
        paths = sorted(set(paths))
    missing = set(paths) - tree
    if missing:
        raise OSError(f"source revision lacks parameter paths: {sorted(missing)}")
    state = hashlib.sha256()
    for relative in paths:
        state.update(relative.encode())
        state.update(b"\0")
        state.update(git_bytes(revision, builder.ROOT / relative))
        state.update(b"\0")
    return state.hexdigest()


def validate_tool_identity(evidence_dir: Path, revision: str | None) -> list[str]:
    if not revision:
        return ["retained tool identity has no source revision"]
    try:
        lock_value = json.loads(git_bytes(revision, builder.TOOLS_LOCK))
        locked_tools = tool_identity.validate_lock(lock_value)
        collection = json.loads(
            (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
        )
        retained = collection["tools"]["identities"]
    except (KeyError, OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        return [f"cannot rederive retained tool identity: {error}"]
    return [] if retained == locked_tools else ["retained tool identities disagree with source tools.lock"]


def validate_parameter_identity(
    evidence_dir: Path, envelope: dict[str, object], revision: str | None
) -> list[str]:
    if not revision:
        return [f"retained parameter identity has no source revision: {evidence_dir}"]
    try:
        source_builder = git_bytes(revision, builder.BUILDER)
        expected = historical_parameters_digest(revision, source_builder)
    except (OSError, subprocess.CalledProcessError) as error:
        return [f"cannot rederive retained parameter identity: {error}"]
    if envelope.get("parametersDigest", {}).get("value") != expected:
        return ["envelope parameters digest disagrees with the source revision"]
    return []


def validate_envelope_result(evidence_dir: Path, value: dict[str, object]) -> list[str]:
    try:
        envelope = json.loads((evidence_dir / "evidence-envelope.json").read_text(encoding="utf-8"))
        actual = envelope["result"]["status"]
        revision_path = evidence_dir / "source-revision.txt"
        revision = revision_path.read_text(encoding="utf-8").strip() if revision_path.exists() else None
    except (KeyError, OSError, json.JSONDecodeError) as error:
        return [f"cannot derive retained envelope result: {error}"]
    outcomes = value["outcomes"]
    if not isinstance(outcomes, list):
        return ["collection summary outcomes are not a list"]
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
    errors.extend(validate_parameter_identity(evidence_dir, envelope, revision))
    errors.extend(validate_tool_identity(evidence_dir, revision))
    return errors


def main() -> int:
    check = len(sys.argv) == 3 and sys.argv[1] == "--check"
    if len(sys.argv) != 2 and not check:
        print("usage: finalize_collection.py [--check] EVIDENCE_DIR", file=sys.stderr)
        return 2
    evidence_dir = Path(sys.argv[2] if check else sys.argv[1])
    try:
        profile = evidence_profile.resolve_profile(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"cannot resolve evidence qualification profile: {error}", file=sys.stderr)
        return 2
    if profile == "retracted":
        if not check:
            print(f"refusing to rewrite explicitly retracted evidence: {evidence_dir}", file=sys.stderr)
            return 2
        print(f"retained evidence is explicitly retracted: {evidence_dir}")
        return 0
    if profile != "v2":
        print(f"active evidence is inconclusive without qualification-v2: {evidence_dir}", file=sys.stderr)
        return 1
    try:
        value = summary(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"cannot derive retained collection summary: {error}", file=sys.stderr)
        return 2
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
