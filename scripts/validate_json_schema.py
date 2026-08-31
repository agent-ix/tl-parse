#!/usr/bin/env python3
"""Validate one JSON instance against a Draft 7 schema."""

from __future__ import annotations

import json
import sys
from pathlib import Path

from jsonschema import Draft7Validator, FormatChecker


def display_path(parts: list[object]) -> str:
    path = "$"
    for part in parts:
        path += f"[{part}]" if isinstance(part, int) else f".{part}"
    return path


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: validate_json_schema.py SCHEMA INSTANCE", file=sys.stderr)
        return 2
    schema = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
    instance = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
    Draft7Validator.check_schema(schema)
    errors = sorted(
        Draft7Validator(schema, format_checker=FormatChecker()).iter_errors(instance),
        key=lambda error: ([str(part) for part in error.absolute_path], error.message),
    )
    result = {
        "errors": [
            {"message": error.message, "path": display_path(list(error.absolute_path))}
            for error in errors
        ],
        "valid": not errors,
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if not errors else 1


if __name__ == "__main__":
    raise SystemExit(main())
