---
id: FR-005
title: Retain hostile-input, fuzz, CLI, and conformance evidence
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-002
    type: implements
---

# FR-005: Retain hostile-input, fuzz, CLI, and conformance evidence

## Description

The repository shall retain versioned malformed and resource fixtures, a fuzz
target seeded by those fixtures, and thin CLI surfaces.

## Inputs

- One of the closed fuzz-target identities `parser` or `clean_ascii_v2`.
- That target's repository-owned `SHA256SUMS` seed manifest and listed regular
  files.
- The fixed smoke bounds of 64 executions and 300 seconds, the observed nightly
  Rust identity, the observed cargo-fuzz identity, and LeakSanitizer
  availability.

## Outputs

- Exactly one `tl-parse.fuzz-campaign/v1` JSON document for each recognized
  target.
- One normalized Quoin entry naming `fuzz:<target>`, plus the campaign's
  target, run bound, seed-manifest path/digest/count, tool identities, sanitizer
  state, process status, elapsed-time class, domain outcome, limitations, and
  zero or more crash artifact metadata records.

## Behavior

- Corpus files and manifest are checksum-protected and identify expected
  diagnostic codes or success results.
- The complete local gate compiles each checked-in fuzz target and executes a
  bounded libFuzzer smoke run over every checksum-protected seed. A
  repository-owned Rust producer emits one versioned structured campaign result
  per target, including the target identity, requested run bound, seed-manifest
  digest and count, sanitizer configuration, process outcome, and any produced
  crash-artifact identities. The target exercises parsing, diagnostic
  serialization, and successful canonical round trips under small fixed
  budgets.
- The Rust producer shall refuse an unknown target, malformed manifest,
  non-regular or escaping seed, digest mismatch, duplicate seed name, ambient
  sanitizer override, absent tool, unavailable LeakSanitizer, more than 256
  seeds, or a seed larger than 1 MiB without reporting a passing campaign.
- For a recognized target, the Rust producer shall write its structured result
  even when setup or execution is unavailable or the fuzz process fails.
- The Rust producer shall return a non-zero process status for every non-passing
  domain outcome.
- The producer shall report `pass` only when the fuzz process exits
  successfully and produces no crash artifact; a launched non-zero process is
  `fail`, an unavailable prerequisite is `unavailable`, and a successful
  process that nevertheless leaves an artifact is `suspect`.
- If the fuzz process exceeds 300 seconds, then the Rust producer shall
  terminate it and report `unavailable`.
- Each artifact metadata record shall contain its normalized file name, media
  type, exact byte length, and SHA-256 digest. The producer shall classify
  symlinked, non-regular, escaping, over-1-MiB, or over-16-count artifact
  populations as `suspect` without reading beyond the declared bound. Quoin
  retention of valid structured identities is in scope; lossless binary
  attachment is explicitly deferred to `agent-ix/quoin#363` and shall not be
  replaced by a local archive.
- `tl-parse validate` and `tl-parse format` accept a profile and file/stdin,
  use the library report, and have stable success, invalid-input, and usage
  exit classes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | The malformed/resource corpus is checksum-valid and every fixture produces its declared bounded outcome. | Test (TC-018) |
| FR-005-AC-2 | Each checked-in fuzz target compiles, a 64-execution/300-second libFuzzer smoke run consumes every digest-verified declared seed, and the Rust producer writes one `tl-parse.fuzz-campaign/v1` result that binds the target, bounds, seed manifest, observed tools, sanitizer state, process status, domain outcome, limitation, and bounded artifact identities; every unavailable, failed, or suspect result is non-zero, and successful seeds round-trip under declared limits. | Test (TC-019, TC-046) |
| FR-005-AC-3 | CLI validation/formatting outputs and exit classes match the library for valid, invalid, profile, stdin, source-limit, and usage cases; an oversized seekable file reports its metadata byte count, while a non-closing stream is read only through the first byte beyond the limit, without parsing fabricated text. | Test (TC-020, TC-021) |

## Dependencies

Depends on FR-003 and FR-004. FR-006 consumes and retains this requirement's
structured results. Lossless binary crash-artifact retention remains an external
dependency on `agent-ix/quoin#363`.
