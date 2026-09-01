#!/usr/bin/env python3
"""Resolve active and explicitly retracted evidence qualification profiles."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import tool_identity


ROOT = Path(__file__).resolve().parent.parent
PROFILE = "tl-parse.evidence-qualification/v2"
RETRACTIONS = ROOT / "evidence" / "RETRACTIONS.json"


def retracted_records() -> set[str]:
    value = json.loads(RETRACTIONS.read_text(encoding="utf-8"))
    if value.get("schemaVersion") != "tl-parse.evidence-retractions/v1":
        raise ValueError("evidence retraction registry has an unknown schema")
    records = value.get("records")
    if not isinstance(records, dict) or not all(
        isinstance(name, str)
        and isinstance(item, dict)
        and isinstance(item.get("reason"), str)
        and item["reason"]
        for name, item in records.items()
    ):
        raise ValueError("evidence retraction registry has a malformed record map")
    return set(records)


def resolve_profile(evidence_dir: Path) -> str:
    if evidence_dir.name in retracted_records():
        return "retracted"
    value = json.loads((evidence_dir / "collection-input.json").read_text(encoding="utf-8"))
    if value.get("qualificationProfile") != PROFILE:
        return "inconclusive"
    revision = (evidence_dir / "source-revision.txt").read_text(encoding="utf-8").strip()
    result = subprocess.run(
        ["/usr/bin/git", "cat-file", "-e", f"{revision}:tools.lock"], cwd=ROOT,
        check=False, capture_output=True,
    )
    if result.returncode != 0:
        return "inconclusive"
    try:
        lock_value = json.loads(
            subprocess.run(
                ["/usr/bin/git", "show", f"{revision}:tools.lock"],
                cwd=ROOT,
                check=True,
                capture_output=True,
                text=True,
            ).stdout
        )
    except (json.JSONDecodeError, subprocess.CalledProcessError):
        return "unsupported-lock-schema"
    return (
        "v2"
        if lock_value.get("schemaVersion") == tool_identity.SCHEMA
        else "unsupported-lock-schema"
    )


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: evidence_profile.py EVIDENCE_DIR", file=sys.stderr)
        return 2
    try:
        print(resolve_profile(Path(sys.argv[1])))
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"cannot resolve evidence profile: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
