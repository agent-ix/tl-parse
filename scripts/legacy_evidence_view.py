#!/usr/bin/env python3
"""Read this repository's retained evidence through the pinned shared mapping (FR-006-AC-4).

Four things this file deliberately is not.

It is not a mapping. Every byte it interprets is interpreted by
`engineering_assurance.verification_semantics.map_pgm01_bytes` from the pinned
release. This file chooses which bytes to hand over and reports what came back.
If the answer is unwelcome, the answer is still the answer.

It is not a verifier. It replaces `verify_evidence.sh`,
`verify_evidence_manifest.py`, `verify_evidence_history.py`, `evidence_profile.py`
and `finalize_collection.py`, and it does not inherit their job. It does not
decide whether a retained record qualifies anything, does not maintain an anchor
file, and has no notion of an active record.

It is not a writer. It opens every file under `evidence/` for reading and proves
it: it digests the whole tree before and after the run and fails if one byte
moved. Read-only is a claim, so it is measured. Whether the retained bytes are
the bytes that were *committed* is a separate and stronger claim, and Git is
asked rather than a second local manifest being invented.

It is not a translator of last resort. This repository's 12 retained envelopes
are `quire.derivation-evidence/v1`. The pinned mapping covers
`quire.pgm01-evidence` v1 and v2 and answers `incompatible` for the rest. That
refusal is reported as the compatibility result. Writing a local mapper to turn
it into something friendlier is the exact thing the migration removed.

Exit status: 0 when every declared case matched and no evidence byte moved,
1 when a case did not match or bytes moved, 2 when the pinned release cannot be
loaded.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE = ROOT / "evidence"
FIXTURES = ROOT / "tests" / "fixtures" / "legacy-compat"
EXPECTATIONS = FIXTURES / "expectations.json"
PINS_PATH = ROOT / "assurance" / "pins.json"

# The mapping's own state vocabulary, as PGM01_STATE_MAP defines it. The census
# requires every one of these to be demonstrated by some case, because a state
# nothing exercises is a state nothing would notice the loss of.
REQUIRED_STATES = ("passed", "failed", "error", "skipped", "inconclusive", "unavailable")

# The outcomes map_pgm01_bytes can return. Same argument.
REQUIRED_OUTCOMES = ("lossy", "incompatible", "unreadable")


class ViewError(RuntimeError):
    """The pinned mapping or its inputs could not be used."""


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def load_mapper() -> Callable[..., dict[str, Any]]:
    try:
        from engineering_assurance.verification_semantics import map_pgm01_bytes
    except ImportError as error:
        raise ViewError(
            f"the pinned assurance distribution is unusable: {error}. "
            "Run `make assurance-env`. This is an error and not a skip."
        ) from error
    return map_pgm01_bytes


def release_root() -> Path:
    try:
        import engineering_assurance
    except ImportError as error:
        raise ViewError(f"the pinned assurance distribution is unusable: {error}") from error
    return Path(engineering_assurance.__file__).resolve().parent


def pinned_source_bytes(relative: str) -> bytes:
    """Read a pinned artifact out of the installed release, checked against pins.json.

    The derived fixtures in this repository are one named edit to these bytes. If
    the release's bytes are not the bytes the pin names, the derivation claim is
    void and there is nothing to compare a fixture against.
    """
    pins = json.loads(PINS_PATH.read_text(encoding="utf-8"))
    expected = None
    for artifact in pins["consumed_artifacts"]:
        if artifact["path"] == relative:
            expected = artifact.get("sha256")
            break
    if expected is None:
        raise ViewError(f"{relative} is not a digest-pinned consumed artifact")
    path = release_root() / relative
    if not path.is_file():
        raise ViewError(f"{relative} is absent from the installed release")
    raw = path.read_bytes()
    actual = sha256(raw)
    if actual != expected:
        raise ViewError(f"{relative}: {actual}, pins record {expected}")
    return raw


def derive(raw: bytes, derivation: dict[str, Any]) -> bytes:
    """Apply exactly one named edit to real bytes."""
    operation = derivation["operation"]
    if operation == "truncate":
        count = int(derivation["bytes"])
        if count >= len(raw):
            raise ViewError("a truncation that removes nothing is not a derivation")
        return raw[:count]
    if operation == "replace":
        find = derivation["find"].encode("utf-8")
        replace = derivation["replace"].encode("utf-8")
        occurrences = int(derivation.get("occurrences", 1))
        seen = raw.count(find)
        if seen != occurrences:
            raise ViewError(
                f"derivation expected {occurrences} occurrence(s) of {derivation['find']!r}, "
                f"found {seen}; the edit is not the single named change it claims to be"
            )
        return raw.replace(find, replace)
    raise ViewError(f"unknown derivation operation: {operation}")


def evidence_census() -> dict[str, str]:
    """Digest every retained byte, so read-only can be measured rather than asserted."""
    return {
        path.relative_to(ROOT).as_posix(): sha256(path.read_bytes())
        for path in sorted(EVIDENCE.rglob("*"))
        if path.is_file()
    }


def uncommitted_evidence_changes() -> list[str]:
    """Ask Git whether any retained byte differs from what was committed.

    The before/after census in this file proves only that *this process* did not
    write, which is a narrower claim than it sounds. Git history and pull-request
    review are the integrity boundary for retained bytes, so the boundary is
    consulted here rather than a second local manifest being invented to replace
    the one this migration removed.

    A tree where Git is unavailable reports that fact instead of an empty list,
    because "nothing changed" and "nobody looked" must not be the same answer.
    """
    try:
        result = subprocess.run(
            ["git", "status", "--porcelain", "--", "evidence"],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        raise ViewError(f"git could not be run to check retained bytes: {error}") from error
    if result.returncode != 0:
        raise ViewError(f"git refused to report on retained bytes: {result.stderr.strip()}")
    return [line for line in result.stdout.splitlines() if line.strip()]


def retained_envelopes() -> list[Path]:
    paths = sorted(EVIDENCE.glob("tl-parse-v01-*/evidence-envelope.json"))
    if not paths:
        raise ViewError("no retained evidence envelope was found; there is nothing to read")
    return paths


def states_in(view: dict[str, Any]) -> list[str]:
    return [
        mapping["value"]
        for mapping in view["mappings"]
        if mapping["target_field"] == "state" and isinstance(mapping["value"], str)
    ]


def case_bytes(case: dict[str, Any]) -> bytes:
    """Produce the bytes a declared case is about, re-deriving what it claims to derive."""
    if case.get("root") == "release":
        raw = pinned_source_bytes(case["file"])
    elif case.get("root") == "repository":
        raw = (ROOT / case["file"]).read_bytes()
    else:
        source = pinned_source_bytes(case["source"])
        raw = derive(source, case["derivation"])
        committed = FIXTURES / case["file"]
        if not committed.is_file():
            raise ViewError(f"{case['id']}: declared fixture {case['file']} is not committed")
        if committed.read_bytes() != raw:
            raise ViewError(
                f"{case['id']}: the committed fixture is not the declared derivation "
                "re-applied to the pinned source; a fixture that cannot be re-derived "
                "is a hand-written blob claiming to be one named change"
            )
    return raw


def run_case(mapper: Callable[..., dict[str, Any]], case: dict[str, Any]) -> dict[str, Any]:
    raw = case_bytes(case)
    expected_digest = None
    if "bind_digest" in case:
        expected_digest = case["bind_digest"]
        if expected_digest == "self":
            expected_digest = sha256(raw)
    view = mapper(raw, expected_digest=expected_digest)
    observed_states = states_in(view)
    reasons = [item["reason"] for item in view["unmapped_fields"]]
    # The reported identity is compared, not merely printed. A digest a view
    # states and nothing checks is decorative, and a view that can misreport
    # which bytes it read can attribute any answer to any record.
    identity_ok = view["source_digest"] == sha256(raw)
    matched = identity_ok and view["outcome"] == case["expected_outcome"]
    # Where the mapping distinguishes cases by a structured field, that field is
    # what is asserted. `map_pgm01_bytes` sets `source_record_id` to
    # `tampered-source` for a digest mismatch and `unreadable-source` for
    # undecodable bytes, so tampered and unreadable are told apart structurally
    # rather than by the wording of a reason.
    if matched and "expected_record_id" in case:
        matched = view["source_record_id"] == case["expected_record_id"]
    if matched and "expected_schema_version" in case:
        matched = view["source_schema_version"] == case["expected_schema_version"]
    # One pair remains separable only by the upstream reason text: an unknown
    # check status and a structurally undecodable body both return `unreadable`
    # under the real record id. That is the mapping's only discriminator for
    # them, and the limitation is stated in expectations.json rather than
    # papered over by re-deriving a discriminator locally.
    if matched and "expected_reason_contains" in case:
        matched = any(case["expected_reason_contains"] in reason for reason in reasons)
    if matched:
        matched = set(case.get("requires_states", [])) <= set(observed_states)
    return {
        "id": case["id"],
        "kind": case["kind"],
        "expected_outcome": case["expected_outcome"],
        "outcome": view["outcome"],
        "source_digest": view["source_digest"],
        "source_identity_verified": identity_ok,
        "source_schema_version": view["source_schema_version"],
        "mapped_states": sorted(set(observed_states)),
        "unmapped_reasons": reasons,
        "matched": matched,
        "why": case["why"],
    }


def retained_report(mapper: Callable[..., dict[str, Any]]) -> dict[str, Any]:
    """Read every retained envelope in this repository through the pinned mapping."""
    entries = []
    for path in retained_envelopes():
        raw = path.read_bytes()
        view = mapper(raw)
        entries.append(
            {
                "path": path.relative_to(ROOT).as_posix(),
                "source_digest": view["source_digest"],
                "source_identity_verified": view["source_digest"] == sha256(raw),
                "declared_schema_version": json.loads(raw).get("schemaVersion"),
                "outcome": view["outcome"],
                "reasons": [item["reason"] for item in view["unmapped_fields"]],
            }
        )
    outcomes = sorted({entry["outcome"] for entry in entries})
    families = sorted({entry["declared_schema_version"] for entry in entries})
    return {
        "count": len(entries),
        "declared_schema_versions": families,
        "outcomes": outcomes,
        "statement": (
            "This repository retained no quire.pgm01-evidence record. The pinned mapping "
            "covers quire.pgm01-evidence v1 and v2 and refuses every other schema version, "
            "so each retained envelope reads as 'incompatible'. That is the mapping declining "
            "to interpret a shape it has never seen. It is reported, not converted into a "
            "pass and not converted into a defect of these records. tl-syntax reported the "
            "identical refusal over its own 23 envelopes. Filed upstream as "
            "agent-ix/engineering-assurance#21."
        ),
        "entries": entries,
    }


def census(mapper: Callable[..., dict[str, Any]]) -> dict[str, Any]:
    uncommitted = uncommitted_evidence_changes()
    before = evidence_census()
    declaration = json.loads(EXPECTATIONS.read_text(encoding="utf-8"))
    cases = [run_case(mapper, case) for case in declaration["cases"]]
    retained = retained_report(mapper)
    after = evidence_census()
    moved = sorted(path for path in set(before) | set(after) if before.get(path) != after.get(path))
    demonstrated_states = {state for case in cases for state in case["mapped_states"]}
    demonstrated_outcomes = {case["outcome"] for case in cases} | set(retained["outcomes"])
    missing_states = sorted(set(REQUIRED_STATES) - demonstrated_states)
    missing_outcomes = sorted(set(REQUIRED_OUTCOMES) - demonstrated_outcomes)
    unmatched = [case["id"] for case in cases if not case["matched"]]
    misattributed = [
        entry["path"] for entry in retained["entries"] if not entry["source_identity_verified"]
    ]
    # A census with no positive control is a census that has only ever seen the
    # mapping refuse, which is indistinguishable from a mapping that never works.
    accepted_controls = [
        case["id"]
        for case in cases
        if case["kind"] == "positive_control" and case["matched"] and case["outcome"] == "lossy"
    ]
    return {
        "schemaVersion": "tl-parse.legacy-compatibility-census/v1",
        "mapping_authority": (
            "engineering_assurance.verification_semantics.map_pgm01_bytes from the pinned "
            "release. This repository implements no mapping of its own."
        ),
        "evidence_files_read": len(before),
        # Named for exactly what it measures. This process reads; the array is
        # what changed between its own before and after census, so an empty array
        # means this run wrote nothing. It is not, and does not claim to be, a
        # statement that the retained bytes match what was committed — that is
        # `uncommitted_evidence_changes` below, which asks Git.
        "evidence_bytes_moved_during_this_run": moved,
        "uncommitted_evidence_changes": uncommitted,
        "cases": cases,
        "retained": retained,
        "required_states": list(REQUIRED_STATES),
        "demonstrated_states": sorted(demonstrated_states),
        "undemonstrated_states": missing_states,
        "required_outcomes": list(REQUIRED_OUTCOMES),
        "demonstrated_outcomes": sorted(demonstrated_outcomes),
        "undemonstrated_outcomes": missing_outcomes,
        "accepted_positive_controls": accepted_controls,
        "unmatched_cases": unmatched,
        "misattributed_records": misattributed,
        "matched": (
            not unmatched
            and not moved
            and not uncommitted
            and not missing_states
            and not missing_outcomes
            and not misattributed
            and bool(accepted_controls)
        ),
    }


def run_mutation_probes(mapper: Callable[..., dict[str, Any]]) -> dict[str, Any]:
    """Remove one load-bearing check at a time and require the census to go red.

    A gate that has never been observed to fail is indistinguishable from a gate
    that does not run. Each probe below degrades exactly one thing the census
    relies on; if the census still passes, the check it relies on is decorative.
    """

    def collapsing(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        """Report every non-success state as a pass."""
        view = mapper(raw, **kwargs)
        for mapping in view["mappings"]:
            if mapping["target_field"] == "state" and mapping["value"] != "passed":
                mapping["value"] = "passed"
        return view

    def repairing(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        """Report an unreadable record as readable."""
        view = mapper(raw, **kwargs)
        if view["outcome"] == "unreadable":
            view["outcome"] = "lossy"
        return view

    def accepting(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        """Report a refused schema as an accepted one."""
        view = mapper(raw, **kwargs)
        if view["outcome"] == "incompatible":
            view["outcome"] = "lossy"
        return view

    def unbinding(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        """Ignore the caller's expected identity, so a tampered record reads normally."""
        kwargs.pop("expected_digest", None)
        return mapper(raw, **kwargs)

    def forgetting_identity(raw: bytes, **kwargs: Any) -> dict[str, Any]:
        """Report a source digest that is not the digest of the source."""
        view = mapper(raw, **kwargs)
        view["source_digest"] = "0" * 64
        return view

    probes = {
        "collapse-non-success-states": collapsing,
        "repair-unreadable-outcome": repairing,
        "accept-refused-schema": accepting,
        "unbind-tamper-digest": unbinding,
        "drop-source-identity": forgetting_identity,
    }
    results = []
    for name, degraded in probes.items():
        # No exception handling. A probe that crashes has not demonstrated that
        # the census noticed anything — it has demonstrated that the probe is
        # broken — and counting a traceback as a detection is how "5/5 detected"
        # stops meaning anything.
        detected = not census(degraded)["matched"]
        results.append({"probe": name, "detected": detected})
    detected_count = sum(1 for item in results if item["detected"])
    return {
        "probes": results,
        "detected": detected_count,
        "total": len(results),
        "matched": detected_count == len(results),
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--json", action="store_true", help="emit the census as JSON")
    parser.add_argument(
        "--mutation-probes",
        action="store_true",
        help="degrade one load-bearing check at a time and require the census to notice",
    )
    arguments = parser.parse_args(argv[1:])
    try:
        mapper = load_mapper()
        report = run_mutation_probes(mapper) if arguments.mutation_probes else census(mapper)
    except ViewError as error:
        print(str(error), file=sys.stderr)
        return 2
    if arguments.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    elif arguments.mutation_probes:
        for item in report["probes"]:
            print(f"{item['probe']}: {'detected' if item['detected'] else 'NOT DETECTED'}")
        print(f"mutation probes detected {report['detected']}/{report['total']}")
    else:
        for case in report["cases"]:
            flag = "ok" if case["matched"] else "MISMATCH"
            print(f"{case['id']}: {case['outcome']} ({flag})")
        retained = report["retained"]
        print(
            f"retained envelopes: {retained['count']} "
            f"{retained['declared_schema_versions']} -> {retained['outcomes']}"
        )
        print(
            f"evidence files read: {report['evidence_files_read']}, "
            f"bytes moved this run: {len(report['evidence_bytes_moved_during_this_run'])}, "
            f"uncommitted: {len(report['uncommitted_evidence_changes'])}"
        )
        print(f"states demonstrated: {report['demonstrated_states']}")
        print(f"positive controls accepted: {report['accepted_positive_controls']}")
    if not report["matched"]:
        message = (
            "a load-bearing check was removed and the census did not notice"
            if arguments.mutation_probes
            else "legacy compatibility census did not match"
        )
        print(message, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
