#!/usr/bin/env python3
"""Create or validate a live, revision-bound evidence collection marker."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def revision() -> str:
    return subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, check=True, capture_output=True, text=True
    ).stdout.strip()


def main() -> int:
    if len(sys.argv) != 3 or sys.argv[1] not in {"create", "check"}:
        print("usage: collection_marker.py {create|check} MARKER", file=sys.stderr)
        return 2
    marker = Path(sys.argv[2])
    token = os.environ.get("TL_PARSE_COLLECTION_TOKEN")
    if not token:
        return 1
    if sys.argv[1] == "create":
        marker.write_text(
            json.dumps(
                {"pid": os.getppid(), "sourceRevision": revision(), "token": token},
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        return 0
    try:
        value = json.loads(marker.read_text(encoding="utf-8"))
        pid = value["pid"]
        if value["token"] != token or value["sourceRevision"] != revision():
            return 1
        if not isinstance(pid, int) or pid <= 1:
            return 1
        os.kill(pid, 0)
    except (KeyError, OSError, ValueError, json.JSONDecodeError):
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
