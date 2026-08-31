#!/usr/bin/env python3
"""Behavior tests for the strict traceability coverage policy gate."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "check_traceability_coverage", ROOT / "scripts" / "check_traceability_coverage.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def complete_report() -> dict[str, object]:
    return {
        "totals": {"backed": 2, "total": 2},
        "groups": [{"document": "spec/test-matrix.md", "backed": 2, "total": 2}],
        "binding_census": [
            {"language": "rust", "candidates": 2, "tagged": 2, "bound": 2}
        ],
        "unbacked_rows": [],
        "status_lies": [],
        "untracked_symbols": [],
    }


def main() -> int:
    assert MODULE.validate_report(complete_report()) == []
    for mutate in (
        lambda value: value.update({"totals": {"backed": 1, "total": 2}}),
        lambda value: value["groups"][0].update({"backed": 1}),
        lambda value: value["binding_census"][0].update({"tagged": 1}),
        lambda value: value.update({"status_lies": [{"id": "TC-fabricated"}]}),
    ):
        report = complete_report()
        mutate(report)
        assert MODULE.validate_report(report), "an incomplete coverage report was accepted"
    print("strict traceability coverage behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
