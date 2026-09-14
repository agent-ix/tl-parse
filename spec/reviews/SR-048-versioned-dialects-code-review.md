---
id: SR-048
title: "Code review — versioned dialect architecture"
type: SpecReview
analysis: code-review
scope: "FR-009 implementation, compatibility exports, owner pin, TC-046"
review_set: subset
---

# Code review — versioned dialect architecture

## Summary

Reviewed the implementation for policy ownership, code/test alignment, real
owner-boundary use, compatibility, strict wire admission, resource refusal,
and prohibited semantic duplication. Tests exercise public entry points and
the real tl-syntax reader; no mock owner, mirror AST, evaluator, rewrite rule,
or tautological private-only assertion was introduced.

## Verdict

**PASS.** No open code-review finding remains in the FR-009 allocation.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4801 | high | The crate stored v2/v3 policy inside flat `derived`, `past`, lexer, parser, and formatter matches. Resolved by moving real ownership to `dialect::{v1,v2,v3}` and preserving public paths as compatibility re-exports. | `src/dialect/`; `src/lexer.rs`; `src/parser.rs`; `src/formatter.rs` |
| FND-4802 | high | Successful construction never crossed the strict byte-reader boundary and v3 reports lacked a bounded canonical reader. Resolved with exact owner re-admission, caller-lowered immutable report limits, and `PastParseReport::from_json_bytes`. | FR-009-AC-4; `src/parser.rs`; `src/dialect/v3.rs`; TC-046 |
| FND-4803 | medium | The prior 5,000-depth test is impossible under the owner’s 4,096 ceiling and initially surfaced only generic validation failure. Resolved by mapping owner limits to typed codes and testing exact 4,096/4,097 behavior. | FR-009-AC-3; FR-009-AC-5; `tests/format.rs` |
| FND-4804 | medium | The first formatter refactor narrowed valid formula-v2 future/Boolean compatibility. Resolved by selecting future/Boolean output policy by semantic profile while retaining the explicit v3 schema/profile gate. | `src/formatter.rs`; TC-046 |
| FND-4805 | medium | The pin initially left the fuzz lock, provenance text, deny annotation, digest controls, and FR-002 on prior identities. Resolved across all current enforcing records while retaining historical identities as historical facts. | Cargo locks; `deny.toml`; `docs/ATTRIBUTION.md`; FR-002; TC-020; TC-031 |

## Gate Results

- Rustfmt and strict Clippy over all targets/features: pass.
- All selected non-qualification library, binary, dialect, CLI, contextual,
  corpus, formatter, parser, property, and history-replay tests: pass.
- Corpus conformance and 40,000-case round-trip sweep: pass with zero drift.
- TC-046 exercises cross-dialect refusal, exact v3 graph/text behavior, owner
  mutations, canonical report refusal, exact limits, and hostile UTF-8.
