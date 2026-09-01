#!/usr/bin/env python3
"""Run Cargo through the source-locked rustup proxy and selected toolchain."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

import tool_identity


def main() -> int:
    if len(sys.argv) < 3 or sys.argv[1] not in {"1.75.0", "nightly"}:
        print(
            "usage: run_cargo_toolchain.py {1.75.0|nightly} CARGO_ARGS...",
            file=sys.stderr,
        )
        return 2
    try:
        _, _, tools = tool_identity.load_lock(
            profile_name=os.environ.get("TL_PARSE_TOOL_PROFILE")
        )
    except (OSError, ValueError) as error:
        print(f"cannot select source-locked rustup: {error}", file=sys.stderr)
        return 2
    rustup = Path(tools["rustup"]["path"])
    environment = dict(os.environ)
    environment["PATH"] = f"{rustup.parent}:{environment.get('PATH', '')}"
    result = subprocess.run(
        [str(rustup), "run", sys.argv[1], "cargo", *sys.argv[2:]],
        check=False,
        env=environment,
    )
    return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
