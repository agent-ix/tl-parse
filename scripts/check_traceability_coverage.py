#!/usr/bin/env python3
"""Require complete Quire traceability coverage from its stable JSON report."""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
REFERENCE = re.compile(r"\b(?:TC|SUITE)-[0-9]{3}\b")
ALLOWED_DIAGNOSTICS = {
    "archetype-matches-nothing",
    "catch-all-universal",
    # Repository-side status validation below supplies the skipped check.
    "status-column-matches-nothing",
}


def validate_report(report: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    totals = report.get("totals")
    if not isinstance(totals, dict):
        return ["coverage report has no totals object"]
    if totals.get("backed") != totals.get("total"):
        errors.append(
            f"coverage total is incomplete: {totals.get('backed')}/{totals.get('total')} backed"
        )

    groups = report.get("groups")
    if not isinstance(groups, list) or not groups:
        errors.append("coverage report has no document groups")
    else:
        for group in groups:
            if not isinstance(group, dict):
                errors.append("coverage report contains a malformed group")
            elif group.get("backed") != group.get("total"):
                errors.append(
                    f"{group.get('document', '<unknown>')} is incomplete: "
                    f"{group.get('backed')}/{group.get('total')} backed"
                )

    census = report.get("binding_census")
    if not isinstance(census, list) or not census:
        errors.append("coverage report has no binding census")
    else:
        for item in census:
            if not isinstance(item, dict):
                errors.append("coverage report contains a malformed binding census entry")
                continue
            candidates = item.get("candidates")
            if item.get("tagged") != candidates or item.get("bound") != candidates:
                errors.append(
                    f"{item.get('language', '<unknown>')} binding census is incomplete: "
                    f"{item.get('bound')}/{item.get('tagged')}/{candidates} "
                    "bound/tagged/candidates"
                )

    for field in ("unbacked_rows", "status_lies", "untracked_symbols"):
        findings = report.get(field)
        if not isinstance(findings, list):
            errors.append(f"coverage report has no {field} list")
        elif findings:
            errors.append(f"coverage report contains {len(findings)} {field}")
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, list):
        errors.append("coverage report has no diagnostics list")
    else:
        for diagnostic in diagnostics:
            reason = diagnostic.get("reason") if isinstance(diagnostic, dict) else None
            if reason not in ALLOWED_DIAGNOSTICS:
                errors.append(f"coverage report contains blocking diagnostic {reason!r}")
    return errors


def validate_matrix_statuses(path: Path) -> list[str]:
    errors: list[str] = []
    section = ""
    header: list[str] | None = None
    for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if line.startswith("## "):
            section = line[3:].strip()
            header = None
            continue
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if header is None:
            header = cells
            continue
        if all(set(cell) <= {"-", ":"} for cell in cells):
            continue
        status_name = "Status" if "Status" in header else "Coverage Status"
        if status_name not in header or len(cells) != len(header):
            errors.append(f"{path}:{number} has no parseable status column")
            continue
        expected = "✅ implemented" if section == "Test Case Summary" else "✅ covered"
        if cells[header.index(status_name)] != expected:
            errors.append(f"{path}:{number} does not carry required status {expected!r}")
    return errors


def validate_verification_references() -> list[str]:
    matrix = (ROOT / "spec" / "test-matrix.md").read_text(encoding="utf-8")
    suites = (ROOT / "spec" / "evidence" / "suites.md").read_text(encoding="utf-8")
    declared = set(REFERENCE.findall(matrix)) | set(REFERENCE.findall(suites))
    errors: list[str] = []
    for path in sorted((ROOT / "spec" / "requirements").glob("*.md")):
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if not line.startswith("|") or not re.search(r"-(?:AC|VC)-[0-9]+\s*\|", line):
                continue
            for target in REFERENCE.findall(line):
                if target not in declared:
                    errors.append(f"{path}:{number} references nonexistent verification target {target}")
    return errors


def load_report(path: Path | None) -> tuple[int, dict[str, Any] | None]:
    if path is not None:
        try:
            return 0, json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            print(f"cannot read coverage report {path}: {error}", file=sys.stderr)
            return 2, None
    result = subprocess.run(
        ["quire", "coverage", "--scope", ".", "--strict", "--json"],
        check=False,
        capture_output=True,
        text=True,
    )
    sys.stderr.write(result.stderr)
    if result.returncode != 0:
        return result.returncode, None
    try:
        return 0, json.loads(result.stdout)
    except json.JSONDecodeError as error:
        print(f"quire emitted invalid coverage JSON: {error}", file=sys.stderr)
        return 2, None


def main() -> int:
    if len(sys.argv) == 1:
        report_path = None
    elif len(sys.argv) == 3 and sys.argv[1] == "--report":
        report_path = Path(sys.argv[2])
    else:
        print(
            "usage: check_traceability_coverage.py [--report REPORT.json]",
            file=sys.stderr,
        )
        return 2
    status, report = load_report(report_path)
    if status != 0 or report is None:
        return status
    errors = validate_report(report)
    errors.extend(validate_matrix_statuses(ROOT / "spec" / "test-matrix.md"))
    errors.extend(validate_verification_references())
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    totals = report["totals"]
    print(f"strict traceability coverage is complete: {totals['backed']}/{totals['total']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
