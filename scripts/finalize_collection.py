#!/usr/bin/env python3
"""Write the post-envelope validation summary for a retained collection."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


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


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: finalize_collection.py EVIDENCE_DIR", file=sys.stderr)
        return 2
    evidence_dir = Path(sys.argv[1])
    outcomes = []
    for name in CHECKS:
        status_path = evidence_dir / f"{name}.status.txt"
        stdout_path = evidence_dir / f"{name}.stdout"
        if not status_path.exists():
            outcomes.append({"name": name, "status": "inconclusive", "exitCode": None})
            continue
        exit_code = int(status_path.read_text(encoding="utf-8").strip())
        skipped = (
            stdout_path.exists()
            and stdout_path.read_text(encoding="utf-8").strip() == "skipped-unavailable"
        )
        outcomes.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if skipped
                    else "passed" if exit_code == 0 else "failed"
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
    value = {
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
    (evidence_dir / "collection-summary.json").write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
