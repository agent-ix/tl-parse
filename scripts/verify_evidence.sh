#!/usr/bin/env bash
set -euo pipefail

found=0
if [[ ! -f evidence/ANCHORS ]]; then
  echo "retained evidence anchor manifest is missing" >&2
  exit 1
fi
sha256sum --check evidence/ANCHORS

while IFS= read -r -d '' entry; do
  if [[ -L "$entry" ]]; then
    echo "evidence root contains a symlink: $entry" >&2
    exit 1
  fi
  case "$entry" in
    evidence/ANCHORS|evidence/README.md|evidence/REQUIRED|evidence/*.sha256) ;;
    *)
      echo "unrecognized evidence-root file: $entry" >&2
      exit 1
      ;;
  esac
done < <(find evidence -mindepth 1 -maxdepth 1 \( -type f -o -type l \) -print0 | sort -z)

while IFS= read -r -d '' record; do
  checksum="${record}.sha256"
  if [[ ! -f "$checksum" ]]; then
    if python3 scripts/collection_marker.py check "$record/.collecting"; then
      continue
    fi
    echo "retained evidence directory lacks a checksum manifest: $record" >&2
    exit 1
  fi
done < <(find evidence -mindepth 1 -maxdepth 1 -type d -print0 | sort -z)

assured_record="$(sed -n 's/^- Record: `\(evidence\/[^`]*\)`.*/\1/p' spec/assurance/AA-001.md)"
if [[ -z "$assured_record" || ! -f "${assured_record}.sha256" ]]; then
  echo "assurance argument does not name one retained evidence record" >&2
  exit 1
fi
assured_digest="$(sha256sum "${assured_record}.sha256" | cut -d' ' -f1)"
if ! grep -Fq "${assured_digest}" spec/assurance/AA-001.md; then
  echo "assurance argument does not bind its record's outer manifest digest" >&2
  exit 1
fi
if ! grep -Fqx "${assured_digest}  ${assured_record}.sha256" evidence/ANCHORS; then
  echo "assurance argument and evidence anchors disagree" >&2
  exit 1
fi
while IFS= read -r -d '' checksum; do
  found=1
  if ! grep -Fqx "$(sha256sum "$checksum")" evidence/ANCHORS; then
    echo "retained evidence manifest lacks a committed anchor: $checksum" >&2
    exit 1
  fi
  sha256sum --check "$checksum"
  evidence_dir="${checksum%.sha256}"
  python3 scripts/verify_evidence_manifest.py "$evidence_dir"
  if [[ ! -f "$evidence_dir/collection-summary.json" ]]; then
    echo "retained evidence summary is missing: $evidence_dir" >&2
    exit 1
  fi
  python3 scripts/finalize_collection.py --check "$evidence_dir"
done < <(find evidence -maxdepth 1 -type f -name '*.sha256' -print0 | sort -z)

if [[ $found -eq 0 ]]; then
  if [[ -e evidence/REQUIRED ]]; then
    echo "retained evidence is required but no checksum manifest was found" >&2
    exit 1
  fi
  echo "no retained evidence yet; exact-candidate collection is pending"
fi
