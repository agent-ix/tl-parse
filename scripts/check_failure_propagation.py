#!/usr/bin/env python3
"""Prove every mandatory local-CI prerequisite propagates failures."""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

import tool_identity


ROOT = Path(__file__).resolve().parent.parent
PROBES = {
    "fmt-check", "lint", "test", "check-corpus", "fuzz-build", "fuzz-smoke",
    "deny", "audit-unsafe", "evidence-tool", "spec", "msrv", "rustdoc",
    "verify-evidence", "rust-test-census",
}
COLLECTION_PROBES = PROBES - {"verify-evidence"}
GUARD_TARGET = "check-failure-propagation"
TARGET = re.compile(r"^([A-Za-z0-9_.-]+):(?:\s+(.*?))?\s*$")
SHELL_CONTROL = re.compile(r"&&|\|\||&(?!&)|[;|]")
ATTRIBUTE_IGNORE = re.compile(r"#\s*\[\s*(?:ignore\b|cfg_attr\([^\]]*,\s*ignore\b)")
DISABLED_CRATE_OR_MODULE = re.compile(r"#!\s*\[\s*cfg\s*\([^\]]*\)\s*\]")
DANGEROUS_ASSIGNMENT = re.compile(
    r"^\s*(?:(?:export|override|unexport|private)\s+)*"
    r"(?:SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)\s*[:+?!]*="
)
DANGEROUS_DEFINE = re.compile(
    r"^\s*(?:(?:export|override|private)\s+)*define\s+"
    r"(?:SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)(?:\s|$)"
)
EVAL = re.compile(r"\$\(\s*eval(?:\s|$)")
TARGET_SCOPED_ASSIGNMENT = re.compile(
    r"^\s*(?:[^\s:=]+(?:\s+[^\s:=]+)*)\s*:{1,2}\s*"
    r"(?:(?:export|override|unexport|private)\s+)*"
    r"(?:SHELL|\.SHELLFLAGS|MAKE|MAKEFLAGS)\s*[:+?!]*="
)
MAKEFILE_IMPORT = re.compile(r"^\s*(?:-?include|sinclude)\s+")


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
        source_text = source.read_text(encoding="utf-8")
        if ATTRIBUTE_IGNORE.search(source_text):
            errors.append(f"{source.relative_to(root)} disables a Rust test with ignore")
        if DISABLED_CRATE_OR_MODULE.search(source_text):
            errors.append(f"{source.relative_to(root)} has a crate/module-level cfg exclusion")
    return errors


def inspect_makefile(path: Path, root: Path = ROOT) -> list[str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    errors: list[str] = []
    defined = recipes(path)
    expected_composites = {
        "ci": ["/usr/bin/python3 scripts/run_local_ci.py --include-verify"],
        "ci-for-evidence": ["/usr/bin/python3 scripts/run_local_ci.py"],
    }
    for target, expected_recipes in expected_composites.items():
        if defined.get(target) != expected_recipes:
            errors.append(
                f"{target} runner drift: expected {expected_recipes}, "
                f"observed {defined.get(target, [])}"
            )
    for number, line in enumerate(lines, start=1):
        if re.match(r"^\s*\.(?:IGNORE|SILENT|ONESHELL|DEFAULT)\s*(?::|$)", line):
            errors.append(f"Makefile:{number} declares a recipe-control directive")
        if DANGEROUS_ASSIGNMENT.match(line) or DANGEROUS_DEFINE.match(line):
            errors.append(f"Makefile:{number} overrides a mandatory Make execution control")
        if EVAL.search(line):
            errors.append(f"Makefile:{number} uses forbidden dynamic eval")
        if TARGET_SCOPED_ASSIGNMENT.match(line):
            errors.append(f"Makefile:{number} applies a target-scoped Make execution override")
        if MAKEFILE_IMPORT.match(line):
            errors.append(f"Makefile:{number} imports unchecked Make execution controls")
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
    selected, profile, tools = tool_identity.load_lock(
        profile_name=os.environ.get("TL_PARSE_TOOL_PROFILE")
    )
    return tool_identity.qualified_environment(selected, profile, tools)


def verify_tool_identities() -> list[str]:
    try:
        selected, profile, tools = tool_identity.load_lock(
            profile_name=os.environ.get("TL_PARSE_TOOL_PROFILE")
        )
        unavailable, mismatches = tool_identity.verify_live(selected, profile, tools)
    except (OSError, ValueError) as error:
        return [f"cannot load qualified tool identities: {error}"]
    return unavailable + mismatches


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
    try:
        selected, _, _ = tool_identity.load_lock(
            profile_name=os.environ.get("TL_PARSE_TOOL_PROFILE")
        )
    except (OSError, ValueError) as error:
        print(f"cannot select qualification profile: {error}", file=sys.stderr)
        return 1
    print(f"qualification profile: {selected}")
    print(f"all {len(PROBES)} mandatory local-CI targets propagate failures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
