#!/usr/bin/env python3
"""Prove every mandatory local-CI prerequisite propagates failures."""

from __future__ import annotations

import argparse
import os
import pwd
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
SHELL_CONTROL = re.compile(r"&&|\|\||&(?!&)|[;|]")
ATTRIBUTE_IGNORE = re.compile(r"#\s*\[\s*(?:ignore\b|cfg_attr\([^\]]*,\s*ignore\b)")
MAKEFLAGS_ASSIGNMENT = re.compile(
    r"^\s*(?:(?:export|override|unexport)\s+)?MAKEFLAGS\s*(?::|\+|\?)?="
)


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
    for source in root.rglob("*.rs"):
        if ".git" in source.parts or "target" in source.parts:
            continue
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
        "ASAN_OPTIONS",
    ):
        environment.pop(name, None)
    return environment


def verify_tool_identities() -> list[str]:
    errors: list[str] = []
    real_home = Path(pwd.getpwuid(os.getuid()).pw_dir)
    expected = {
        "bash": "/usr/bin/bash",
        "cargo": str(real_home / ".cargo" / "bin" / "cargo"),
        "git": "/usr/bin/git",
        "make": "/usr/bin/make",
        "python3": "/usr/bin/python3",
        "sha256sum": "/usr/bin/sha256sum",
    }
    for name, path in expected.items():
        if shutil.which(name) != path:
            errors.append(f"{name} must resolve to {path}, got {shutil.which(name)}")
    quire = shutil.which("quire")
    prefixes = (real_home / ".npm-global" / "bin", Path("/opt/hostedtoolcache/node"))
    if quire is None or not any(Path(quire).is_relative_to(prefix) for prefix in prefixes):
        errors.append(f"quire must resolve under a declared npm tool prefix, got {quire}")
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
    if os.environ.get("ASAN_OPTIONS"):
        errors.append("ambient ASAN_OPTIONS is not permitted for local CI")
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
