#!/usr/bin/env python3
"""Observe the local toolchain and let Engineering Assurance classify it (FR-006-AC-1).

Four things this file deliberately is not.

It is not a copy of the compatibility matrix. It never says which version of
anything is correct. It observes what is installed and hands every verdict to
`engineering-assurance compatibility`, because a second copy of the rule is a
second authority, and two authorities drift.

It is not an acceptance authority. The native EA compatibility result carries
both version classification and human acceptance; this gate requires both.
The candidate v0.4.1 matrix withholds until its owner records acceptance.

It is not a network probe. It does not ask a registry whether a release landed.
`npm.ix` in particular is a mirror that lags the public registry and is not an
oracle for anything; the only thing this script does about it is refuse to find
it written down anywhere in this repository.

It is not an envelope. It prints a report and exits. It retains nothing.

Exit status: 0 when the EA gate is satisfied and no local check fails,
1 when the EA gate or a local check is unsatisfied, 2 when the EA CLI cannot
be used — which is a different fact from a failing check and gets its own code.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
PINS_PATH = ROOT / "assurance" / "pins.json"

FORBIDDEN_REGISTRY = "npm.ix"

# Files a mirror reference could realistically hide in. Read line by line rather
# than grepped as a blob, so that pins.json's own prose about the mirror does not
# match itself and report a violation that is actually the rule being written down.
MIRROR_SCAN_FILES = (
    ".npmrc",
    "Cargo.toml",
    "Cargo.lock",
    "fuzz/Cargo.toml",
    "fuzz/Cargo.lock",
    "package.json",
    "package-lock.json",
    ".github/workflows/ci.yml",
)


class PinError(RuntimeError):
    """The pinned assurance distribution could not be used."""


def observe(argv: list[str]) -> str | None:
    """Run a version probe. An absent tool is None, which upstream calls unknown."""
    try:
        result = subprocess.run(argv, capture_output=True, text=True, check=False)
    except (OSError, ValueError):
        return None
    if result.returncode != 0:
        return None
    value = result.stdout.strip()
    return value or None


def observe_quire() -> str | None:
    """Read the CLI version from quire's own provenance record, not its banner."""
    raw = observe(["quire", "provenance"])
    if raw is None:
        return None
    try:
        return str(json.loads(raw)["cli"]["version"])
    except (json.JSONDecodeError, KeyError, TypeError):
        return None


def observe_engineering_assurance() -> str | None:
    raw = observe(["engineering-assurance", "--version"])
    if raw is None:
        return None
    match = re.search(r"\b(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\b", raw)
    return match.group(1) if match else None


def classify_with_ea(observed: dict[str, str | None]) -> dict[str, Any]:
    request = {
        "protocol": "engineering-assurance.compatibility-request/v1",
        "observed": [
            {"component": name, "version": version}
            for name, version in observed.items()
        ],
    }
    try:
        result = subprocess.run(
            ["engineering-assurance", "compatibility"],
            input=json.dumps(request), capture_output=True, text=True, check=False,
        )
    except (OSError, ValueError) as error:
        raise PinError(f"the pinned EA CLI is unavailable: {error}") from error
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise PinError(f"EA compatibility returned non-JSON: {error}; {result.stderr}") from error
    if not isinstance(report, dict) or result.returncode not in (0, 1) or report.get("protocol") != "engineering-assurance.compatibility-result/v1":
        raise PinError(f"EA compatibility failed: exit {result.returncode}; {report}; {result.stderr}")
    if result.returncode != (0 if report.get("gate_satisfied") else 1):
        raise PinError("EA compatibility exit status disagrees with gate_satisfied")
    return report


def mirror_references(pins: dict[str, Any]) -> list[str]:
    """Find any place this repository would resolve a component from the mirror."""
    offenders: list[str] = []
    for name in MIRROR_SCAN_FILES:
        path = ROOT / name
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for number, line in enumerate(text.splitlines(), start=1):
            if FORBIDDEN_REGISTRY in line:
                offenders.append(f"{name}:{number}")
    # pins.json is inspected structurally: its prose says the mirror's name on
    # purpose, and matching that would be the check reporting its own statement.
    requirement = pins["engineering_assurance"]["requirement"]
    if FORBIDDEN_REGISTRY in requirement:
        offenders.append("assurance/pins.json:engineering_assurance.requirement")
    for artifact in pins["consumed_artifacts"]:
        if FORBIDDEN_REGISTRY in artifact["path"]:
            offenders.append(f"assurance/pins.json:consumed_artifacts:{artifact['path']}")
    return offenders


def upstream_pin_mismatches(pins: dict[str, Any]) -> list[str]:
    """Check the tl-syntax pin this repository actually compiles against.

    A lockfile that drifted from `Cargo.toml`, a `TL_SYNTAX_REVISION` constant
    that still named an old pin, or active guidance that named a superseded pin
    would make the crate report a dependency identity it is not built from.
    """
    declared = pins["upstream_dependency"]["compiled_revision"]
    problems: list[str] = []
    checks = {
        "Cargo.toml": f'rev = "{declared}"',
        "Cargo.lock": f"#{declared}",
        "fuzz/Cargo.lock": f"#{declared}",
        "src/lib.rs": f'TL_SYNTAX_REVISION: &str = "{declared}"',
        "spec/assurance/AA-001.md": declared,
    }
    for name, needle in checks.items():
        path = ROOT / name
        if not path.is_file():
            problems.append(f"{name}: absent")
            continue
        if needle not in path.read_text(encoding="utf-8"):
            problems.append(f"{name}: does not name the compiled revision {declared}")
    current_records = ("README.md", "CLAUDE.md", "deny.toml", "spec/assurance/AA-001.md")
    for superseded in pins["upstream_dependency"].get("superseded_compiled_revisions", []):
        for name in current_records:
            path = ROOT / name
            if path.is_file() and superseded in path.read_text(encoding="utf-8"):
                problems.append(f"{name}: still names superseded compiled revision {superseded}")
    return problems


def build_report() -> dict[str, Any]:
    pins = json.loads(PINS_PATH.read_text(encoding="utf-8"))
    observed = {
        "quire-cli": observe_quire(),
        "quoin": observe(["quoin", "--version"]),
        "ix-flow": observe(["ix-flow", "--version"]),
        "engineering-assurance": observe_engineering_assurance(),
    }
    classification = classify_with_ea(observed)
    offenders = mirror_references(pins)
    upstream = upstream_pin_mismatches(pins)
    versions_ok = classification["versions_compatible"]
    gate_satisfied = classification["gate_satisfied"]
    return {
        "schemaVersion": "tl-parse.shared-pin-report/v1",
        "matrix_version": classification["matrix_version"],
        "acceptance_state": "accepted" if classification["human_acceptance_recorded"] else "not_recorded",
        "acceptance_recorded_here": False,
        "acceptance_authority": (
            "The matrix embedded in the exact Engineering Assurance CLI. "
            "This repository reports its result and is not a second acceptance authority."
        ),
        "versions_compatible": versions_ok,
        "human_acceptance_recorded": classification["human_acceptance_recorded"],
        "gate_satisfied": gate_satisfied,
        "mirror_references": offenders,
        "upstream_pin_mismatches": upstream,
        "accepted": gate_satisfied and not offenders and not upstream,
        "components": classification["components"],
    }


def main(argv: list[str]) -> int:
    as_json = argv[1:] == ["--json"]
    if argv[1:] and not as_json:
        print("usage: check_shared_pins.py [--json]", file=sys.stderr)
        return 2
    try:
        report = build_report()
    except PinError as error:
        print(str(error), file=sys.stderr)
        return 2
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        for item in report["components"]:
            observed = item["observed"] if item["observed"] is not None else "not observed"
            print(f"{item['component']}: {observed} -> {item['verdict']} ({item['reason']})")
        for offender in report["mirror_references"]:
            print(f"mirror registry reference: {offender}", file=sys.stderr)
        for problem in report["upstream_pin_mismatches"]:
            print(f"upstream pin disagreement: {problem}", file=sys.stderr)
        print(
            f"acceptance state recorded by the pinned release: {report['acceptance_state']} "
            "(EA native gate; requires attributed human acceptance)"
        )
        print(
            "shared pins accepted" if report["accepted"] else "shared pins NOT accepted",
            file=sys.stderr if not report["accepted"] else sys.stdout,
        )
    return 0 if report["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
