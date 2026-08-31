#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 0 ]]; then
  final_evidence_dir="$1"
else
  evidence_revision="$(/usr/bin/git rev-parse --short=12 HEAD)"
  evidence_timestamp="$(/usr/bin/date -u +%Y%m%dT%H%M%SZ)"
  final_evidence_dir="evidence/tl-parse-v01-${evidence_revision}-${evidence_timestamp}"
fi
checksum_path="${final_evidence_dir}.sha256"
pgm01_schema_digest="0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256"
pgm01_validator_digest="1c2881d5f8800dab031f6afa26d5ad11f88a5ab42a942bc9fe0c2853b58df2f1"

if [[ -e "$final_evidence_dir" || -e "$checksum_path" ]]; then
  echo "refusing to overwrite retained evidence: $final_evidence_dir" >&2
  exit 2
fi
if [[ -n "$(/usr/bin/git status --porcelain --untracked-files=all)" ]]; then
  echo "refusing to collect evidence from a modified or untracked source tree" >&2
  exit 2
fi
if [[ -n "${PGM01_PYTHON:-}" ]]; then
  echo "PGM01_PYTHON overrides are not permitted" >&2
  exit 2
fi
if ! /usr/bin/python3 -c 'import jsonschema' >/dev/null 2>&1; then
  echo "jsonschema is required for evidence collection" >&2
  exit 2
fi
if [[ -n "${PGM01_SCHEMA:-}" ]] && \
   [[ "$(/usr/bin/sha256sum "$PGM01_SCHEMA" | /usr/bin/cut -d' ' -f1)" != "$pgm01_schema_digest" ]]; then
  echo "PGM-01 schema digest does not match the pinned envelope schema" >&2
  exit 2
fi
if [[ -n "${PGM01_VALIDATOR:-}" ]] && \
   [[ "$(/usr/bin/sha256sum "$PGM01_VALIDATOR" | /usr/bin/cut -d' ' -f1)" != "$pgm01_validator_digest" ]]; then
  echo "PGM-01 validator digest does not match the reviewed validator" >&2
  exit 2
fi

if ! /usr/bin/python3 scripts/tool_identity.py --verify-live; then
  echo "qualified tool identities do not match tools.lock" >&2
  exit 2
fi
trusted_path="$(/usr/bin/python3 scripts/tool_identity.py --trusted-path)"
real_home="$(/usr/bin/python3 scripts/tool_identity.py --home)"
staging_root="$(/usr/bin/mktemp -d -p . .tl-parse-evidence-stage.XXXXXX)"
cleanup_staging() {
  if [[ -n "${staging_root:-}" && -d "$staging_root" ]]; then
    /usr/bin/rm -rf -- "$staging_root"
  fi
}
trap cleanup_staging EXIT
evidence_dir="$staging_root/$(basename "$final_evidence_dir")"
/usr/bin/mkdir -p "$evidence_dir"
collection_failed=0
clean_env=(/usr/bin/env -i PATH="$trusted_path" HOME="$real_home" USER="${USER:-}" LANG="${LANG:-C}" PGM01_SCHEMA="${PGM01_SCHEMA:-}" PGM01_VALIDATOR="${PGM01_VALIDATOR:-}")

run_and_retain() {
  local name="$1"
  shift
  set +e
  "$@" >"$evidence_dir/$name.stdout" 2>"$evidence_dir/$name.stderr"
  local status=$?
  set -e
  local output_file
  for output_file in "$evidence_dir/$name.stdout" "$evidence_dir/$name.stderr"; do
    "${clean_env[@]}" python3 -c 'from pathlib import Path; import sys; p=Path(sys.argv[1]); d=p.read_bytes(); p.write_bytes(d.rstrip(b"\n") + b"\n" if d else d)' "$output_file"
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

"${clean_env[@]}" git rev-parse HEAD >"$evidence_dir/source-revision.txt"
echo clean >"$evidence_dir/source-state.txt"
"${clean_env[@]}" rustc --version --verbose >"$evidence_dir/rustc-version.txt"
"${clean_env[@]}" cargo --version --verbose >"$evidence_dir/cargo-version.txt"
"${clean_env[@]}" python3 --version >"$evidence_dir/python-version.txt"
"${clean_env[@]}" python3 -c 'import importlib.metadata; print(importlib.metadata.version("jsonschema"))' >"$evidence_dir/jsonschema-version.txt"
"${clean_env[@]}" quire provenance --pretty >"$evidence_dir/quire-provenance.json"
"${clean_env[@]}" cargo metadata --format-version 1 --all-features >"$evidence_dir/metadata.json"
for tool in bash cargo git make python3 quire rustc sha256sum; do
  "${clean_env[@]}" python3 scripts/tool_identity.py --tool-path "$tool" \
    >"$evidence_dir/tool-${tool}-path.txt"
  "${clean_env[@]}" python3 scripts/tool_identity.py --tool-sha256 "$tool" \
    >"$evidence_dir/tool-${tool}-sha256.txt"
done

# The candidate cannot already carry an AA-001 record for itself. Run every
# substantive CI prerequisite here; ordinary `make ci` adds verify-evidence
# after this record and its AA-001 binding are committed.
run_and_retain make-ci "${clean_env[@]}" make ci-for-evidence
run_and_retain make-spec "${clean_env[@]}" make spec
run_and_retain quire-coverage "${clean_env[@]}" python3 scripts/check_traceability_coverage.py
run_and_retain msrv "${clean_env[@]}" cargo +1.75.0 check --all-targets --all-features
run_and_retain rustdoc "${clean_env[@]}" env RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features
run_and_retain default-dependencies "${clean_env[@]}" cargo tree --no-default-features --edges normal
run_and_retain corpus-integrity "${clean_env[@]}" make check-corpus
# Retained tool logs are immutable evidence and may contain tool-authored
# trailing spaces. Their bytes are protected by the evidence manifests; this
# gate checks the authored source/specification diff.
run_and_retain diff-integrity "${clean_env[@]}" git diff --check \
  "origin/main...$(/usr/bin/git rev-parse HEAD)" -- . ':(exclude)evidence/**'

"${clean_env[@]}" python3 scripts/build_evidence_envelope.py "$evidence_dir" provisional
run_and_retain input-schema "${clean_env[@]}" python3 scripts/validate_json_schema.py schemas/tl-parse-evidence-input-v1.schema.json "$evidence_dir/collection-input.json"
run_and_retain manifest-schema "${clean_env[@]}" python3 scripts/validate_json_schema.py schemas/tl-parse-evidence-manifest-v1.schema.json "$evidence_dir/evidence-manifest.json"

if [[ -n "${PGM01_SCHEMA:-}" ]]; then
  run_and_retain pgm01-schema "${clean_env[@]}" python3 scripts/validate_json_schema.py "$PGM01_SCHEMA" "$evidence_dir/evidence-envelope.json"
else
  retain_skipped pgm01-schema
fi

if [[ -n "${PGM01_VALIDATOR:-}" ]]; then
  run_and_retain pgm01-validator "${clean_env[@]}" python3 "$PGM01_VALIDATOR" --fixture "$evidence_dir/evidence-envelope.json"
else
  retain_skipped pgm01-validator
fi

"${clean_env[@]}" python3 scripts/build_evidence_envelope.py "$evidence_dir" final

if [[ -n "${PGM01_SCHEMA:-}" ]]; then
  run_and_retain sealed-pgm01-schema "${clean_env[@]}" python3 scripts/validate_json_schema.py "$PGM01_SCHEMA" "$evidence_dir/evidence-envelope.json"
else
  retain_skipped sealed-pgm01-schema
fi

if [[ -n "${PGM01_VALIDATOR:-}" ]]; then
  run_and_retain sealed-pgm01-validator "${clean_env[@]}" python3 "$PGM01_VALIDATOR" --fixture "$evidence_dir/evidence-envelope.json"
else
  retain_skipped sealed-pgm01-validator
fi

if [[ "$(<"$evidence_dir/sealed-pgm01-schema.status.txt")" -ne 0 || \
      "$(<"$evidence_dir/sealed-pgm01-validator.status.txt")" -ne 0 ]]; then
  "${clean_env[@]}" python3 scripts/build_evidence_envelope.py "$evidence_dir" sealed-failed
fi

"${clean_env[@]}" python3 scripts/finalize_collection.py "$evidence_dir"

/usr/bin/mkdir -p "$(/usr/bin/dirname "$final_evidence_dir")"
/usr/bin/mv "$evidence_dir" "$final_evidence_dir"
/usr/bin/rmdir "$staging_root"
staging_root=""
evidence_dir="$final_evidence_dir"

/usr/bin/find "$evidence_dir" -type f -print0 \
  | /usr/bin/sort -z \
  | /usr/bin/xargs -0 /usr/bin/sha256sum >"$checksum_path"
if [[ $collection_failed -ne 0 ]]; then
  echo "one or more retained evidence commands failed" >&2
  exit 1
fi
