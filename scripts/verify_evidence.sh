#!/usr/bin/env bash
set -euo pipefail

found=0
while IFS= read -r -d '' checksum; do
  found=1
  sha256sum --check "$checksum"
done < <(find evidence -maxdepth 1 -type f -name '*.sha256' -print0 | sort -z)

if [[ $found -eq 0 ]]; then
  if [[ -e evidence/REQUIRED ]]; then
    echo "retained evidence is required but no checksum manifest was found" >&2
    exit 1
  fi
  echo "no retained evidence yet; exact-candidate collection is pending"
fi
