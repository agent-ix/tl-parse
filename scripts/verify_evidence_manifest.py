#!/usr/bin/env python3
"""Verify retained manifest artifact identities against retained files."""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(evidence_dir: Path) -> list[str]:
    manifest = json.loads(
        (evidence_dir / "evidence-manifest.json").read_text(encoding="utf-8")
    )
    errors = []
    for artifact in manifest["artifacts"]:
        relative = Path(artifact["path"])
        if relative.is_absolute() or ".." in relative.parts:
            errors.append(f"unsafe artifact path: {relative}")
            continue
        path = evidence_dir / relative
        if not path.is_file():
            errors.append(f"missing artifact: {relative}")
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
