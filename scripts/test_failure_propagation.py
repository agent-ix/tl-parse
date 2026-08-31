#!/usr/bin/env python3
"""Mutation tests for the local-CI failure-propagation policy."""

from __future__ import annotations

import importlib.util
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "check_failure_propagation", ROOT / "scripts" / "check_failure_propagation.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def rejected(text: str) -> bool:
    with tempfile.TemporaryDirectory() as temporary:
        makefile = Path(temporary) / "Makefile"
        makefile.write_text(text, encoding="utf-8")
        return bool(MODULE.inspect(makefile))


def main() -> int:
    original = (ROOT / "Makefile").read_text(encoding="utf-8")
    assert MODULE.inspect(ROOT / "Makefile") == []
    assert rejected(original.replace("\t$(CARGO) clippy", "\t-$(CARGO) clippy", 1))
    assert rejected(original.replace("\t$(CARGO) test", "\t$(CARGO) test || true", 1))
    assert rejected(original.replace("ci: ", "ci: fabricated ", 1))
    print("failure-propagation policy behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
