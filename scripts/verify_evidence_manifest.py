#!/usr/bin/env python3
"""Verify retained manifest artifact identities against retained files."""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path


CHECKSUM_LINE = re.compile(r"^([0-9a-f]{64})  (.+)$")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(evidence_dir: Path) -> list[str]:
    manifest = json.loads(
        (evidence_dir / "evidence-manifest.json").read_text(encoding="utf-8")
    )
    errors: list[str] = []
    checksum_path = evidence_dir.with_suffix(".sha256")
    expected_files: set[Path] = set()
    try:
        lines = checksum_path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        return [f"cannot read exact-membership manifest: {error}"]
    for number, line in enumerate(lines, start=1):
        match = CHECKSUM_LINE.fullmatch(line)
        if match is None:
            errors.append(f"malformed checksum line {number}: {line!r}")
            continue
        path = Path(match.group(2))
        try:
            expected_files.add(path.relative_to(evidence_dir))
        except ValueError:
            errors.append(f"checksum path escapes evidence directory: {path}")
    actual_files = {
        path.relative_to(evidence_dir)
        for path in evidence_dir.rglob("*")
        if path.is_file() and not path.is_symlink()
    }
    for relative in sorted(actual_files - expected_files):
        errors.append(f"unlisted retained artifact: {relative}")
    for relative in sorted(expected_files - actual_files):
        errors.append(f"missing retained artifact: {relative}")
    for artifact in manifest["artifacts"]:
        relative = Path(artifact["path"])
        if relative.is_absolute() or ".." in relative.parts:
            errors.append(f"unsafe artifact path: {relative}")
            continue
        path = evidence_dir / relative
        if not path.is_file() or path.is_symlink():
            errors.append(f"missing or non-regular artifact: {relative}")
            continue
        if path.stat().st_size != artifact["size"]:
            errors.append(f"artifact size mismatch: {relative}")
        if sha256(path) != artifact["sha256"]:
            errors.append(f"artifact digest mismatch: {relative}")
    return errors


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: verify_evidence_manifest.py EVIDENCE_DIR", file=sys.stderr)
        return 2
    errors = verify(Path(sys.argv[1]))
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
