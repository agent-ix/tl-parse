#!/usr/bin/env python3
"""Re-derive clean-room attribution digests from the pinned Cargo source."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
ROW = re.compile(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|$", re.MULTILINE)


def main() -> int:
    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--all-features"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    package = next(item for item in metadata["packages"] if item["name"] == "tl-syntax")
    source = package.get("source", "")
    if not source.endswith("#740182f13b84858008d6f176f75136737d405c1b"):
        print(f"attribution source revision drifted: {source}", file=sys.stderr)
        return 1
    source_root = Path(package["manifest_path"]).parent
    document = (ROOT / "docs" / "ATTRIBUTION.md").read_text(encoding="utf-8")
    declared = dict(ROW.findall(document))
    expected = {"src/syntax.rs", "src/document.rs", "LICENSE-MIT", "LICENSE-APACHE"}
    if set(declared) != expected:
        print("attribution file census drifted", file=sys.stderr)
        return 1
    for relative, digest in declared.items():
        observed = hashlib.sha256((source_root / relative).read_bytes()).hexdigest()
        if observed != digest:
            print(f"attribution digest mismatch: {relative}", file=sys.stderr)
            return 1
    print("clean-room attribution digests match the pinned Cargo source")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
