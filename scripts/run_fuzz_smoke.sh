#!/usr/bin/env bash
set -euo pipefail

scratch="$(mktemp -d)"
preserve_scratch=0
cleanup() {
  if [[ $preserve_scratch -eq 0 ]]; then
    rm -rf -- "$scratch"
  else
    echo "fuzz failure inputs retained at $scratch" >&2
  fi
}
trap cleanup EXIT

generated_corpus="$scratch/generated"
seed_corpus="$scratch/seeds"
artifact_dir="$scratch/artifacts"
mkdir -p "$generated_corpus" "$seed_corpus" "$artifact_dir"
while read -r _ filename; do
  cp -- "fuzz/corpus/parser/$filename" "$seed_corpus/$filename"
done < fuzz/corpus/parser/SHA256SUMS

asan_options="${ASAN_OPTIONS:-}"
if [[ "${TL_PARSE_FUZZ_DISABLE_LEAKS:-0}" == "1" ]]; then
  asan_options="${asan_options:+$asan_options:}detect_leaks=0"
  echo "LeakSanitizer explicitly disabled by TL_PARSE_FUZZ_DISABLE_LEAKS=1" >&2
elif [[ -r /proc/self/status ]] &&
     grep -Eq '^NoNewPrivs:[[:space:]]+1$' /proc/self/status &&
     grep -Eq '^Seccomp:[[:space:]]+2$' /proc/self/status; then
  # LeakSanitizer cannot inspect processes in this restricted sandbox. Keep
  # AddressSanitizer enabled and make the skipped leak lane visible in logs.
  asan_options="${asan_options:+$asan_options:}detect_leaks=0"
  echo "LeakSanitizer unavailable under no-new-privileges/seccomp sandbox; disabled" >&2
else
  echo "LeakSanitizer enabled" >&2
fi

set +e
ASAN_OPTIONS="$asan_options" cargo +nightly fuzz run parser \
  "$generated_corpus" "$seed_corpus" -- \
  "-artifact_prefix=$artifact_dir/" -runs=64
status=$?
set -e
if [[ $status -ne 0 ]]; then
  preserve_scratch=1
fi
exit "$status"
