---
id: SR-049
title: "Rust review — versioned dialect architecture"
type: SpecReview
analysis: code-review
scope: "FR-009 Rust implementation and TC-046"
review_set: subset
---

# Rust review — versioned dialect architecture

## Summary

Reviewed idiomatic Rust, repository conventions, public error surfaces,
untrusted-input bounds, panic/unsafe behavior, integer conversion boundaries,
allocation/work ceilings, graph traversal, and traced-test quality. The crate
remains synchronous and lock-free; formatting is iterative; parsing retains its
bounded recursion; all strict readers preflight before typed decoding.

## Verdict

**PASS.** No open Rust finding remains in the reviewed allocation.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4901 | high | Source offsets, node identities, file lengths, and formatter indices used unchecked `as` conversions. Resolved with checked `TryFrom` paths and fail-closed diagnostics or total saturation at narrower platform boundaries. | `src/parser.rs`; `src/lexer.rs`; `src/formatter.rs`; `src/bin/tl-parse.rs`; `src/dialect/v2.rs` |
| FND-4902 | high | Decoding a report before bounding bytes/depth/strings/work could retain attacker-selected input prematurely. Resolved with quote-aware preflight, immutable maxima, caller-lowered limits, canonical comparison, and a typed non-exhaustive error. | `src/diagnostic.rs`; `src/dialect/v3.rs`; TC-046 |
| FND-4903 | medium | Report validation did not require successful source/depth observations to fit effective limits or cross-check the embedded document through the strict owner reader. Both checks are now enforced. | `src/dialect/v3.rs`; TC-046 |
| FND-4904 | medium | Mutation coverage initially proved only profile refusal. Resolved with direct canonical-byte operator-shape and root-topology mutations at the tl-parse boundary. | `tests/versioned_dialects.rs`; FR-009-AC-4 |

## Safety and Tooling Results

- `#![forbid(unsafe_code)]`; unsafe audit passes; no async, blocking bridge,
  mutex, or file/network operation exists in the library boundary.
- Production source contains no unchecked integer cast, `unwrap`, `expect`,
  `todo`, or `unimplemented`. The legacy zero-span fallback contains one
  unreachable arm proven by `SourceSpan::new(0, 0)`'s owner invariant; no
  untrusted value can select it.
- Strict Clippy with warnings denied: pass.
- Rust 1.75 all-target check and selected implementation test suite: pass.
- Warning-free rustdoc, release build, cargo-deny, fuzz-target build, checksum
  manifests, and 73-test compiled trace census: pass.
