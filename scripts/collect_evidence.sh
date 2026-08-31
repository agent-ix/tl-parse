#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 0 ]]; then
  evidence_dir="$1"
else
  evidence_revision="$(git rev-parse --short=12 HEAD)"
  evidence_timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
  evidence_dir="evidence/tl-parse-v01-${evidence_revision}-${evidence_timestamp}"
fi
checksum_path="${evidence_dir}.sha256"
pgm01_python="${PGM01_PYTHON:-python3}"

if [[ -e "$evidence_dir" || -e "$checksum_path" ]]; then
  echo "refusing to overwrite retained evidence: $evidence_dir" >&2
  exit 2
fi
if [[ -n "$(git status --porcelain --untracked-files=all)" ]]; then
  echo "refusing to collect evidence from a modified or untracked source tree" >&2
  exit 2
fi
if ! python3 -c 'import jsonschema' >/dev/null 2>&1; then
  echo "jsonschema is required for evidence collection" >&2
  exit 2
fi

mkdir -p "$evidence_dir"
collection_failed=0

run_and_retain() {
  local name="$1"
  shift
  set +e
  "$@" >"$evidence_dir/$name.stdout" 2>"$evidence_dir/$name.stderr"
  local status=$?
  set -e
  local output_file
  for output_file in "$evidence_dir/$name.stdout" "$evidence_dir/$name.stderr"; do
    python3 -c 'from pathlib import Path; import sys; p=Path(sys.argv[1]); d=p.read_bytes(); p.write_bytes(d.rstrip(b"\n") + b"\n" if d else d)' "$output_file"
  done
  echo "$status" >"$evidence_dir/$name.status.txt"
  if [[ $status -ne 0 ]]; then
    collection_failed=1
  fi
}

retain_skipped() {
  local name="$1"
  echo skipped-unavailable >"$evidence_dir/$name.stdout"
  : >"$evidence_dir/$name.stderr"
  echo 125 >"$evidence_dir/$name.status.txt"
  collection_failed=1
}

git rev-parse HEAD >"$evidence_dir/source-revision.txt"
echo clean >"$evidence_dir/source-state.txt"
rustc --version --verbose >"$evidence_dir/rustc-version.txt"
cargo --version --verbose >"$evidence_dir/cargo-version.txt"
python3 --version >"$evidence_dir/python-version.txt"
python3 -c 'import importlib.metadata; print(importlib.metadata.version("jsonschema"))' >"$evidence_dir/jsonschema-version.txt"
quire provenance --pretty >"$evidence_dir/quire-provenance.json"
cargo metadata --format-version 1 --all-features >"$evidence_dir/metadata.json"

run_and_retain make-ci make ci
run_and_retain make-spec make spec
run_and_retain quire-coverage python3 scripts/check_traceability_coverage.py
run_and_retain rustdoc env RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features
run_and_retain default-dependencies cargo tree --no-default-features --edges normal
run_and_retain corpus-integrity make check-corpus
run_and_retain diff-integrity git diff --check "origin/main...$(git rev-parse HEAD)"

python3 scripts/build_evidence_envelope.py "$evidence_dir" provisional
run_and_retain input-schema python3 scripts/validate_json_schema.py schemas/tl-parse-evidence-input-v1.schema.json "$evidence_dir/collection-input.json"
run_and_retain manifest-schema python3 scripts/validate_json_schema.py schemas/tl-parse-evidence-manifest-v1.schema.json "$evidence_dir/evidence-manifest.json"

if [[ -n "${PGM01_SCHEMA:-}" ]]; then
  run_and_retain pgm01-schema python3 scripts/validate_json_schema.py "$PGM01_SCHEMA" "$evidence_dir/evidence-envelope.json"
else
  retain_skipped pgm01-schema
fi

if [[ -n "${PGM01_VALIDATOR:-}" ]]; then
  run_and_retain pgm01-validator "$pgm01_python" "$PGM01_VALIDATOR" --fixture "$evidence_dir/evidence-envelope.json"
else
  retain_skipped pgm01-validator
fi

python3 scripts/build_evidence_envelope.py "$evidence_dir" final

if [[ -n "${PGM01_SCHEMA:-}" ]]; then
  run_and_retain sealed-pgm01-schema python3 scripts/validate_json_schema.py "$PGM01_SCHEMA" "$evidence_dir/evidence-envelope.json"
else
  retain_skipped sealed-pgm01-schema
fi

if [[ -n "${PGM01_VALIDATOR:-}" ]]; then
  run_and_retain sealed-pgm01-validator "$pgm01_python" "$PGM01_VALIDATOR" --fixture "$evidence_dir/evidence-envelope.json"
else
  retain_skipped sealed-pgm01-validator
fi

if [[ "$(<"$evidence_dir/sealed-pgm01-schema.status.txt")" -ne 0 || \
      "$(<"$evidence_dir/sealed-pgm01-validator.status.txt")" -ne 0 ]]; then
  python3 scripts/build_evidence_envelope.py "$evidence_dir" sealed-failed
fi

python3 scripts/finalize_collection.py "$evidence_dir"

find "$evidence_dir" -type f -print0 | sort -z | xargs -0 sha256sum >"$checksum_path"
if [[ $collection_failed -ne 0 ]]; then
  echo "one or more retained evidence commands failed" >&2
  exit 1
fi
