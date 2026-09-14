---
id: SR-050
title: "Gap analysis — versioned dialect architecture"
type: SpecReview
analysis: gap-analysis
scope: "FR-009, TC-046, Task-009 tl-parse allocation"
review_set: subset
---

# Gap analysis — versioned dialect architecture

## Summary

Traced every FR-009 criterion to the implementation and public-boundary tests,
then inspected production behavior for code without an owning requirement. The
analysis covers the complete tl-parse allocation of Task-009, not a test-only
or qualification slice.

## Verdict

**VALIDATED.** FR-009 is 5/5 backed and no implementation or test gap remains
inside the tl-parse Task-009 allocation. The crate owns text dialect policy and
report admission only; evaluation, rewriting, signal semantics, Contract-IR,
monitoring, and qualification remain outside this allocation.

## Trace Map

| Obligation | Implementation | Evidence |
|---|---|---|
| FR-009-AC-1 | `dialect::{v1,v2,v3}`, lexer classification, parser policy dispatch | TC-046 cross-accept/refuse controls plus existing v1/v2/v3 cases |
| FR-009-AC-2 | v3 spelling/binding power/schema policy and shared formatter traversal | TC-046 plus exact O/H/Y/S/T graph/span/precedence tests |
| FR-009-AC-3 | crate-root compatibility re-exports, exact pin/provenance identities, typed owner-limit mapping | TC-046, TC-020, TC-031, exact 4,096/4,097 graph-depth boundary |
| FR-009-AC-4 | canonical parse-success re-admission and embedded-report owner validation | TC-046 profile/operator/topology mutations and real owner-reader assertions |
| FR-009-AC-5 | parse/format limits and bounded canonical report preflight | TC-046 exact/one-over limits, duplicate/unknown/trailing/noncanonical JSON, hostile UTF-8 |

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5001 | medium | TC-046 initially used representative v3 cases without tracing the already exhaustive O/H/Y/S/T and arbitrary-input properties. Resolved by adding TC-046 tags to those public tests instead of duplicating their oracle logic. | `tests/clean_ascii_v3.rs`; TC-046 |
| FND-5002 | medium | Exact-limit coverage omitted diagnostic retention and formatter one-over refusal. Resolved with exact/tight diagnostic, output-byte, and formatter-work controls. | `tests/versioned_dialects.rs`; FR-009-AC-5 |
| FND-5003 | low | The matrix described FR-009/TC-046 as planned after implementation. Both rows are now covered/implemented. | `spec/test-matrix.md` |

## Coverage Result

- FR-009: 5/5 criteria backed; no unbacked row or status lie in scope.
- Rust trace census: 73 compiled requirement-tagged tests.
- Quire reports 75/75 Rust candidates tagged/bound and 100% coverage for every
  FR-009 criterion. Repository-wide 104/108 matrix backing includes four older
  evidence-suite rows and cross-repository traces; no broader qualification
  completion is claimed or changed here.
