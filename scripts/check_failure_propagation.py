#!/usr/bin/env python3
"""Prove every mandatory local-CI prerequisite propagates failures."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
PROBES = {
    "fmt-check", "lint", "test", "check-corpus", "fuzz-build", "fuzz-smoke",
    "deny", "audit-unsafe", "evidence-tool", "spec", "msrv", "rustdoc",
    "verify-evidence",
}
GUARD_TARGET = "check-failure-propagation"
TARGET = re.compile(r"^([A-Za-z0-9_.-]+):(?:\s+(.*?))?\s*$")
SHELL_CONTROL = re.compile(r"&&|\|\||[;|]")
ATTRIBUTE_IGNORE = re.compile(r"#\s*\[\s*(?:ignore\b|cfg_attr\([^\]]*,\s*ignore\b)")
MAKEFLAGS_ASSIGNMENT = re.compile(r"^\s*MAKEFLAGS\s*(?::|\+|\?)?=")


def recipes(path: Path) -> dict[str, list[str]]:
    result: dict[str, list[str]] = {}
    current: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        match = TARGET.fullmatch(line)
        if match and not line.startswith(("\t", ".")):
            current = match.group(1).split()
            for name in current:
                result.setdefault(name, [])
        elif line.startswith("\t"):
            for name in current:
                result.setdefault(name, []).append(line[1:])
        elif line and not line.startswith((" ", "#")):
            current = []
    return result


def inspect_ignored_tests(root: Path) -> list[str]:
    errors: list[str] = []
    for directory in ("src", "tests", "fuzz"):
        base = root / directory
        if not base.exists():
            continue
        for source in base.rglob("*.rs"):
            if ATTRIBUTE_IGNORE.search(source.read_text(encoding="utf-8")):
                errors.append(f"{source.relative_to(root)} disables a Rust test with ignore")
    return errors


def inspect_makefile(path: Path, root: Path = ROOT) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []
    ci = next((line for line in lines if line.startswith("ci:")), "")
    observed = set(ci.removeprefix("ci:").split())
    expected = PROBES | {GUARD_TARGET}
    if observed != expected:
        errors.append(
            f"ci prerequisite census drift: expected {sorted(expected)}, observed {sorted(observed)}"
        )
    for number, line in enumerate(lines, start=1):
        if re.match(r"^\s*\.(?:IGNORE|SILENT)\s*(?::|$)", line):
            errors.append(f"Makefile:{number} declares a recipe-control directive")
        if MAKEFLAGS_ASSIGNMENT.match(line):
            errors.append(f"Makefile:{number} assigns MAKEFLAGS")
        if not line.startswith("\t"):
            continue
        recipe = line[1:].lstrip("@ ")
        if recipe.startswith("-"):
            errors.append(f"Makefile:{number} ignores a recipe failure")
        if SHELL_CONTROL.search(recipe):
            errors.append(f"Makefile:{number} uses a forbidden shell control operator")
    errors.extend(inspect_ignored_tests(root))
    return errors


def clean_environment() -> dict[str, str]:
    environment = dict(os.environ)
    for name in (
        "MAKEFLAGS", "CARGO", "PYTHON", "QUIRE", "SHA256SUM", "BASH",
        "PYTHONOPTIMIZE", "TL_PARSE_FUZZ_DISABLE_LEAKS",
    ):
        environment.pop(name, None)
    return environment


def verify_tool_identities() -> list[str]:
    errors: list[str] = []
    expected_cargo = Path.home() / ".cargo" / "bin" / "cargo"
    observed_cargo = shutil.which("cargo")
    if observed_cargo is None or Path(observed_cargo) != expected_cargo:
        errors.append(
            f"cargo must resolve to the rustup-managed wrapper {expected_cargo}, got {observed_cargo}"
        )
    identities = {
        "cargo": ("--version", re.compile(r"^cargo \d")),
        "python3": ("--version", re.compile(r"^Python \d")),
        "quire": ("--version", re.compile(r"^quire \d")),
        "sha256sum": ("--version", re.compile(r"^sha256sum \(GNU coreutils\)")),
        "bash": ("--version", re.compile(r"^GNU bash, version \d")),
    }
    for command, (argument, identity) in identities.items():
        try:
            value = subprocess.run(
                [command, argument], check=False, capture_output=True, text=True,
                env=clean_environment(),
            )
        except FileNotFoundError:
            errors.append(f"required tool is unavailable: {command}")
            continue
        output = (value.stdout + value.stderr).strip()
        if value.returncode != 0 or identity.search(output) is None:
            errors.append(f"required tool did not self-identify as {command}: {output!r}")
    return errors


def inspect_expanded_recipes(makefile: Path, root: Path = ROOT) -> list[str]:
    errors: list[str] = []
    for target in sorted(PROBES):
        result = subprocess.run(
            ["make", "--no-print-directory", "-n", "-f", str(makefile), target],
            cwd=root, check=False, capture_output=True, text=True,
            env=clean_environment(),
        )
        if result.returncode != 0:
            errors.append(f"cannot expand mandatory target {target}: {result.stderr.strip()}")
            continue
        for command in result.stdout.splitlines():
            if SHELL_CONTROL.search(command):
                errors.append(
                    f"expanded mandatory target {target} uses forbidden shell control operators: {command}"
                )
    return errors


def probe_command_positions(makefile: Path) -> list[str]:
    errors: list[str] = []
    defined = recipes(makefile)
    with tempfile.TemporaryDirectory() as directory:
        probe = Path(directory) / "Makefile"
        for target in sorted(PROBES):
            commands = defined.get(target, [])
            for selected in range(len(commands)):
                lines = [f".PHONY: {target}", f"{target}:"]
                for index, command in enumerate(commands):
                    if index != selected:
                        lines.append("\ttrue")
                        continue
                    stripped = command.lstrip("@ ")
                    make_prefix = "-" if stripped.startswith("-") else ""
                    shell_suffix = ""
                    match = SHELL_CONTROL.search(stripped)
                    if match is not None:
                        shell_suffix = stripped[match.start():]
                    lines.append(f"\t{make_prefix}false{shell_suffix}")
                probe.write_text("\n".join(lines) + "\n", encoding="utf-8")
                value = subprocess.run(
                    ["make", "--no-print-directory", "-f", str(probe), target],
                    cwd=ROOT, check=False, stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL, env=clean_environment(),
                )
                if value.returncode == 0:
                    errors.append(
                        f"mandatory target {target} swallowed failure at command {selected + 1}"
                    )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--makefile", type=Path, default=ROOT / "Makefile")
    parser.add_argument("--inspect-only", action="store_true")
    parser.add_argument("--static-only", action="store_true")
    args = parser.parse_args()
    makefile = args.makefile.resolve()
    errors = inspect_makefile(makefile)
    if os.environ.get("MAKEFLAGS"):
        errors.append("ambient MAKEFLAGS is not permitted for local CI")
    if os.environ.get("PYTHONOPTIMIZE") or sys.flags.optimize:
        errors.append("optimized Python disables policy assertions")
    if not errors and not args.static_only:
        errors.extend(inspect_expanded_recipes(makefile))
    if not errors and not args.inspect_only and not args.static_only:
        errors.extend(verify_tool_identities())
        errors.extend(probe_command_positions(makefile))
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print(f"all {len(PROBES)} mandatory local-CI targets propagate failures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
