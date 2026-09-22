---
id: SR-063
title: "Rust and code review — structured fuzz campaign evidence"
type: SpecReview
analysis: code-review
scope: "tl-parse#25 implementation through 8ddfbc3"
review_set: subset
---

# Rust and code review — structured fuzz campaign evidence

## Summary

The tl-parse#25 implementation was reviewed under both
`agent-skills/code-review` and its `agent-skills/rust-review` dispatch. The
review covered the Rust campaign producer, process supervision, seed and
artifact boundaries, structured Quoin intake, mutation controls, dependency
policy, requirement traces, and the exact repository gates. Every actionable
finding was fixed before this review was closed.

The resulting boundary is fail closed: only two target identities are
accepted; seed and artifact populations, bytes, names and digests are bounded;
tool probes and fuzz runs are time-bounded process groups; every non-pass
domain outcome exits non-zero; and the consumer refuses any campaign whose
identity, bounds, manifest, tools, sanitizer, process, or outcome fields do not
form the declared closed contract.

## Assurance Context

- **Profile:** `AP-001` (`spec/assurance/AP-001.md`), version 0.2, status
  `active`; its review policy requires specification review, code review and
  gap analysis.
- **Baseline:** implementation through `8ddfbc3` on PR #34.
- **Impact evaluated:** silent evidence reinterpretation and hostile-input or
  artifact growth across both fuzz targets and the Quoin intake boundary.
- **Decision boundary:** the campaigns are bounded population evidence, not a
  universal proof or release authority; lossless binary attachment remains
  deferred to `agent-ix/quoin#363`.
- **Active exceptions:** none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5401 | high | **Fixed:** the first intake implementation bound protocol, target, symbol and outcomes but could still attest a pass after mutations to the run/deadline bounds, manifest identity, tool identity, sanitizer, or process result. The Rust adapter now validates the complete closed campaign shape against repository-owned manifest bytes and explicit Rust mutation tests prove every field is load-bearing. | `examples/fuzz_campaign.rs`; `scripts/assurance_chain.py`; TC-047 |
| FND-5402 | medium | **Fixed:** external process cleanup and output bounds were incomplete during review. Both cargo-fuzz and version probes now run in process groups, timeouts kill and reap the group, version output is capped at 8 KiB, and the campaign has finite seed, artifact, byte and time bounds. | `examples/fuzz_campaign.rs:520`; `examples/fuzz_campaign.rs:560`; `examples/fuzz_campaign.rs:703`; TC-047 |
| FND-5403 | medium | **Fixed:** the ambient dependency audit warned about a wildcard crate constraint and unused license allowances, while the first coverage assertion/header followed a different ambient module contract. `tl-syntax` now has both exact version and revision constraints, unused license allowances are removed, and the matrix/assertion follow pinned Quire 0.31's 94/94 contract. | `Cargo.toml`; `deny.toml`; `spec/test-matrix.md`; `tests/shared_assurance.rs`; commits `ecb3580`, `cc543ea` |
| FND-5404 | high | **Fixed:** live ticket policy review found that the strict fuzz-result validator and its mutation generation had initially expanded Python, contrary to the Rust-only implementation directive. Validation, outcome mapping, and mutation assertions now live in `examples/fuzz_campaign.rs`; Python only invokes the built Rust adapter as part of the pre-existing Quoin orchestration, the old shell producer is deleted, and no JavaScript or TypeScript path was added. | tl-parse#25 required-language policy; `examples/fuzz_campaign.rs`; `scripts/assurance_chain.py`; `tests/shared_assurance.rs` |
| FND-5405 | low | No remaining Rust defect was found: production code contains no panic/unwrap/expect or unsafe block, fallible integer conversions protect wire sizes, test panics are test oracles, and the new dependency passes MSRV and cargo-deny. | `examples/fuzz_campaign.rs`; `Cargo.lock`; `deny.toml` |

## Verification

- `make ci` reached every Rust, corpus, fuzz, dependency and assurance gate at
  the pre-language-remediation baseline; the Rust adapter unit/integration,
  strict Clippy, formatting, Quire and compiled-test census gates pass after
  FND-5404 at `8ddfbc3`.
- Both `parser` and `clean_ascii_v2` completed 64 LeakSanitizer-enabled runs
  with no crash artifact and emitted separate passing
  `tl-parse.fuzz-campaign/v1` documents.
- `cargo test --all-targets --all-features`: 66 requirement-tagged tests pass,
  including eight TC-047 Rust producer/adapter tests and the shared intake
  integration tests.
- `quire coverage --scope . --strict`: 94/94 rows backed and Rust
  68/68/68 bound/tagged/candidates.
- Rust 1.75 MSRV, strict Clippy, rustfmt, rustdoc warnings-as-errors,
  cargo-deny, unsafe audit, checksum verification, the 40,000-case round-trip
  sweep, shared pin admission, and every Quoin scenario/control/probe pass.

## Verdict

**PASS** — all code-review and Rust-review findings are fixed. No code-level
blocker remains for tl-parse#25 or PR #34.
