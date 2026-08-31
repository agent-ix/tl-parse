#!/usr/bin/env python3
"""Verify the executable identities used for local qualification."""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "tools.lock"
REQUIRED = ("bash", "cargo", "git", "make", "python3", "quire", "rustc", "sha256sum")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_lock(value: Any) -> dict[str, dict[str, str]]:
    if not isinstance(value, dict) or value.get("schemaVersion") != "tl-parse.qualified-tools/v1":
        raise ValueError("tools.lock has an unknown schema")
    tools = value.get("tools")
    if not isinstance(tools, dict) or set(tools) != set(REQUIRED):
        raise ValueError("tools.lock does not contain the exact mandatory-tool census")
    validated: dict[str, dict[str, str]] = {}
    for name in REQUIRED:
        identity = tools.get(name)
        if not isinstance(identity, dict) or set(identity) != {"path", "sha256"}:
            raise ValueError(f"tools.lock has a malformed identity for {name}")
        path = identity.get("path")
        digest = identity.get("sha256")
        if not isinstance(path, str) or not Path(path).is_absolute():
            raise ValueError(f"tools.lock path for {name} is not absolute")
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError(f"tools.lock digest for {name} is malformed")
        validated[name] = {"path": path, "sha256": digest}
    environment = value.get("environment")
    if not isinstance(environment, dict) or environment.get("home") != "/home/peter":
        raise ValueError("tools.lock has an unknown qualification home")
    return validated


def load_lock(path: Path = LOCK) -> tuple[dict[str, Any], dict[str, dict[str, str]]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    return value, validate_lock(value)


def trusted_path(tools: dict[str, dict[str, str]]) -> str:
    parents: list[str] = []
    for name in REQUIRED:
        parent = str(Path(tools[name]["path"]).parent)
        if parent not in parents:
            parents.append(parent)
    return ":".join(parents)


def qualified_environment(value: dict[str, Any], tools: dict[str, dict[str, str]]) -> dict[str, str]:
    environment = dict(os.environ)
    environment["HOME"] = value["environment"]["home"]
    environment["PATH"] = trusted_path(tools)
    for name in (
        "MAKE", "MAKEFLAGS", "CARGO", "PYTHON", "QUIRE", "SHA256SUM", "BASH",
        "PYTHONOPTIMIZE", "ASAN_OPTIONS", "TL_PARSE_FUZZ_DISABLE_LEAKS",
    ):
        environment.pop(name, None)
    return environment


def verify_live(
    value: dict[str, Any], tools: dict[str, dict[str, str]]
) -> tuple[list[str], list[str]]:
    unavailable: list[str] = []
    mismatches: list[str] = []
    environment = qualified_environment(value, tools)
    for name in REQUIRED:
        expected = tools[name]
        locked_path = Path(expected["path"])
        try:
            locked_digest = sha256(locked_path)
        except OSError as error:
            unavailable.append(f"cannot read locked tool {name}: {error}")
            continue
        if locked_digest != expected["sha256"]:
            mismatches.append(
                f"locked tool digest mismatch for {name}: expected {expected['sha256']}, got {locked_digest}"
            )
            continue
        observed = shutil.which(name, path=environment["PATH"])
        if observed is None:
            unavailable.append(f"qualified tool is unavailable: {name}")
            continue
        try:
            observed_digest = sha256(Path(observed))
        except OSError as error:
            unavailable.append(f"cannot read qualified tool {name}: {error}")
            continue
        if observed_digest != expected["sha256"]:
            mismatches.append(
                f"qualified tool digest mismatch for {name}: expected {expected['sha256']}, "
                f"got {observed_digest} at {observed}"
            )
    return unavailable, mismatches


def main() -> int:
    simple_options = {"--verify-live", "--trusted-path", "--home"}
    tool_options = {"--tool-path", "--tool-sha256"}
    if not (
        (len(sys.argv) == 2 and sys.argv[1] in simple_options)
        or (len(sys.argv) == 3 and sys.argv[1] in tool_options and sys.argv[2] in REQUIRED)
    ):
        print(
            "usage: tool_identity.py {--verify-live|--trusted-path|--home|"
            "--tool-path NAME|--tool-sha256 NAME}",
            file=sys.stderr,
        )
        return 2
    try:
        value, tools = load_lock()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"qualified tool lock is unavailable: {error}", file=sys.stderr)
        return 2
    if sys.argv[1] == "--trusted-path":
        print(trusted_path(tools))
        return 0
    if sys.argv[1] == "--home":
        print(value["environment"]["home"])
        return 0
    if sys.argv[1] == "--tool-path":
        print(tools[sys.argv[2]]["path"])
        return 0
    if sys.argv[1] == "--tool-sha256":
        print(tools[sys.argv[2]]["sha256"])
        return 0
    unavailable, mismatches = verify_live(value, tools)
    for error in unavailable + mismatches:
        print(error, file=sys.stderr)
    if unavailable:
        return 2
    return 1 if mismatches else 0


if __name__ == "__main__":
    raise SystemExit(main())
