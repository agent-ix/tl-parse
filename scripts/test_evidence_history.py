#!/usr/bin/env python3
"""Behavior tests for retained evidence identity and Git-history binding."""

from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SPEC = importlib.util.spec_from_file_location(
    "verify_evidence_history", ROOT / "scripts" / "verify_evidence_history.py"
)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def main() -> int:
    if sys.flags.optimize or os.environ.get("PYTHONOPTIMIZE"):
        print("optimized Python disables policy assertions", file=sys.stderr)
        return 2
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        evidence = root / "evidence"
        record_id = "tl-parse-v01-" + "a" * 12 + "-20260831T200000Z"
        record = evidence / record_id
        record.mkdir(parents=True)
        (record / "source-revision.txt").write_text("a" * 40 + "\n", encoding="utf-8")
        (record / "evidence-envelope.json").write_text(
            json.dumps({"recordId": record_id}) + "\n", encoding="utf-8"
        )
        manifest = evidence / f"{record_id}.sha256"
        manifest.write_text("introduced\n", encoding="utf-8")
        assert any("without Git metadata" in error for error in MODULE.verify(root))
        (record / "evidence-envelope.json").write_text(
            json.dumps({"recordId": record_id + "-clone"}) + "\n", encoding="utf-8"
        )
        assert MODULE.verify(root), "a cloned record with a false recordId was accepted"

    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        evidence = root / "evidence"
        evidence.mkdir()
        manifest = evidence / ("tl-parse-v01-" + "b" * 12 + "-20260831T200000Z.sha256")
        manifest.write_text("introduced\n", encoding="utf-8")
        subprocess.run(["/usr/bin/git", "init", "-q"], cwd=root, check=True)
        subprocess.run(["/usr/bin/git", "config", "user.name", "Policy Test"], cwd=root, check=True)
        subprocess.run(["/usr/bin/git", "config", "user.email", "policy@example.invalid"], cwd=root, check=True)
        subprocess.run(["/usr/bin/git", "add", "."], cwd=root, check=True)
        subprocess.run(["/usr/bin/git", "commit", "-qm", "introduce"], cwd=root, check=True)
        assert MODULE.verify(root), "missing record directory should fail identity verification"
        manifest.unlink()
        assert any("removed" in error for error in MODULE.verify(root)), (
            "a historically introduced record deletion was accepted"
        )
    print("retained evidence history behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
