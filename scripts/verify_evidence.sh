#!/usr/bin/env bash
set -euo pipefail

found=0
if [[ -f evidence/ANCHORS ]]; then
  sha256sum --check evidence/ANCHORS
fi
while IFS= read -r -d '' checksum; do
  found=1
  if [[ -f evidence/ANCHORS ]] && ! grep -Fqx "$(sha256sum "$checksum")" evidence/ANCHORS; then
    echo "retained evidence manifest lacks a committed anchor: $checksum" >&2
    exit 1
  fi
  sha256sum --check "$checksum"
  evidence_dir="${checksum%.sha256}"
  python3 scripts/verify_evidence_manifest.py "$evidence_dir"
  if [[ -e "$evidence_dir/collection-summary.json" ]]; then
    python3 scripts/finalize_collection.py --check "$evidence_dir"
  fi
done < <(find evidence -maxdepth 1 -type f -name '*.sha256' -print0 | sort -z)

if [[ $found -eq 0 ]]; then
  if [[ -e evidence/REQUIRED ]]; then
    echo "retained evidence is required but no checksum manifest was found" >&2
    exit 1
  fi
  echo "no retained evidence yet; exact-candidate collection is pending"
fi
