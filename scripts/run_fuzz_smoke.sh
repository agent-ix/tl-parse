#!/usr/bin/env bash
set -euo pipefail

scratch="$(mktemp -d)"
cleanup() {
  rm -rf -- "$scratch"
}
trap cleanup EXIT

generated_corpus="$scratch/generated"
seed_corpus="$scratch/seeds"
artifact_dir="$scratch/artifacts"
mkdir -p "$generated_corpus" "$seed_corpus" "$artifact_dir"
while read -r _ filename; do
  cp -- "fuzz/corpus/parser/$filename" "$seed_corpus/$filename"
done < fuzz/corpus/parser/SHA256SUMS

# LeakSanitizer cannot run under ptrace-based sandboxes and aborts after a
# successful fuzz run. Leak detection is outside this bounded parser smoke;
# AddressSanitizer's memory-safety checks remain enabled.
ASAN_OPTIONS="${ASAN_OPTIONS:+$ASAN_OPTIONS:}detect_leaks=0" \
  cargo +nightly fuzz run parser "$generated_corpus" "$seed_corpus" -- \
  "-artifact_prefix=$artifact_dir/" -runs=64
