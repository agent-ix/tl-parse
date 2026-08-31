#!/usr/bin/env python3
"""Behavior tests for the active evidence-collection marker."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "scripts" / "collection_marker.py"


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, check=True,
        capture_output=True, text=True,
    ).stdout.strip()
    with tempfile.TemporaryDirectory() as directory:
        marker = Path(directory) / ".collecting"
        marker.write_text(
            json.dumps({"pid": os.getppid(), "sourceRevision": revision}) + "\n",
            encoding="utf-8",
        )
        value = subprocess.run(
            [sys.executable, str(SCRIPT), "check", str(marker)], cwd=ROOT, check=False,
            capture_output=True,
        )
        assert value.returncode != 0, "a non-collector ancestor hid an evidence directory"
        marker.write_text("{}\n", encoding="utf-8")
        value = subprocess.run(
            [sys.executable, str(SCRIPT), "check", str(marker)], cwd=ROOT, check=False,
            capture_output=True,
        )
        assert value.returncode != 0, "marker checker exit contract accepted malformed input"
    print("collection marker behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
