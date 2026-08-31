#!/usr/bin/env python3
"""Behavior tests for the strict traceability coverage policy gate."""

from __future__ import annotations

import importlib.util
import tempfile
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
        "diagnostics": [],
    }


def main() -> int:
    assert MODULE.validate_report(complete_report()) == []
    for mutate in (
        lambda value: value.update({"totals": {"backed": 1, "total": 2}}),
        lambda value: value["groups"][0].update({"backed": 1}),
        lambda value: value["binding_census"][0].update({"tagged": 1}),
        lambda value: value.update({"status_lies": [{"id": "TC-fabricated"}]}),
        lambda value: value.update({"diagnostics": [{"reason": "uncatalogued-skip"}]}),
    ):
        report = complete_report()
        mutate(report)
        assert MODULE.validate_report(report), "an incomplete coverage report was accepted"
    with tempfile.TemporaryDirectory() as directory:
        matrix = Path(directory) / "matrix.md"
        matrix.write_text(
            "## Functional Requirement Coverage\n\n"
            "| Functional Req | Coverage Status |\n|---|---|\n| FR-001 | fabricated |\n",
            encoding="utf-8",
        )
        assert MODULE.validate_matrix_statuses(matrix)
    requirement = ROOT / "spec" / "requirements" / "FR-004-canonical-format.md"
    original = requirement.read_text(encoding="utf-8")
    try:
        requirement.write_text(original.replace("TC-017", "TC-999"), encoding="utf-8")
        assert MODULE.validate_verification_references()
    finally:
        requirement.write_text(original, encoding="utf-8")
    print("strict traceability coverage behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
