#!/usr/bin/env python3
"""Drive the official change-assurance chain over already-produced results (FR-006).

Four things this file deliberately is not.

It is not a producer. It never runs a parser, a formatter, a corpus replay, a
compiler, or a fuzzer. Every input it reads was written by
`make assurance-inputs`, and if one is absent it says so and names that target.
A driver that can produce its own inputs is a driver that can produce a green run
out of nothing.

It is not an envelope. Quoin's packaged FR-063 record, FR-064 attestation and
FR-065 receipt schemas are the shapes. This file projects
`assurance/change-assurance.json` into the record body Quoin requires and derives
nothing beyond the digests that file's own `derived_fields` names.

It is not a verdict. It runs `quoin` and reports what `quoin` said. Where a
scenario expects a refusal, the refusal is the expected result and the run is
green because the tool refused, not because the tool agreed.

It is not a retention store. Nothing is written under `evidence/`, nothing is
committed, and the Quoin store it uses lives under `target/`, which is ignored.

Exit status: 0 when every scenario, control and probe matched, 1 when one did
not, 2 on a usage or environment error — which is a different fact from a
mismatch and gets its own code.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
DECLARATION = ROOT / "assurance" / "change-assurance.json"
ASSURANCE_DIR = ROOT / "target" / "assurance"
STORE = ROOT / "target" / "assurance-store"
CORPUS_MANIFEST = ROOT / "corpus" / "v1" / "manifest.json"

CONFORMANCE_PROTOCOL = "tl-parse.parser-conformance/v1"
ROUNDTRIP_PROTOCOL = "tl-parse.roundtrip-sweep/v1"

# Every proof obligation's retained result, and the media type its producer
# declares. Stated rather than sniffed, because a producer's content type is
# part of what it produced.
INPUTS = {
    "PROOF-parser-conformance": ("parser-conformance.jsonl", "application/x-ndjson"),
    "PROOF-roundtrip-property": ("roundtrip-property.jsonl", "application/x-ndjson"),
    "PROOF-test-census": ("test-census.json", "application/json"),
    "PROOF-clean-room-attribution": ("attribution.json", "application/json"),
    "PROOF-quire-static-export": ("quire-static-export.json", "application/json"),
    "PROOF-legacy-compatibility": ("legacy-compatibility.json", "application/json"),
    "PROOF-msrv": ("msrv.jsonl", "application/x-ndjson"),
}

# The outcome vocabulary a producer's rows may use, and the attestation result
# each maps to. Every value is listed; an unlisted one is refused.
#
# `malformed` maps to `passed`, and this is the one place tl-parse deliberately
# diverges from tl-syntax's identical table, so it is worth being explicit about
# why rather than leaving a reader to assume it was copied wrong.
#
# In tl-syntax, `malformed` meant "the producer's own row was malformed", which
# is a defect and rightly `failed`. In tl-parse it means something else: the
# INPUT FIXTURE was malformed and the parser reported it as malformed, which is
# the parser doing exactly its job. Six of this repository's seven corpus
# fixtures are malformed by design, so mapping that to `failed` would report a
# permanently failing proof for a permanently correct parser.
#
# The state is not thereby collapsed into `pass`. The producer's rows retain the
# word `malformed`, Quoin retains those bytes exactly, and three separate
# scenarios below assert that: the malformed rows exist, their count equals the
# number of fixtures the corpus manifest declares malformed, and the word
# survives into the retained bytes. A producer that silently accepted every
# malformed input would report `pass` rows instead and fail all three.
#
# A producer row whose SHAPE cannot be read is not an outcome at all — it raises,
# below, rather than being given a state.
ROW_RESULTS = {
    "pass": "passed",
    "malformed": "passed",
    "fail": "failed",
    "unavailable": "unavailable",
    "not-computed": "not_computed",
    "vacuous": "not_computed",
}

# Precedence when a stream carries more than one outcome. A single failure
# outranks any number of passes, and an unavailable outranks a not-computed,
# because the strongest thing observed is what the run has to be reported as.
RESULT_PRECEDENCE = ("failed", "unavailable", "not_computed", "passed")


class ChainError(RuntimeError):
    """The chain could not be driven. Distinct from a scenario that did not match."""


def digest_of(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def quoin(*arguments: str, stdin: str | None = None) -> subprocess.CompletedProcess[str]:
    """Invoke the pinned Quoin CLI. It is the only command this file runs."""
    if shutil.which("quoin") is None:
        raise ChainError("quoin is not on PATH; the pinned CLI is required")
    return subprocess.run(
        ["quoin", *arguments], input=stdin, capture_output=True, text=True, check=False
    )


def tool_version(argv: list[str]) -> str | None:
    """Observe a tool's version, or report that it could not be observed.

    `None` is the answer when the probe failed, and it is recorded as `null`
    rather than replaced with a plausible-looking default. A fabricated version
    in a sealed attestation's environment is worse than an absent one, because a
    reader cannot tell it apart from a real observation.
    """
    try:
        result = subprocess.run(argv, capture_output=True, text=True, check=False)
    except OSError:
        return None
    if result.returncode != 0:
        return None
    return result.stdout.strip() or None


SEMVER = re.compile(r"\b(\d+\.\d+\.\d+)\b")


def semantic_version(text: str | None) -> str | None:
    """Extract the version the attestation schema's `immutable_version` accepts."""
    if text is None:
        return None
    found = SEMVER.search(text)
    return found.group(1) if found else None


def observe_environment() -> dict[str, Any]:
    quire_version: str | None = None
    try:
        raw = subprocess.run(["quire", "provenance"], capture_output=True, text=True, check=False)
    except OSError:
        # An absent tool is an unobserved tool, recorded as null. It is not a
        # crash, and it is certainly not a version.
        raw = None
    if raw is not None and raw.returncode == 0:
        try:
            provenance = json.loads(raw.stdout)
            quire_version = (
                f"{provenance['cli']['version']} engine {provenance['engine']['version']}"
            )
        except (json.JSONDecodeError, KeyError, TypeError):
            quire_version = None
    return {
        "quoin": tool_version(["quoin", "--version"]),
        "quire": quire_version,
        "ix-flow": tool_version(["ix-flow", "--version"]),
        "rustc": tool_version(["rustc", "--version"]),
        "platform": sys.platform,
    }


# ---------------------------------------------------------------------------
# The native adapter
# ---------------------------------------------------------------------------

# The domain stream's outcome vocabulary, and the Quoin entry outcome each one
# transcribes to. Every value is listed. An outcome this table does not name is
# refused rather than defaulted, because a silently defaulted unknown state is
# how twelve states become two.
#
# Quoin's normalized entry vocabulary is three-valued by design; the twelve
# states live in the producer's own structured result, which Quoin retains byte
# for byte. `malformed` transcribes to `pass` because the OBLIGATION — malformed
# input is reported as malformed — was discharged, not because the input passed.
CONFORMANCE_OUTCOMES = {
    "pass": "pass",
    "malformed": "pass",
    "fail": "fail",
    "unavailable": "skip",
    "not-computed": "skip",
    "vacuous": "skip",
}


def adapt_conformance(raw: str, protocol: str = CONFORMANCE_PROTOCOL) -> dict[str, Any]:
    """Transcribe the declared domain protocol into Quoin's normalized entries.

    This is the whole of the adapter. It reads a protocol it names, maps a state
    vocabulary it enumerates, and refuses anything else. It runs nothing, judges
    nothing, and never looks at a process's output stream to decide an outcome.
    """
    entries = []
    lines = [line for line in raw.splitlines() if line.strip()]
    if not lines:
        raise ChainError("the conformance stream is empty; there is nothing to transcribe")
    for number, line in enumerate(lines, start=1):
        try:
            row = json.loads(line)
        except json.JSONDecodeError as error:
            raise ChainError(f"conformance stream line {number} is malformed: {error}") from error
        declared = row.get("protocol")
        if declared != protocol:
            raise ChainError(
                f"conformance stream line {number} declares protocol {declared!r}; "
                f"this adapter transcribes {protocol} and refuses to guess"
            )
        outcome = row.get("outcome")
        if outcome not in CONFORMANCE_OUTCOMES:
            raise ChainError(
                f"conformance stream line {number} declares outcome {outcome!r}, "
                "which this adapter does not name"
            )
        entries.append(
            {
                "symbol": row["symbol"],
                "outcome": CONFORMANCE_OUTCOMES[outcome],
                "traceIds": list(row.get("traceIds", [])),
                # The domain outcome is carried alongside rather than discarded.
                # Quoin normalizes to three values; the twelve-state vocabulary
                # is preserved here and in the retained bytes.
                "domainOutcome": outcome,
            }
        )
    return {"entries": entries}


def declared_malformed_fixtures() -> int:
    """How many corpus fixtures the manifest declares malformed.

    This is the oracle for the malformed count: it comes from the corpus
    declaration, not from the producer's own output, so a producer that stopped
    reporting malformed rows cannot also move the number it is checked against.
    """
    manifest = json.loads(CORPUS_MANIFEST.read_text(encoding="utf-8"))
    return sum(1 for case in manifest["cases"] if case.get("expectedCode") is not None)


# ---------------------------------------------------------------------------
# The chain
# ---------------------------------------------------------------------------


class Chain:
    """Seal, retain and verify, entirely through the pinned Quoin CLI."""

    @staticmethod
    def crate_version() -> str:
        for line in (ROOT / "Cargo.toml").read_text(encoding="utf-8").splitlines():
            if line.startswith("version = "):
                return line.split('"')[1]
        raise ChainError("Cargo.toml declares no package version")

    def observe_tool_versions(self) -> dict[str, str]:
        """One observed version per declared tool identity.

        A tool whose version cannot be observed raises. The alternative is a
        sealed attestation naming a version nobody measured, and an attestation
        is only worth its weakest field.
        """
        crate = self.crate_version()
        probes = {
            # PROOF-msrv's declared command runs cargo THROUGH rustup at the
            # pinned MSRV, so the version sealed into the attestation has to be
            # that toolchain's. Observing ambient `cargo --version` named a
            # version that did not produce the bytes being attested.
            "cargo": lambda: semantic_version(
                tool_version(["rustup", "run", "1.75.0", "cargo", "--version"])
            ),
            "quire": lambda: semantic_version(
                (self.environment.get("quire") or "").split(" ")[0] or None
            ),
        }
        versions: dict[str, str] = {}
        for proof in self.declaration["record"]["definition"]["proof_obligations"]:
            identity = proof["tool_identity"]
            if identity in versions:
                continue
            observed = probes[identity]() if identity in probes else crate
            if observed is None:
                raise ChainError(
                    f"the version of {identity} could not be observed; an attestation "
                    "will not be sealed naming a version nobody measured"
                )
            versions[identity] = observed
        return versions

    def __init__(self, candidate_revision: str, store: Path) -> None:
        self.revision = candidate_revision
        self.store = store
        self.environment = observe_environment()
        self.declaration = json.loads(DECLARATION.read_text(encoding="utf-8"))
        self.observed_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
        self.tool_versions = self.observe_tool_versions()

    # -- record ------------------------------------------------------------

    def record_body(self) -> dict[str, Any]:
        """Project the declaration into Quoin's record body, deriving only digests."""
        declared = json.loads(json.dumps(self.declaration["record"]))
        sources = self.declaration["sources"]
        declared["subject"]["base_revision"] = self.revision
        for connection in declared["source_connections"]:
            path = ROOT / sources[connection["source_id"]]
            if not path.is_file():
                raise ChainError(f"declared source {connection['source_id']} is missing at {path}")
            connection["revision"] = self.revision
            connection["digest"] = digest_of(path.read_bytes())
        for proof in declared["definition"]["proof_obligations"]:
            configuration = proof.pop("configuration")
            path = ROOT / configuration
            if not path.is_file():
                raise ChainError(
                    f"{proof['proof_id']} names configuration {configuration}, which is missing"
                )
            proof["configuration_digest"] = digest_of(path.read_bytes())
            proof["_configuration_path"] = configuration
        export = ASSURANCE_DIR / INPUTS["PROOF-quire-static-export"][0]
        if not export.is_file():
            raise ChainError(
                f"{export} is absent. Run `make assurance-inputs`; this driver does "
                "not run producers."
            )
        declared["impact_snapshot"]["revision"] = self.revision
        declared["impact_snapshot"]["digest"] = digest_of(export.read_bytes())
        return declared

    def seal_record(self) -> tuple[str, dict[str, Any]]:
        body = self.record_body()
        configurations = {
            proof["proof_id"]: proof.pop("_configuration_path")
            for proof in body["definition"]["proof_obligations"]
        }
        result = quoin(
            "change-assurance",
            "seal-record",
            "--repo",
            str(self.store),
            "--input",
            "-",
            "--json",
            stdin=json.dumps(body),
        )
        if result.returncode != 0:
            raise ChainError(f"quoin refused the change-assurance record: {result.stderr.strip()}")
        digest = json.loads(result.stdout)["digest"]
        self.configurations = configurations
        self.record = body
        return digest, body

    # -- attestation -------------------------------------------------------

    def attestation_body(
        self,
        record_digest: str,
        proof_id: str,
        result_state: str,
        *,
        candidate_revision: str | None = None,
    ) -> dict[str, Any]:
        proof = next(
            item
            for item in self.record["definition"]["proof_obligations"]
            if item["proof_id"] == proof_id
        )
        return {
            "schema_version": 1,
            "record_type": "proof_attestation",
            "attestation_id": f"{proof_id}:{result_state}",
            "record_digest": record_digest,
            "candidate_revision": candidate_revision or self.revision,
            "proof_id": proof_id,
            "command": proof["command"],
            "tool": {
                "identity": proof["tool_identity"],
                "version": self.tool_versions[proof["tool_identity"]],
                "configuration_digest": proof["configuration_digest"],
            },
            "environment": self.environment,
            "observed_at": self.observed_at,
            "result": result_state,
        }

    def seal_attestation(
        self, body: dict[str, Any], output: Path, media_type: str
    ) -> dict[str, Any]:
        result = quoin(
            "change-assurance",
            "seal-attestation",
            "--input",
            "-",
            "--output",
            str(output),
            "--media-type",
            media_type,
            "--json",
            stdin=json.dumps(body),
        )
        if result.returncode != 0:
            raise ChainError(f"quoin refused the proof attestation: {result.stderr.strip()}")
        return json.loads(result.stdout)

    def intake(self, attestation: dict[str, Any], output: Path) -> subprocess.CompletedProcess[str]:
        return quoin(
            "change-assurance",
            "intake",
            "--repo",
            str(self.store),
            "--attestation",
            "-",
            "--output",
            str(output),
            "--json",
            stdin=json.dumps(attestation),
        )

    def receipt(
        self,
        record_digest: str,
        selections: dict[str, str],
        decisions: Path,
        *,
        candidate_revision: str | None = None,
        audits: Path | None = None,
    ) -> tuple[int, dict[str, Any]]:
        arguments = [
            "change-assurance",
            "receipt",
            "--repo",
            str(self.store),
            "--record",
            record_digest,
            "--candidate-revision",
            candidate_revision or self.revision,
            "--decisions",
            str(decisions),
            "--json",
        ]
        for proof_id, attestation_digest in selections.items():
            arguments.extend(["--select", f"{proof_id}={attestation_digest}"])
        if audits is not None:
            arguments.extend(["--audits", str(audits)])
        result = quoin(*arguments)
        if result.returncode == 2:
            raise ChainError(f"quoin refused to emit a receipt: {result.stderr.strip()}")
        return result.returncode, json.loads(result.stdout)

    def verify_receipt(self, receipt: dict[str, Any]) -> tuple[int, str]:
        result = quoin(
            "change-assurance",
            "verify-receipt",
            "--input",
            "-",
            "--json",
            stdin=json.dumps(receipt),
        )
        return result.returncode, (result.stdout or result.stderr).strip()


# ---------------------------------------------------------------------------
# Inputs
# ---------------------------------------------------------------------------


def require_inputs() -> dict[str, Path]:
    paths = {}
    for proof_id, (name, _) in INPUTS.items():
        path = ASSURANCE_DIR / name
        if not path.is_file():
            raise ChainError(
                f"{path.relative_to(ROOT)} is absent. Run `make assurance-inputs`. "
                "This driver consumes producer output and never creates it, so an "
                "absent input is an error rather than a step it can quietly do itself."
            )
        paths[proof_id] = path
    return paths


def _worst(results: list[str]) -> str:
    for candidate in RESULT_PRECEDENCE:
        if candidate in results:
            return candidate
    raise ChainError("a producer result stream carried no outcome at all")


def _rows_result(rows: list[dict[str, Any]], where: str) -> str:
    if not rows:
        raise ChainError(
            f"{where} carries no rows. A producer that reported nothing is vacuous, "
            "and vacuous is not passed."
        )
    results = []
    for index, row in enumerate(rows):
        outcome = row.get("outcome")
        if outcome not in ROW_RESULTS:
            raise ChainError(f"{where} row {index} declares outcome {outcome!r}, which is not named")
        results.append(ROW_RESULTS[outcome])
    return _worst(results)


def _load_json(raw: str, path: Path) -> Any:
    """Parse a producer's JSON, or say which file and which target, and stop.

    A bare JSONDecodeError traceback exits 1, which is the code a scenario
    mismatch uses. Unreadable producer output is an environment fault, so it gets
    exit 2 and names the target that writes the file.
    """
    try:
        return json.loads(raw)
    except json.JSONDecodeError as error:
        raise ChainError(
            f"{path.relative_to(ROOT)} is not readable JSON ({error}). This is what "
            "`make assurance-inputs` writes; a result that cannot be read is not a pass."
        ) from error


def derive_result(proof_id: str, path: Path) -> str:
    """Read the producer's own structured verdict out of the bytes it wrote.

    This is the difference between an attestation that states what happened and
    one that states what the caller hoped. Nothing here parses a transcript for
    words: every producer this repository owns emits a declared structured
    result, and `cargo` emits its own JSON message stream, so the verdict is read
    from a field in every case.

    A producer whose output cannot be read at all raises rather than defaulting.
    An attestation that says `passed` because its input was unreadable is the
    single worst failure this file could have.
    """
    raw = path.read_text(encoding="utf-8")
    if not raw.strip():
        raise ChainError(
            f"{path.relative_to(ROOT)} is empty. A producer that wrote nothing has not "
            "reported a result, and an empty file is not a pass. Run `make assurance-inputs`."
        )
    if proof_id in ("PROOF-parser-conformance", "PROOF-roundtrip-property"):
        rows = [_load_json(line, path) for line in raw.splitlines() if line.strip()]
        return _rows_result(rows, path.name)
    if proof_id == "PROOF-test-census":
        return _rows_result(_load_json(raw, path)["entries"], path.name)
    if proof_id == "PROOF-clean-room-attribution":
        report = _load_json(raw, path)
        return "passed" if report["matched"] else "failed"
    if proof_id == "PROOF-legacy-compatibility":
        census = _load_json(raw, path)
        return "passed" if census["matched"] else "failed"
    if proof_id == "PROOF-quire-static-export":
        export = _load_json(raw, path)
        # Quire's export is a static fact set, not a run, so it has no outcome
        # field. What it can be held to is that it carries the coverage facts the
        # impact snapshot claims.
        #
        # An earlier version asked only whether ANY nested value was non-empty.
        # That is true of every real Quire output — `engine` alone satisfies it —
        # so `not_computed` was unreachable and an export reporting 0 of 72 rows
        # backed still attested `passed`. The verdict now reads the totals Quire
        # itself computed.
        if not isinstance(export, dict) or not export:
            return "not_computed"
        totals = export.get("totals")
        if not isinstance(totals, dict) or "backed" not in totals or "total" not in totals:
            # No totals means nothing was measured, which is not a clean export.
            return "not_computed"
        total = totals.get("total") or 0
        backed = totals.get("backed") or 0
        if total == 0 or backed == 0:
            # A coverage export over no rows, or one in which nothing is backed,
            # measured nothing worth snapshotting.
            return "not_computed"
        if export.get("status_lies"):
            # Quire found a row whose declared status disagrees with its evidence.
            return "failed"
        # A partially-backed export is not a failure — four suite rows here are
        # deliberately unbacked and SR-007 says why. The exact figures are pinned
        # by TC-024, so a doctored export changes a number a test asserts rather
        # than only a threshold this driver applies.
        return "passed"
    if proof_id == "PROOF-msrv":
        # `cargo --message-format=json` emits one JSON object per line and ends
        # with `build-finished`. The verdict is that object's `success` field.
        messages = [_load_json(line, path) for line in raw.splitlines() if line.strip()]
        finished = [item for item in messages if item.get("reason") == "build-finished"]
        if not finished:
            # The build did not report finishing. That is not a failure and it is
            # certainly not a pass; it is a run whose result was not computed.
            return "not_computed"
        if any(
            item.get("reason") == "compiler-message"
            and item.get("message", {}).get("level") == "error"
            for item in messages
        ):
            return "failed"
        return "passed" if finished[-1].get("success") is True else "failed"
    raise ChainError(f"no result rule is declared for {proof_id}")


def derive_failed_stream(raw: str) -> str:
    """One named edit to the real conformance stream: the first pass becomes a fail.

    The corpus is green and has to stay green, so the failing case is derived from
    the real run rather than invented. A `fail` state demonstrated by a stream
    nobody produced is a state nobody has actually seen travel the chain.
    """
    lines = [line for line in raw.splitlines() if line.strip()]
    for index, line in enumerate(lines):
        row = json.loads(line)
        if row.get("outcome") == "pass":
            row["outcome"] = "fail"
            lines[index] = json.dumps(row)
            return "\n".join(lines) + "\n"
    raise ChainError("the conformance stream contains no passing row to derive a failure from")


# ---------------------------------------------------------------------------
# Scenarios
# ---------------------------------------------------------------------------


def run_chain(candidate_revision: str, workspace: Path) -> dict[str, Any]:
    inputs = require_inputs()
    store = workspace / "store"
    store.mkdir(parents=True, exist_ok=True)
    scratch = workspace / "scratch"
    scratch.mkdir(parents=True, exist_ok=True)

    chain = Chain(candidate_revision, store)
    record_digest, _ = chain.seal_record()

    decisions = scratch / "decisions.json"
    decisions.write_text(
        json.dumps({"run_id": chain.record["review_workflow"]["run_id"], "events": []}),
        encoding="utf-8",
    )

    def audit_reports(path: Path) -> Path:
        """A clean FR-032 audit report per proof, naming that proof's own obligations."""
        reports = []
        for proof in chain.record["definition"]["proof_obligations"]:
            report = {
                "findings": [],
                "healthy": list(proof["obligation_ids"]),
                "unevaluated": [],
            }
            reports.append(
                {
                    "proof_id": proof["proof_id"],
                    "report_digest": digest_of(
                        json.dumps(report, sort_keys=True, separators=(",", ":")).encode("utf-8")
                    ),
                    "report": report,
                }
            )
        path.write_text(json.dumps(reports), encoding="utf-8")
        return path

    audits = audit_reports(scratch / "audits.json")

    def proof_rows(receipt: dict[str, Any]) -> dict[str, dict[str, Any]]:
        return {row["proof_id"]: row for row in receipt["proofs"]}

    scenarios: list[dict[str, Any]] = []
    controls: list[dict[str, Any]] = []

    def scenario(name: str, state: str | None, matched: bool, detail: Any) -> None:
        """Record a scenario. `state` is None when it demonstrates no outcome."""
        scenarios.append(
            {"scenario": name, "state": state, "matched": bool(matched), "detail": detail}
        )

    def control(name: str, pairs_with: str, matched: bool, detail: Any) -> None:
        controls.append(
            {"control": name, "pairs_with": pairs_with, "matched": bool(matched), "detail": detail}
        )

    # -- 1. the honest path: seal, retain, and get the bytes back unchanged ---
    selections: dict[str, str] = {}
    observed_results: dict[str, str] = {}
    retained_conformance: bytes = b""
    for proof_id, path in inputs.items():
        media_type = INPUTS[proof_id][1]
        observed = derive_result(proof_id, path)
        observed_results[proof_id] = observed
        body = chain.attestation_body(record_digest, proof_id, observed)
        sealed = chain.seal_attestation(body, path, media_type)
        taken = chain.intake(sealed, path)
        if taken.returncode != 0:
            raise ChainError(
                f"{proof_id}: intake refused an unmodified producer output: {taken.stderr.strip()}"
            )
        detail = json.loads(taken.stdout)
        retained = Path(detail["directory"]) / "output.bin"
        identical = retained.read_bytes() == path.read_bytes()
        selections[proof_id] = sealed["digest"]
        if proof_id == "PROOF-parser-conformance":
            retained_conformance = retained.read_bytes()
            scenario(
                "retain-producer-output",
                "pass",
                identical,
                {"retained": str(retained), "bytes": retained.stat().st_size},
            )
            control(
                "intake-accepts-unchanged-bytes",
                "retained-bytes-changed-after-sealing",
                identical,
                {"proof": proof_id},
            )

    scenario(
        "attested-results-are-read-from-producer-output",
        None,
        all(result == "passed" for result in observed_results.values()),
        observed_results,
    )

    # -- 1b. malformed input stays malformed ---------------------------------
    #
    # The repository-owned behaviour this migration must not lose. Three
    # separate facts, because any one of them alone is satisfiable by an
    # implementation that got it wrong.
    conformance_rows = [
        json.loads(line)
        for line in inputs["PROOF-parser-conformance"].read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    malformed_rows = [row for row in conformance_rows if row.get("outcome") == "malformed"]
    declared_malformed = declared_malformed_fixtures()
    scenario(
        "malformed-input-is-reported-as-malformed",
        "malformed",
        len(malformed_rows) == declared_malformed and declared_malformed > 0,
        {
            "declared_by_corpus_manifest": declared_malformed,
            "reported_by_producer": len(malformed_rows),
            "why": (
                "the count is checked against the corpus manifest, not against the "
                "producer's own output, so a producer that stopped reporting malformed "
                "rows cannot also move the number it is compared to"
            ),
        },
    )
    # The proof is not dragged to `failed` by a corpus that is malformed by design.
    scenario(
        "malformed-does-not-fail-its-proof",
        None,
        observed_results["PROOF-parser-conformance"] == "passed" and bool(malformed_rows),
        {
            "attested": observed_results["PROOF-parser-conformance"],
            "malformed_rows": len(malformed_rows),
        },
    )
    # And the word survives into the bytes Quoin retained, so the state is not
    # merely computed and discarded.
    retained_text = retained_conformance.decode("utf-8", errors="replace")
    retained_malformed = sum(
        1
        for line in retained_text.splitlines()
        if line.strip() and json.loads(line).get("outcome") == "malformed"
    )
    scenario(
        "malformed-survives-into-retained-bytes",
        None,
        retained_malformed == declared_malformed,
        {"retained_malformed_rows": retained_malformed},
    )
    # The positive control: the adapter transcribes malformed to a Quoin `pass`
    # entry because the obligation was discharged, and it must be seen NOT to
    # invent that outcome for a stream that has none.
    transcribed_real = adapt_conformance(
        inputs["PROOF-parser-conformance"].read_text(encoding="utf-8")
    )
    carried = sum(
        1 for entry in transcribed_real["entries"] if entry["domainOutcome"] == "malformed"
    )
    control(
        "adapter-carries-the-domain-outcome-alongside",
        "malformed-input-is-reported-as-malformed",
        carried == declared_malformed,
        {"carried": carried},
    )

    # -- 2. the receipt, and re-verifying it ---------------------------------
    status, receipt = chain.receipt(record_digest, selections, decisions)
    verified_status, _ = chain.verify_receipt(receipt)
    scenario(
        "receipt-reports-the-absent-human-decision",
        "partial",
        status == 1
        and receipt["outcome"] == "incomplete"
        and "decision_missing" in receipt["reasons"]
        and receipt["checks"]["review"]["outcome"] == "incomplete"
        and receipt["decision_event"] is None,
        {
            "outcome": receipt["outcome"],
            "exit": status,
            "reasons": receipt["reasons"],
            "review": receipt["checks"]["review"],
        },
    )
    scenario(
        "re-verify-the-sealed-receipt",
        "pass",
        verified_status == status,
        {"verify_exit": verified_status, "receipt_exit": status},
    )
    control(
        "verify-accepts-an-unedited-receipt",
        "refuse-an-edited-receipt",
        verified_status != 2,
        {"exit": verified_status},
    )

    # -- 3. an edited receipt is refused -------------------------------------
    edited = json.loads(json.dumps(receipt))
    edited["outcome"] = "valid"
    edited_status, edited_detail = chain.verify_receipt(edited)
    scenario(
        "refuse-an-edited-receipt",
        "tampered",
        edited_status == 2,
        {"exit": edited_status, "message": edited_detail[:200]},
    )

    # -- 4. retained bytes changed after sealing -----------------------------
    moved = scratch / "moved.jsonl"
    moved.write_bytes(inputs["PROOF-parser-conformance"].read_bytes())
    body = chain.attestation_body(record_digest, "PROOF-parser-conformance", "passed")
    sealed_moved = chain.seal_attestation(body, moved, "application/x-ndjson")
    moved.write_bytes(moved.read_bytes() + b"\n")
    refused = chain.intake(sealed_moved, moved)
    scenario(
        "retained-bytes-changed-after-sealing",
        "tampered",
        refused.returncode != 0,
        {"exit": refused.returncode, "message": refused.stderr.strip()[:200]},
    )

    # -- 5. a stale candidate binding ----------------------------------------
    stale_status, stale_receipt = chain.receipt(
        record_digest, selections, decisions, candidate_revision="0" * 40, audits=audits
    )
    stale_reasons = set(proof_rows(stale_receipt)["PROOF-parser-conformance"]["reasons"])
    scenario(
        "stale-candidate-binding",
        "stale",
        "candidate_revision_mismatch" in stale_reasons,
        {"outcome": stale_receipt["outcome"], "reasons": sorted(stale_reasons)},
    )

    # -- 6. attested non-success states, each named by its own reason ---------
    audited_status, audited = chain.receipt(record_digest, selections, decisions, audits=audits)
    passing_row = proof_rows(audited)["PROOF-parser-conformance"]
    control(
        "an-audited-passing-proof-is-valid-and-reasonless",
        "attested-failed",
        passing_row["outcome"] == "valid" and not passing_row["reasons"],
        {"row": passing_row["outcome"], "reasons": passing_row["reasons"]},
    )
    control(
        "receipt-discharges-a-current-binding",
        "stale-candidate-binding",
        "candidate_revision_mismatch" not in passing_row["reasons"],
        {"reasons": passing_row["reasons"]},
    )

    failed_stream = scratch / "failed.jsonl"
    failed_stream.write_text(
        derive_failed_stream(inputs["PROOF-parser-conformance"].read_text(encoding="utf-8")),
        encoding="utf-8",
    )
    expected_reason = {
        "failed": "result_failed",
        "unavailable": "result_unavailable",
        "not_computed": "result_not_computed",
    }
    state_name = {"failed": "fail", "unavailable": "unavailable", "not_computed": "not-computed"}
    observed_reasons: dict[str, set[str]] = {}
    for state, source in (
        ("failed", failed_stream),
        ("unavailable", inputs["PROOF-parser-conformance"]),
        ("not_computed", inputs["PROOF-parser-conformance"]),
    ):
        body = chain.attestation_body(record_digest, "PROOF-parser-conformance", state)
        body["attestation_id"] = f"PROOF-parser-conformance:{state}"
        sealed_state = chain.seal_attestation(body, source, "application/x-ndjson")
        taken = chain.intake(sealed_state, source)
        if taken.returncode != 0:
            raise ChainError(f"intake refused a {state} attestation: {taken.stderr.strip()}")
        state_selections = dict(selections)
        state_selections["PROOF-parser-conformance"] = sealed_state["digest"]
        _, state_receipt = chain.receipt(record_digest, state_selections, decisions, audits=audits)
        rows = proof_rows(state_receipt)
        reasons = set(rows["PROOF-parser-conformance"]["reasons"])
        observed_reasons[state] = reasons
        scenario(
            f"attested-{state}",
            state_name[state],
            expected_reason[state] in reasons,
            {"reasons": sorted(reasons), "receipt_outcome": state_receipt["outcome"]},
        )
        if state == "failed":
            control(
                "passing-proof-is-not-reported-as-failing",
                "attested-failed",
                not set(rows["PROOF-roundtrip-property"]["reasons"])
                & set(expected_reason.values()),
                {"roundtrip_reasons": rows["PROOF-roundtrip-property"]["reasons"]},
            )

    distinct = len({frozenset(value) for value in observed_reasons.values()}) == 3
    scenario(
        "non-success-states-stay-distinguishable",
        None,
        distinct,
        {state: sorted(value) for state, value in observed_reasons.items()},
    )

    # -- 7. an unaudited proof is not-computed, not clean ---------------------
    unaudited_row = proof_rows(receipt)["PROOF-parser-conformance"]
    scenario(
        "audited-clean-versus-unaudited",
        "not-computed",
        "audit_not_evaluated" in unaudited_row["reasons"]
        and "audit_not_evaluated" not in passing_row["reasons"],
        {
            "unaudited": unaudited_row["reasons"],
            "audited": passing_row["reasons"],
            "why": (
                "an audit with no findings and no audit at all are different facts; "
                "the absence is reported as not-computed rather than as clean"
            ),
        },
    )
    control(
        "an-audit-that-was-run-clears-not-computed",
        "audited-clean-versus-unaudited",
        audited_status in (0, 1) and "audit_not_evaluated" not in audited["reasons"],
        {"receipt_reasons": audited["reasons"]},
    )

    # -- 8. a proof with no attestation stays missing -------------------------
    partial_selections = {key: value for key, value in selections.items() if key != "PROOF-msrv"}
    _, partial = chain.receipt(record_digest, partial_selections, decisions, audits=audits)
    missing_row = proof_rows(partial).get("PROOF-msrv", {})
    scenario(
        "unattested-proof-stays-missing",
        "partial",
        partial["outcome"] != "valid"
        and "attestation_missing" in set(missing_row.get("reasons", [])),
        {"outcome": partial["outcome"], "msrv_reasons": missing_row.get("reasons")},
    )

    # -- 9. the open unknowns survive into the receipt ------------------------
    declared_unknowns = {item["id"] for item in chain.record["definition"]["unknowns"]}
    carried_unknowns = {item["id"] for item in audited.get("unknowns", [])}
    scenario(
        "declared-unknowns-are-carried-not-dropped",
        "inconclusive",
        declared_unknowns == carried_unknowns and "unresolved_unknown" in audited["reasons"],
        {"declared": sorted(declared_unknowns), "carried": sorted(carried_unknowns)},
    )

    names = {item["scenario"] for item in scenarios}
    dangling = sorted(item["control"] for item in controls if item["pairs_with"] not in names)
    if dangling:
        raise ChainError(
            f"these controls name a scenario that does not exist: {dangling}. "
            f"Scenarios present: {sorted(names)}"
        )

    return {
        "record_digest": record_digest,
        "candidate_revision": candidate_revision,
        "impact_snapshot_digest": chain.record["impact_snapshot"]["digest"],
        "quire_export": str(
            (ASSURANCE_DIR / INPUTS["PROOF-quire-static-export"][0]).relative_to(ROOT)
        ),
        "attested_results": observed_results,
        "malformed_rows": len(malformed_rows),
        "declared_malformed_fixtures": declared_malformed,
        "receipt_outcome": receipt["outcome"],
        "audited_receipt_outcome": audited["outcome"],
        "audited_receipt_reasons": audited["reasons"],
        "scenarios": scenarios,
        "controls": controls,
    }


# ---------------------------------------------------------------------------
# Adapter probes
# ---------------------------------------------------------------------------


def adapter_probes(workspace: Path) -> list[dict[str, Any]]:
    """Exercise the native adapter and Quoin's evidence audit in a scratch tree."""
    inputs = require_inputs()
    probe_root = workspace / "adapter"
    if probe_root.exists():
        shutil.rmtree(probe_root)
    probe_root.mkdir(parents=True)
    shutil.copytree(ROOT / "spec", probe_root / "spec")

    stream = inputs["PROOF-parser-conformance"].read_text(encoding="utf-8")
    commit = "0" * 40
    results = []

    def record(suite: str, payload: dict[str, Any], commit_sha: str) -> dict[str, Any]:
        path = probe_root / "run.json"
        # Quoin's `entries` adapter owns its own shape; the domain outcome this
        # adapter carries alongside is for this repository's readers, not for
        # Quoin, so it is dropped at the tool boundary rather than smuggled in.
        payload = {
            "entries": [
                {key: value for key, value in entry.items() if key != "domainOutcome"}
                for entry in payload["entries"]
            ]
        }
        path.write_text(json.dumps(payload), encoding="utf-8")
        outcome = quoin(
            "evidence",
            "record",
            "--repo",
            str(probe_root),
            "--suite",
            suite,
            "--commit",
            commit_sha,
            "--tool",
            "tl-parse-corpus-conformance 0.1.0",
            "--adapter",
            "entries",
            "--kind",
            "Integration",
            "--results",
            str(path),
            "--json",
        )
        if outcome.returncode != 0:
            raise ChainError(f"quoin refused an evidence record: {outcome.stderr.strip()}")
        return json.loads(outcome.stdout)

    def audit_kinds() -> dict[str, int]:
        outcome = quoin("evidence", "audit", "--repo", str(probe_root), "--json")
        if outcome.returncode not in (0, 1):
            raise ChainError(f"quoin evidence audit failed: {outcome.stderr.strip()}")
        findings = json.loads(outcome.stdout)["findings"]
        counted: dict[str, int] = {}
        for finding in findings:
            counted[finding["kind"]] = counted.get(finding["kind"], 0) + 1
        return counted

    # Probe 1 (positive control): the real run binds real obligations.
    transcribed = adapt_conformance(stream)
    bound = record("SUITE-001", transcribed, commit)["bound"]
    results.append(
        {
            "probe": "accepts-the-real-run",
            "state": "pass",
            "matched": bool(bound),
            "detail": {"bound": len(bound), "entries": len(transcribed["entries"])},
        }
    )

    # Probe 2: the adapter must carry a non-success outcome through as a
    # non-success outcome, transcribed by the adapter rather than hand-built.
    not_computed_stream = "\n".join(
        json.dumps({**json.loads(line), "outcome": "not-computed"})
        for line in stream.splitlines()
        if line.strip()
    )
    downgraded = adapt_conformance(not_computed_stream)
    preserved = all(entry["outcome"] != "pass" for entry in downgraded["entries"])
    results.append(
        {
            "probe": "adapter-preserves-non-success-outcomes",
            "state": "not-computed",
            "matched": preserved and bool(downgraded["entries"]),
            "detail": {
                "outcomes": sorted({entry["outcome"] for entry in downgraded["entries"]}),
                "entries": len(downgraded["entries"]),
            },
        }
    )

    # Probe 3: a run in which every bound symbol was skipped is vacuous.
    record("SUITE-001", downgraded, commit)
    kinds = audit_kinds()
    results.append(
        {
            "probe": "audit-reports-a-vacuous-run",
            "state": "vacuous",
            "matched": kinds.get("vacuous-evidence", 0) > 0,
            "detail": kinds,
        }
    )

    # Probe 4: a reworded statement makes its bound evidence suspect.
    record("SUITE-001", transcribed, commit)
    requirement = probe_root / "spec" / "requirements" / "FR-005-corpus-cli-evidence.md"
    text = requirement.read_text(encoding="utf-8")
    # The reworded text has to be the STATEMENT of the obligation the evidence is
    # bound to — `quoin evidence record` binds these entries to FR-005-AC-1 — not
    # merely prose elsewhere in the document. Editing the surrounding narrative
    # changes the file without changing the claim, and the audit correctly says
    # nothing about it.
    marker = "every fixture produces its declared bounded outcome"
    if marker not in text:
        raise ChainError(
            "the probe's statement marker is no longer present in FR-005-AC-1; it must "
            "reword the criterion the evidence binds to, or the probe proves nothing"
        )
    requirement.write_text(
        text.replace(marker, f"{marker} and class", 1), encoding="utf-8"
    )
    kinds = audit_kinds()
    results.append(
        {
            "probe": "audit-reports-a-suspect-link",
            "state": "suspect",
            "matched": kinds.get("suspect-link", 0) > 0,
            "detail": kinds,
        }
    )

    # Probe 5: a foreign protocol is refused by the adapter, not guessed at.
    foreign = "\n".join(
        json.dumps({**json.loads(line), "protocol": "some.other.protocol/v1"})
        for line in stream.splitlines()
        if line.strip()
    )
    refused = False
    try:
        adapt_conformance(foreign)
    except ChainError:
        refused = True
    results.append(
        {
            "probe": "refuses-a-foreign-protocol",
            "state": "unsupported",
            "matched": refused,
            "detail": {"protocol": "some.other.protocol/v1"},
        }
    )

    # Probe 6: an empty stream is refused rather than transcribed into a clean run.
    empty_refused = False
    try:
        adapt_conformance("")
    except ChainError:
        empty_refused = True
    results.append(
        {
            "probe": "refuses-an-empty-stream",
            "state": "vacuous",
            "matched": empty_refused,
            "detail": {},
        }
    )

    # Probe 7: an outcome the adapter does not name is refused rather than
    # defaulted. `malformed` is a named outcome here and `mangled` is not, so
    # this also shows the malformed mapping is an enumerated decision rather
    # than a fall-through that would accept anything.
    unnamed = "\n".join(
        json.dumps({**json.loads(line), "outcome": "mangled"})
        for line in stream.splitlines()
        if line.strip()
    )
    unnamed_refused = False
    try:
        adapt_conformance(unnamed)
    except ChainError:
        unnamed_refused = True
    results.append(
        {
            "probe": "refuses-an-unnamed-outcome",
            "state": "unsupported",
            "matched": unnamed_refused,
            "detail": {"outcome": "mangled"},
        }
    )

    return results


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--candidate-revision")
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "--keep-store",
        action="store_true",
        help="keep this run's Quoin store under target/assurance-store for inspection",
    )
    parser.add_argument(
        "--adapt",
        metavar="PATH",
        help=(
            "transcribe a domain conformance stream into Quoin's normalized entries "
            "and print them; this is the adapter on its own, with no chain around it"
        ),
    )
    arguments = parser.parse_args(argv[1:])

    if arguments.adapt is not None:
        try:
            entries = adapt_conformance(Path(arguments.adapt).read_text(encoding="utf-8"))
        except (ChainError, OSError) as error:
            print(str(error), file=sys.stderr)
            return 2
        print(json.dumps(entries, indent=2, sort_keys=True))
        return 0

    if arguments.candidate_revision is None:
        print("--candidate-revision is required", file=sys.stderr)
        return 2

    STORE.mkdir(parents=True, exist_ok=True)
    workspace = Path(tempfile.mkdtemp(prefix="run-", dir=STORE))

    try:
        chain = run_chain(arguments.candidate_revision, workspace)
        probes = adapter_probes(workspace)
    except ChainError as error:
        print(str(error), file=sys.stderr)
        return 2
    finally:
        if not arguments.keep_store:
            shutil.rmtree(workspace, ignore_errors=True)

    report = {
        "schemaVersion": "tl-parse.assurance-chain-report/v1",
        **chain,
        "adapter_probes": probes,
        "states_demonstrated": sorted(
            {
                item["state"]
                for group in (chain["scenarios"], probes)
                for item in group
                if item["matched"] and item.get("state") is not None
            }
        ),
        "matched": all(
            item["matched"]
            for group in (chain["scenarios"], chain["controls"], probes)
            for item in group
        ),
    }
    if arguments.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        for item in chain["scenarios"]:
            print(
                f"scenario {item['scenario']} [{item['state']}]: "
                f"{'ok' if item['matched'] else 'MISMATCH'}"
            )
        for item in chain["controls"]:
            print(
                f"control  {item['control']} (pairs with {item['pairs_with']}): "
                f"{'ok' if item['matched'] else 'MISMATCH'}"
            )
        for item in probes:
            print(
                f"probe    {item['probe']} [{item['state']}]: "
                f"{'ok' if item['matched'] else 'MISMATCH'}"
            )
    if not report["matched"]:
        print("the assurance chain did not match its declared scenarios", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
