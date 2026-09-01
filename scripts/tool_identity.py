#!/usr/bin/env python3
"""Select and verify a reviewed qualification-tool profile."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parent.parent
LOCK = ROOT / "tools.lock"
SCHEMA = "tl-parse.qualified-tools/v2"
PROFILE_NAME = re.compile(r"[a-z0-9][a-z0-9._-]{0,63}")
REQUIRED = ("bash", "cargo", "git", "make", "python3", "quire", "rustc", "sha256sum")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_profile(name: str, value: Any) -> tuple[dict[str, Any], dict[str, dict[str, str]]]:
    if not PROFILE_NAME.fullmatch(name):
        raise ValueError(f"invalid qualification profile name: {name!r}")
    if not isinstance(value, dict) or set(value) != {"environment", "tools"}:
        raise ValueError(f"qualification profile {name!r} has malformed fields")
    tools = value.get("tools")
    if not isinstance(tools, dict) or set(tools) != set(REQUIRED):
        raise ValueError(f"qualification profile {name!r} lacks the exact tool census")
    validated: dict[str, dict[str, str]] = {}
    for tool_name in REQUIRED:
        identity = tools.get(tool_name)
        if not isinstance(identity, dict) or set(identity) != {"path", "sha256"}:
            raise ValueError(f"qualification profile {name!r} has a malformed {tool_name} identity")
        path = identity.get("path")
        digest = identity.get("sha256")
        if not isinstance(path, str) or not Path(path).is_absolute():
            raise ValueError(f"qualification profile {name!r} path for {tool_name} is not absolute")
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
            raise ValueError(f"qualification profile {name!r} digest for {tool_name} is malformed")
        validated[tool_name] = {"path": path, "sha256": digest}
    environment = value.get("environment")
    if (
        not isinstance(environment, dict)
        or set(environment) != {"home", "cargoTargetDir"}
        or not isinstance(environment.get("home"), str)
        or not Path(environment["home"]).is_absolute()
        or not isinstance(environment.get("cargoTargetDir"), str)
        or not Path(environment["cargoTargetDir"]).is_absolute()
    ):
        raise ValueError(f"qualification profile {name!r} has a malformed environment")
    return value, validated


def load_lock(
    path: Path = LOCK, profile_name: str | None = None
) -> tuple[str, dict[str, Any], dict[str, dict[str, str]]]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict) or value.get("schemaVersion") != SCHEMA:
        raise ValueError("tools.lock has an unknown schema")
    if set(value) != {"schemaVersion", "defaultProfile", "profiles"}:
        raise ValueError("tools.lock has malformed top-level fields")
    profiles = value.get("profiles")
    default = value.get("defaultProfile")
    if not isinstance(profiles, dict) or not profiles:
        raise ValueError("tools.lock has no qualification profiles")
    if not isinstance(default, str) or default not in profiles:
        raise ValueError("tools.lock default profile is unavailable")
    selected = profile_name or default
    if selected not in profiles:
        raise ValueError(f"qualification profile is unavailable: {selected!r}")
    profile, tools = validate_profile(selected, profiles[selected])
    return selected, profile, tools


def trusted_path(tools: dict[str, dict[str, str]]) -> str:
    parents: list[str] = []
    for name in REQUIRED:
        parent = str(Path(tools[name]["path"]).parent)
        if parent not in parents:
            parents.append(parent)
    return ":".join(parents)


def qualified_environment(
    profile_name: str, profile: dict[str, Any], tools: dict[str, dict[str, str]]
) -> dict[str, str]:
    return {
        "HOME": profile["environment"]["home"],
        "PATH": trusted_path(tools),
        "CARGO_TARGET_DIR": profile["environment"]["cargoTargetDir"],
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TL_PARSE_TOOL_PROFILE": profile_name,
    }


def verify_live(
    profile_name: str, profile: dict[str, Any], tools: dict[str, dict[str, str]]
) -> tuple[list[str], list[str]]:
    unavailable: list[str] = []
    mismatches: list[str] = []
    environment = qualified_environment(profile_name, profile, tools)
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
        if Path(observed) != locked_path:
            mismatches.append(
                f"qualified path mismatch for {name}: profile declares {locked_path}, resolved {observed}"
            )
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
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile")
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--verify-live", action="store_true")
    action.add_argument("--trusted-path", action="store_true")
    action.add_argument("--home", action="store_true")
    action.add_argument("--cargo-target-dir", action="store_true")
    action.add_argument("--profile-name", action="store_true")
    action.add_argument("--tool-path", choices=REQUIRED)
    action.add_argument("--tool-sha256", choices=REQUIRED)
    args = parser.parse_args()
    try:
        selected, profile, tools = load_lock(profile_name=args.profile)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"qualified tool lock is unavailable: {error}", file=sys.stderr)
        return 2
    if args.trusted_path:
        print(trusted_path(tools))
        return 0
    if args.home:
        print(profile["environment"]["home"])
        return 0
    if args.cargo_target_dir:
        print(profile["environment"]["cargoTargetDir"])
        return 0
    if args.profile_name:
        print(selected)
        return 0
    if args.tool_path:
        print(tools[args.tool_path]["path"])
        return 0
    if args.tool_sha256:
        print(tools[args.tool_sha256]["sha256"])
        return 0
    unavailable, mismatches = verify_live(selected, profile, tools)
    for error in unavailable + mismatches:
        print(error, file=sys.stderr)
    if unavailable:
        return 2
    return 1 if mismatches else 0


if __name__ == "__main__":
    raise SystemExit(main())
