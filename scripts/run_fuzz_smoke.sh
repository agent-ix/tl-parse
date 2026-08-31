#!/usr/bin/env bash
set -euo pipefail

scratch="$(mktemp -d)"
cleanup() {
  rm -rf -- "$scratch"
}
trap cleanup EXIT

generated_corpus="$scratch/generated"
seed_corpus="$scratch/seeds"
mkdir -p "$generated_corpus" "$seed_corpus"
while read -r _ filename; do
  cp -- "fuzz/corpus/parser/$filename" "$seed_corpus/$filename"
done < fuzz/corpus/parser/SHA256SUMS

cargo +nightly fuzz run parser "$generated_corpus" "$seed_corpus" -- -runs=64
