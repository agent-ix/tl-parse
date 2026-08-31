#!/usr/bin/env python3
"""Prove every mandatory local-CI prerequisite propagates a real failure."""

from __future__ import annotations

import argparse
import os
import re
import shlex
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
TOOL_IDENTITIES = {
    "CARGO": ("cargo", "--version", re.compile(r"^cargo \d")),
    "PYTHON": ("python3", "--version", re.compile(r"^Python \d")),
    "QUIRE": ("quire", "--version", re.compile(r"^quire \d")),
    "SHA256SUM": (
        "sha256sum",
        "--version",
        re.compile(r"^sha256sum \(GNU coreutils\)"),
    ),
    "BASH": ("bash", "--version", re.compile(r"^GNU bash, version \d")),
}
ATTRIBUTE_IGNORE = re.compile(r"#\s*\[[^\]]*\bignore\b[^\]]*\]")


def makeflags_ignore_errors(value: str) -> bool:
    try:
        tokens = shlex.split(value)
    except ValueError:
        return True
    return any(
        token == "--ignore-errors"
        or (token.startswith("-") and not token.startswith("--") and "i" in token[1:])
        or (token and not token.startswith("-") and "=" not in token and "i" in token)
        for token in tokens
    )


def verify_tool_identities() -> list[str]:
    errors: list[str] = []
    for variable, (default, argument, identity) in TOOL_IDENTITIES.items():
        command = os.environ.get(variable, default)
        try:
            result = subprocess.run(
                [command, argument], check=False, capture_output=True, text=True
            )
        except FileNotFoundError:
            errors.append(f"required tool is unavailable: {variable}={command}")
            continue
        output = (result.stdout + result.stderr).strip()
        if result.returncode != 0 or identity.search(output) is None:
            errors.append(f"required tool did not self-identify as {default}: {output!r}")
    return errors


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
        if re.match(r"^\s*\.(?:IGNORE|SILENT)\s*(?::|$)", line):
            errors.append(f"Makefile:{number} declares a global recipe-control directive")
        if re.match(r"^\s*MAKEFLAGS\s*(?::|\+|\?)?=", line) and makeflags_ignore_errors(
            line.split("=", 1)[1]
        ):
            errors.append(f"Makefile:{number} enables MAKEFLAGS ignore-errors")
        if not line.startswith("\t"):
            continue
        recipe = line[1:].lstrip("@")
        if recipe.startswith("-"):
            errors.append(f"Makefile:{number} ignores a recipe failure")
        if re.search(r"\|\|\s*true(?:\s|$)|;\s*true(?:\s|$)", recipe):
            errors.append(f"Makefile:{number} contains a false-success command")
    for path in [ROOT / "src", ROOT / "tests", ROOT / "fuzz"]:
        for source in path.rglob("*.rs"):
            if ATTRIBUTE_IGNORE.search(source.read_text(encoding="utf-8")):
                errors.append(f"{source.relative_to(ROOT)} disables a Rust test with #[ignore]")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--makefile", type=Path, default=ROOT / "Makefile")
    parser.add_argument("--inspect-only", action="store_true")
    args = parser.parse_args()
    errors = inspect(args.makefile)
    if makeflags_ignore_errors(os.environ.get("MAKEFLAGS", "")):
        errors.append("ambient MAKEFLAGS enables ignored recipe failures")
    if not args.inspect_only and not errors:
        errors.extend(verify_tool_identities())
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
