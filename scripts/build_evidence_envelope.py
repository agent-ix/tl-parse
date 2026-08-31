#!/usr/bin/env python3
"""Build tl-parse's PGM-01 collection input, manifest, and envelope."""

from __future__ import annotations

import datetime as dt
import hashlib
import json
import platform
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
PGM01_POLICY_REVISION = "7dac9d8c19952412b56a0347387666e2ca81e01d"
PGM01_SCHEMA_DIGEST = "0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256"
TL_SYNTAX_REVISION = "740182f13b84858008d6f176f75136737d405c1b"
DIALECT_RECORD_DIGEST = "22959d4df6c7a1230172289903f1c31f36859b6f2a0e4556e886bdb7ebc9ae11"
INPUT_SCHEMA = ROOT / "schemas" / "tl-parse-evidence-input-v1.schema.json"
MANIFEST_SCHEMA = ROOT / "schemas" / "tl-parse-evidence-manifest-v1.schema.json"
TOOLS_LOCK = ROOT / "tools.lock"
COLLECTOR = ROOT / "scripts" / "collect_evidence.sh"
BUILDER = Path(__file__).resolve()
VALIDATOR = ROOT / "scripts" / "validate_json_schema.py"
FINALIZER = ROOT / "scripts" / "finalize_collection.py"
EVIDENCE_VERIFIER = ROOT / "scripts" / "verify_evidence_manifest.py"
TRACEABILITY_VALIDATOR = ROOT / "scripts" / "check_traceability_coverage.py"
EVIDENCE_SHELL_VERIFIER = ROOT / "scripts" / "verify_evidence.sh"
EVIDENCE_RETRACTIONS = ROOT / "evidence" / "RETRACTIONS.json"
COMMANDS = (
    "make-ci",
    "make-spec",
    "quire-coverage",
    "msrv",
    "rustdoc",
    "default-dependencies",
    "corpus-integrity",
    "diff-integrity",
    "input-schema",
    "manifest-schema",
    "pgm01-schema",
    "pgm01-validator",
)


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(value: str) -> dict[str, str]:
    return {"algorithm": "sha256", "value": value}


def schema_identity(name: str, path: Path) -> dict[str, object]:
    return {"id": name, "version": "v1", "digest": digest(sha256_file(path))}


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def first_line(path: Path) -> str:
    return path.read_text(encoding="utf-8").splitlines()[0]


def command_outcomes(directory: Path) -> list[dict[str, object]]:
    values = []
    for name in COMMANDS:
        status_path = directory / f"{name}.status.txt"
        if not status_path.exists():
            values.append({"name": name, "status": "inconclusive", "exitCode": None})
            continue
        code = int(status_path.read_text().strip())
        skipped = code == 125
        values.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if skipped
                    else "passed" if code == 0 else "failed"
                ),
                "exitCode": code,
            }
        )
    return values


def classify_result(
    phase: str, outcomes: list[dict[str, object]]
) -> tuple[str, str]:
    statuses = {outcome["status"] for outcome in outcomes}
    if phase == "sealed-failed" or "failed" in statuses:
        return "error", "one or more retained tl-parse checks failed"
    if phase in {"provisional", "final"}:
        return "inconclusive", "exact finalized-envelope validation is external or pending"
    if "inconclusive" in statuses or "skipped-unavailable" in statuses:
        return "inconclusive", "schema or governance validation is unavailable or pending"
    return "inconclusive", "unrecognized collection phase cannot be conclusive"


def parameter_paths() -> tuple[Path, ...]:
    fixed_paths = {
        ROOT / "Cargo.toml",
        ROOT / "Cargo.lock",
        ROOT / "Makefile",
        ROOT / "deny.toml",
        ROOT / "rust-toolchain.toml",
        TOOLS_LOCK,
        ROOT / ".github" / "workflows" / "ci.yml",
        ROOT / "src" / "diagnostic.rs",
        ROOT / "src" / "format.rs",
        ROOT / "src" / "lib.rs",
        ROOT / "src" / "lexer.rs",
        ROOT / "src" / "parser.rs",
        ROOT / "src" / "bin" / "tl-parse.rs",
        ROOT / "docs" / "DIALECT-001-clean-room-mltl-v1.md",
        ROOT / "docs" / "ATTRIBUTION.md",
        ROOT / "corpus" / "README.md",
        ROOT / "corpus" / "v1" / "SHA256SUMS",
        ROOT / "corpus" / "v1" / "manifest.json",
        ROOT / "fuzz" / "README.md",
        ROOT / "fuzz" / "Cargo.toml",
        ROOT / "fuzz" / "Cargo.lock",
        ROOT / "fuzz" / "corpus" / "parser" / "SHA256SUMS",
        ROOT / "fuzz" / "fuzz_targets" / "parser.rs",
        ROOT / "scripts" / "run_fuzz_smoke.sh",
        ROOT / "evidence" / "README.md",
        COLLECTOR,
        BUILDER,
        VALIDATOR,
        FINALIZER,
        EVIDENCE_VERIFIER,
        TRACEABILITY_VALIDATOR,
        EVIDENCE_SHELL_VERIFIER,
        EVIDENCE_RETRACTIONS,
        INPUT_SCHEMA,
        MANIFEST_SCHEMA,
    }
    paths = fixed_paths | {
        path
        for path in (ROOT / "scripts").iterdir()
        if path.is_file() and path.suffix in {".py", ".sh"}
    }
    return tuple(sorted(paths, key=lambda path: str(path.relative_to(ROOT))))


def parameters_digest(read_bytes: Any | None = None) -> str:
    reader = read_bytes or (lambda path: path.read_bytes())
    state = hashlib.sha256()
    for path in parameter_paths():
        state.update(str(path.relative_to(ROOT)).encode())
        state.update(b"\0")
        state.update(reader(path))
        state.update(b"\0")
    return state.hexdigest()


def build(directory: Path, phase: str) -> None:
    directory = directory.resolve()
    relative = str(directory.relative_to(ROOT))
    revision = (directory / "source-revision.txt").read_text().strip()
    metadata = json.loads((directory / "metadata.json").read_text())
    package = next(item for item in metadata["packages"] if item["name"] == "tl-parse")
    recorded_at = dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    quire_version = json.loads((directory / "quire-provenance.json").read_text())["cli"]["version"]

    collection_input = {
        "schemaVersion": "tl-parse.evidence-input/v1",
        "qualificationProfile": "tl-parse.evidence-qualification/v2",
        "sourceRevision": revision,
        "sourceState": (directory / "source-state.txt").read_text().strip(),
        "commands": [
            "make ci-for-evidence (all candidate gates; final make ci adds the AA-001 self-binding)",
            "make spec",
            "python3 scripts/check_traceability_coverage.py",
            "RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features",
            "cargo tree --no-default-features --edges normal",
            "make check-corpus",
            f"git diff --check origin/main...{revision} -- . :(exclude)evidence/**",
            "validate local evidence schemas and exact merged PGM-01 envelope",
            "python3 scripts/build_evidence_envelope.py EVIDENCE_DIR final",
            "validate exact finalized PGM-01 envelope",
            "python3 scripts/finalize_collection.py EVIDENCE_DIR",
        ],
        "tools": {
            "cargo": first_line(directory / "cargo-version.txt"),
            "jsonschema": (directory / "jsonschema-version.txt").read_text().strip(),
            "python": (directory / "python-version.txt").read_text().strip(),
            "quire": quire_version,
            "rustc": first_line(directory / "rustc-version.txt"),
            "identities": {
                name: {
                    "path": (directory / f"tool-{name}-path.txt").read_text().strip(),
                    "sha256": (directory / f"tool-{name}-sha256.txt").read_text().strip(),
                }
                for name in ("bash", "cargo", "git", "make", "python3", "quire", "rustc", "sha256sum")
            },
        },
        "pgm01": {
            "policy": "ix://agent-ix/quire-contract-ir/PGM-01",
            "candidateRevision": PGM01_POLICY_REVISION,
            "envelopeSchema": "quire.derivation-evidence/v1",
            "envelopeSchemaDigest": digest(PGM01_SCHEMA_DIGEST),
        },
        "dependency": {
            "tlSyntaxRevision": TL_SYNTAX_REVISION,
            "cargoLockDigest": digest(sha256_file(ROOT / "Cargo.lock")),
        },
        "dialect": {
            "revision": "tl-parse.clean-ascii/v1",
            "recordDigest": digest(DIALECT_RECORD_DIGEST),
            "documentDigest": digest(
                sha256_file(ROOT / "docs" / "DIALECT-001-clean-room-mltl-v1.md")
            ),
        },
        "corpus": {
            "revision": "tl-parse-corpus/v1",
            "manifestDigest": digest(sha256_file(ROOT / "corpus" / "v1" / "manifest.json")),
            "checksumDigest": digest(sha256_file(ROOT / "corpus" / "v1" / "SHA256SUMS")),
            "fuzzChecksumDigest": digest(
                sha256_file(ROOT / "fuzz" / "corpus" / "parser" / "SHA256SUMS")
            ),
        },
        "limits": {
            "sourceBytes": 1_048_576,
            "tokens": 100_000,
            "nodes": 10_000,
            "depth": 256,
            "diagnostics": 64,
            "parseWork": 1_000_000,
            "outputBytes": 1_048_576,
            "formatWork": 4_194_304,
        },
    }
    input_path = directory / "collection-input.json"
    write_json(input_path, collection_input)

    excluded = {
        "collection-input.json",
        "evidence-envelope.json",
        "evidence-manifest.json",
        "collection-summary.json",
    }
    artifacts = [
        {"path": path.name, "sha256": sha256_file(path), "size": path.stat().st_size}
        for path in sorted(directory.iterdir(), key=lambda item: item.name)
        if path.is_file() and path.name not in excluded
    ]
    outcomes = command_outcomes(directory)
    any_failed = any(item["status"] == "failed" for item in outcomes)
    any_inconclusive = any(
        item["status"] in {"inconclusive", "skipped-unavailable"} for item in outcomes
    )
    limitations = [
        "manual-dispatch remote CI is not part of this local envelope",
        "generated and fuzz populations are bounded evidence rather than proof over every bounded string",
        "the crate constructs syntax graphs but does not establish temporal semantic correctness",
        "independent human approval and the source-release decision remain pending",
    ]
    if any_failed:
        limitations.append("one or more locally collected commands failed")
    if any_inconclusive:
        limitations.append("one or more schema or governance checks were unavailable or pending")
    if phase == "provisional":
        limitations.append("this provisional envelope precedes its own schema and governance checks")
    if phase == "final":
        limitations.append("the exact finalized envelope is validated externally and does not self-attest")
    if phase == "sealed-failed":
        limitations.append("validation of the finalized envelope failed; see sealed validation artifacts")
    manifest = {
        "schemaVersion": "tl-parse.evidence-manifest/v1",
        "sourceRevision": revision,
        "collectedAt": recorded_at,
        "outcomes": outcomes,
        "artifacts": artifacts,
        "limitations": limitations,
    }
    manifest_path = directory / "evidence-manifest.json"
    write_json(manifest_path, manifest)

    host = next(
        line.split(": ", 1)[1]
        for line in (directory / "rustc-version.txt").read_text().splitlines()
        if line.startswith("host: ")
    )
    result_status, result_summary = classify_result(phase, outcomes)
    envelope = {
        "schemaVersion": "quire.derivation-evidence/v1",
        "recordId": directory.name,
        "recordedAt": recorded_at,
        "producer": {
            "name": "tl-parse-evidence-collector",
            "version": package["version"],
            "sourceRevision": revision,
            "executableDigest": digest(sha256_file(COLLECTOR)),
            "invocation": ["bash", "scripts/collect_evidence.sh", relative],
        },
        "inputs": [{
            "role": "evidence-collection-input", "uri": "collection-input.json",
            "mediaType": "application/json",
            "schema": schema_identity("tl-parse.evidence-input", INPUT_SCHEMA),
            "contentDigest": digest(sha256_file(input_path)),
        }],
        "backend": {"kind": "none", "reason": "deterministic packaging; invoked tools are identified in the input"},
        "outputs": [{
            "role": "tl-parse-evidence-manifest", "uri": "evidence-manifest.json",
            "mediaType": "application/json",
            "schema": schema_identity("tl-parse.evidence-manifest", MANIFEST_SCHEMA),
            "contentDigest": digest(sha256_file(manifest_path)),
        }],
        "parametersDigest": digest(parameters_digest()),
        "environment": {
            "targetTriple": host,
            "operatingSystem": platform.platform(),
            "toolchain": collection_input["tools"]["rustc"],
            "dependenciesDigest": digest(sha256_file(ROOT / "Cargo.lock")),
        },
        "provenance": {
            "repository": "https://github.com/agent-ix/tl-parse",
            "sourceRevision": revision,
            "candidateRevision": revision,
            "contributionMethod": "agent-assisted",
            "reviewers": ["@kreneskyp"],
        },
        "result": {
            "status": result_status,
            "summary": result_summary,
            "requirementRefs": ["PGM-01-R08", "PGM-01-R09", "MP-001"],
        },
        "extensions": {"dev.agent-ix.tl-parse": {
            "componentClass": "text-boundary-tool",
            "dialectRevision": "tl-parse.clean-ascii/v1",
            "corpusRevision": "tl-parse-corpus/v1",
            "envelopeSchemaDigest": PGM01_SCHEMA_DIGEST,
            "pgm01CandidateRevision": PGM01_POLICY_REVISION,
            "reviewState": "pending",
            "sourceState": "clean",
        }},
    }
    write_json(directory / "evidence-envelope.json", envelope)


def main() -> int:
    if len(sys.argv) not in {2, 3}:
        print("usage: build_evidence_envelope.py EVIDENCE_DIR [PHASE]", file=sys.stderr)
        return 2
    phase = sys.argv[2] if len(sys.argv) == 3 else "final"
    if phase not in {"provisional", "final", "sealed-failed"}:
        print(f"unknown evidence build phase: {phase}", file=sys.stderr)
        return 2
    build(Path(sys.argv[1]), phase)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
