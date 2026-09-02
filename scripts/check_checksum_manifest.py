#!/usr/bin/env python3
"""Verify a SHA256SUMS file without changing shell working directories."""

from __future__ import annotations

import hashlib
import re
import sys
from pathlib import Path


LINE = re.compile(r"^([0-9a-f]{64})  ([^/][^\n]*)$")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: check_checksum_manifest.py DIRECTORY", file=sys.stderr)
        return 2
    directory = Path(sys.argv[1])
    manifest = directory / "SHA256SUMS"
    try:
        lines = manifest.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        print(error, file=sys.stderr)
        return 1
    failed = False
    seen: set[str] = set()
    for number, line in enumerate(lines, start=1):
        match = LINE.fullmatch(line)
        if match is None or match.group(2) in seen or ".." in Path(match.group(2)).parts:
            print(f"{manifest}:{number}: malformed or unsafe entry", file=sys.stderr)
            failed = True
            continue
        seen.add(match.group(2))
        path = directory / match.group(2)
        try:
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            print(f"{path}: FAILED ({error})", file=sys.stderr)
            failed = True
            continue
        if actual != match.group(1):
            print(f"{path}: FAILED", file=sys.stderr)
            failed = True
        else:
            print(f"{path}: OK")
    return 1 if failed or not seen else 0


if __name__ == "__main__":
    raise SystemExit(main())
