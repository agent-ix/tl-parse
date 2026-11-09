---
id: NFR-005
title: Prove declared limits, error variants, and memory safety under test
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-parse/NFR-001
    type: constrains
  - target: ix://agent-ix/tl-parse/NFR-004
    type: constrains
---

# NFR-005: Prove declared limits, error variants, and memory safety under test

## Statement

tl-parse shall accept input exactly at every declared resource limit, refuse it
one unit above with that limit's own typed code, produce every variant of every
public error, code, and disposition enum from at least one test through the
public API, and run its unit and property suites without undefined behavior
under Miri.

## Scope

Applies to the process-safe constants `HARD_MAX_*`, `ParseArtifactLimits::OWNER_MAXIMA`,
the caller-selectable `ParseLimits`, `FormatLimits`, and `ParseArtifactLimits`
fields, the fixed v4 canonical-text depth, and the bounded CLI input read. The
public enums are `DiagnosticCode`, `FormatErrorCode`,
`StrictParseArtifactReadError`, `ContextualParseError`, and
`InfiniteDisposition`. tl-parse is a `std` crate and makes no `no_std` claim;
the embedded build obligation belongs to tl-syntax.

## Rationale

Existing tests exercise limits only at small caller-chosen values, so a wrong
hard constant, a missing clamp, or a limit that no longer produces its own code
passes unnoticed. A variant that no test can produce is either dead or
unverified. The crate forbids `unsafe` but depends on `serde_json` and
`tl-syntax`, so Miri is a cheap check that the whole tested surface stays free
of undefined behavior.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Declared limits lacking an at-limit accept test or a limit-plus-one typed refusal test | 0 | 0 | Test |
| Public error, code, and disposition variants not produced by any test | 0 | 0 | Test |
| Undefined-behavior reports from the unit and property suites under Miri | 0 | 0 | Test |

## Verification

One test per declared limit builds an input at the limit and at limit plus one,
asserting acceptance and the specific typed refusal. One test per enum drives
the public API to each variant. The Miri target runs the unit and property
suites with reduced proptest case counts.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-005-AC-1 | Each `HARD_MAX_SOURCE_BYTES`, `HARD_MAX_TOKENS`, `HARD_MAX_NODES`, `HARD_MAX_DEPTH`, `HARD_MAX_DIAGNOSTICS`, `HARD_MAX_PARSE_WORK`, `HARD_MAX_OUTPUT_BYTES`, and `HARD_MAX_FORMAT_WORK` value is accepted at its declared size and refused at one above with the matching `DiagnosticCode` or `FormatErrorCode`, and a caller limit above the constant is clamped to it. | unit-testing |
| NFR-005-AC-2 | Each strict-reader ceiling (`HARD_MAX_PARSE_ARTIFACT_BYTES`, `_DEPTH`, `_STRING_BYTES`, `_WORK`, and `OWNER_MAXIMA`) admits a document at the ceiling and refuses one above it with the matching `StrictParseArtifactReadError` variant, and no caller value raises a ceiling. | unit-testing |
| NFR-005-AC-3 | The v4 canonical-text depth bound and the CLI bounded read admit input at the bound and refuse input one above with `FormatErrorCode::InvalidGraph` and a `source_limit` diagnostic respectively. | unit-testing |
| NFR-005-AC-4 | Every variant of `DiagnosticCode`, `FormatErrorCode`, `StrictParseArtifactReadError`, `ContextualParseError`, and `InfiniteDisposition` is returned by at least one test through the public API. | unit-testing |
| NFR-005-AC-5 | The unit and property suites complete under Miri with zero undefined-behavior reports, using a documented proptest case count and isolation setting. | unit-testing |
| NFR-005-AC-6 | tl-parse documentation and manifest make no `no_std` claim and state that the crate requires `std`. | unit-testing |

## Dependencies

NFR-001 and NFR-004 supply the limits being proved. tl-syntax owns the graph
ceilings and the `no_std` build.
