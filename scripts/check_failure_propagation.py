#!/usr/bin/env python3
"""Prove every mandatory local-CI prerequisite propagates a real failure."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
EXPECTED = {
    "fmt-check": "CARGO",
    "lint": "CARGO",
    "test": "CARGO",
    "check-corpus": "SHA256SUM",
    "fuzz-build": "CARGO",
    "fuzz-smoke": "BASH",
    "deny": "CARGO",
    "audit-unsafe": "BASH",
    "evidence-tool": "PYTHON",
    "spec": "QUIRE",
    "msrv": "CARGO",
    "rustdoc": "CARGO",
    "verify-evidence": "BASH",
}


def inspect(makefile: Path) -> list[str]:
    lines = makefile.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []
    ci = next((line for line in lines if line.startswith("ci:")), "")
    prerequisites = set(ci.removeprefix("ci:").split())
    required = set(EXPECTED) | {"check-failure-propagation"}
    if prerequisites != required:
        errors.append(
            f"ci prerequisite census drift: expected {sorted(required)}, observed {sorted(prerequisites)}"
        )
    for number, line in enumerate(lines, start=1):
        if not line.startswith("\t"):
            continue
        recipe = line[1:].lstrip("@")
        if recipe.startswith("-"):
            errors.append(f"Makefile:{number} ignores a recipe failure")
        if re.search(r"\|\|\s*true(?:\s|$)|;\s*true(?:\s|$)", recipe):
            errors.append(f"Makefile:{number} contains a false-success command")
    for path in [ROOT / "src", ROOT / "tests", ROOT / "fuzz"]:
        for source in path.rglob("*.rs"):
            if re.search(r"#\s*\[\s*ignore(?:\s|\]|\()", source.read_text(encoding="utf-8")):
                errors.append(f"{source.relative_to(ROOT)} disables a Rust test with #[ignore]")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--makefile", type=Path, default=ROOT / "Makefile")
    parser.add_argument("--inspect-only", action="store_true")
    args = parser.parse_args()
    errors = inspect(args.makefile)
    if not args.inspect_only and not errors:
        for target, variable in EXPECTED.items():
            result = subprocess.run(
                ["make", "--no-print-directory", "-f", str(args.makefile), target, f"{variable}=false"],
                cwd=ROOT,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            if result.returncode == 0:
                errors.append(f"{target} swallowed a deliberately failing {variable} command")
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print(f"all {len(EXPECTED)} mandatory local-CI targets propagate failures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
