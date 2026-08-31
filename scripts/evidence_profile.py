#!/usr/bin/env python3
"""Resolve active and explicitly retracted evidence qualification profiles."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


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
    return "v2" if result.returncode == 0 else "inconclusive"
