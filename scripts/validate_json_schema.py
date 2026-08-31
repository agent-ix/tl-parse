#!/usr/bin/env python3
"""Validate one JSON instance against a Draft 7 schema."""

from __future__ import annotations

import json
import sys
from pathlib import Path

try:
    from jsonschema import Draft7Validator, FormatChecker
except ImportError as error:
    print(f"JSON Schema validation unavailable: {error}", file=sys.stderr)
    raise SystemExit(125) from error


def display_path(parts: list[object]) -> str:
    path = "$"
    for part in parts:
        path += f"[{part}]" if isinstance(part, int) else f".{part}"
    return path


def required_formats(value: object) -> set[str]:
    if isinstance(value, dict):
        found = {value["format"]} if isinstance(value.get("format"), str) else set()
        for child in value.values():
            found.update(required_formats(child))
        return found
    if isinstance(value, list):
        found: set[str] = set()
        for child in value:
            found.update(required_formats(child))
        return found
    return set()


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: validate_json_schema.py SCHEMA INSTANCE", file=sys.stderr)
        return 2
    try:
        schema = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
        instance = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
        Draft7Validator.check_schema(schema)
    except (OSError, json.JSONDecodeError, TypeError, ValueError) as error:
        print(f"JSON Schema gate error: {error}", file=sys.stderr)
        return 2
    checker = FormatChecker()
    missing = sorted(required_formats(schema) - set(checker.checkers))
    if missing:
        print(
            "JSON Schema validation unavailable: required format checker(s) missing: "
            + ", ".join(missing),
            file=sys.stderr,
        )
        return 125
    errors = sorted(
        Draft7Validator(schema, format_checker=checker).iter_errors(instance),
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
