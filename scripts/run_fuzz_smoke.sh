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

if [[ -n "${ASAN_OPTIONS:-}" ]]; then
  echo "ambient ASAN_OPTIONS is not permitted" >&2
  exit 2
elif [[ -n "${TL_PARSE_FUZZ_DISABLE_LEAKS:-}" ]]; then
  echo "TL_PARSE_FUZZ_DISABLE_LEAKS is not permitted" >&2
  exit 2
elif [[ -r /proc/self/status ]] &&
     grep -Eq '^NoNewPrivs:[[:space:]]+1$' /proc/self/status &&
     grep -Eq '^Seccomp:[[:space:]]+2$' /proc/self/status; then
  # LeakSanitizer cannot inspect processes in this restricted sandbox. Keep
  # AddressSanitizer enabled and make the skipped leak lane visible in logs.
  echo "LeakSanitizer unavailable under no-new-privileges/seccomp sandbox; disabled" >&2
  exit 125
else
  echo "LeakSanitizer enabled" >&2
fi

set +e
ASAN_OPTIONS=detect_leaks=1 /usr/bin/python3 scripts/run_cargo_toolchain.py nightly \
  fuzz run parser \
  --target-dir "$CARGO_TARGET_DIR/fuzz" \
  "$generated_corpus" "$seed_corpus" -- \
  "-artifact_prefix=$artifact_dir/" -runs=64
status=$?
set -e
if [[ $status -ne 0 ]]; then
  preserve_scratch=1
fi
exit "$status"
