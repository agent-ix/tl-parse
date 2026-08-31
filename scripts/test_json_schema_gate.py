#!/usr/bin/env python3
"""Behavioral self-test for required JSON Schema format enforcement."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    with tempfile.TemporaryDirectory() as temporary:
        directory = Path(temporary)
        schema = directory / "schema.json"
        instance = directory / "instance.json"
        schema.write_text(
            json.dumps({"$schema": "http://json-schema.org/draft-07/schema#", "type": "string", "format": "date-time"}),
            encoding="utf-8",
        )
        instance.write_text(json.dumps("not-a-date-time"), encoding="utf-8")
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "validate_json_schema.py"), str(schema), str(instance)],
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode not in {1, 125}:
            print("required format validation falsely passed", file=sys.stderr)
            return 1
        if result.returncode == 125 and "unavailable" not in result.stderr:
            print("missing format checker was not classified unavailable", file=sys.stderr)
            return 1
    print("required JSON Schema format behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
