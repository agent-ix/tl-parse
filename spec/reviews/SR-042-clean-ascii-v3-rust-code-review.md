---
id: SR-042
title: "Rust/code review — clean-ascii/v3 past parsing"
type: SpecReview
analysis: code-review
scope: "src/lexer.rs, src/parser.rs, src/format.rs, src/past.rs, src/lib.rs, tests/clean_ascii_v3.rs, tests/parser.rs, tests/cli.rs, docs/DIALECT-003-clean-ascii-v3.md, docs/ATTRIBUTION.md"
review_set: subset
---

# Rust/code review — clean-ascii/v3 past parsing

## Summary

The Task-002 candidate exposes a separate, fixed-profile v3 entry point rather
than widening v1 or v2. It constructs formula-v2 O/H/Y/S/T nodes directly,
retains parser resource ceilings and iterative formatting, rejects the future
operator family, validates its versioned report on decode, and introduces no
unsafe, blocking, or unbounded production path. Every authorial review finding
was corrected before this review was closed.

## Verdict

**PASS** — no open Rust or code-review finding remains in Task-002 scope.

## Assurance Context

`AP-001` applies to the parser and malformed-input boundaries. The review used
accepted tl-syntax MRS-003, FR-013-AC-2, TC-055, and the parser half of TC-057
as its requirement boundary. No hosted run, Quoin record, or qualification
claim is made. The shared-assurance suite correctly refuses the ambient host
because its Quire and ix-flow versions differ from the repository pins; that
external mismatch did not weaken the implementation gates.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4201 | medium | The first lexer version recognized the leading `Y` in a long identifier such as `Yesterday`, producing a partial-token diagnostic instead of the exact identifier span. Fixed with an operand-aware strong-Previous lexical boundary and an exact regression case. | `src/lexer.rs`, `tests/clean_ascii_v3.rs`, TC-055 |
| FND-4202 | medium | The first strict report decoder admitted a fabricated report with neither a document nor any retained/truncated diagnostic and did not reject out-of-range limits, statistics, or source spans. Fixed with report-state, effective-limit, graph, and source-bound validation plus wire mutations. | `src/past.rs`, `tests/clean_ascii_v3.rs`, TC-055, TC-057 |
| FND-4203 | medium | The initial v3 document digest test asserted only a 64-character shape, so arbitrary normative-document drift could stay green. Fixed by pinning the exact document and compact-record SHA-256 values. | `src/lib.rs`, `tests/clean_ascii_v3.rs`, `docs/DIALECT-003-clean-ascii-v3.md` |
| FND-4204 | low | The tl-syntax dependency moved but the v1 provenance document, deny-source annotation, and compiled-pin assertions initially still named the previous pin. Fixed while preserving the distinct historical authorship basis. | `docs/ATTRIBUTION.md`, `docs/DIALECT-001-clean-room-mltl-v1.md`, `deny.toml`, `tests/cli.rs` |
| FND-4205 | low | The complete shared-assurance lane cannot run on this host: the repository expects Quire 0.31.0 and ix-flow 0.0.4, while the host exposes Quire 0.32.0 and ix-flow 0.2.3, and the per-worktree assurance environment is absent. The tests fail closed; no result was reclassified as passing. | `assurance/pins.json`, `tests/shared_assurance.rs` |

## Gate Results

- Rustfmt and strict Clippy over all targets/features: pass.
- All ordinary library, binary, v1, v2, v3, CLI, contextual, corpus, formatter,
  parser, and property tests: pass; 52 tests in the selected non-assurance lane.
- Rust 1.75 all-target/all-feature check: pass.
- Rustdoc with warnings denied, release build, unsafe audit, and checked corpus
  manifests: pass.
- Cargo Deny advisories, bans, licenses, and sources: pass; the existing
  wildcard-git and unmatched-license allowances remain warnings only.
- Compiled Rust trace census: pass, 66 requirement-tagged tests.
