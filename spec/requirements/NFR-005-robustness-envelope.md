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
`InfiniteDisposition`. tl-parse is a `std` crate.

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
| NFR-005-AC-1 | For each of `HARD_MAX_SOURCE_BYTES`, `HARD_MAX_TOKENS`, `HARD_MAX_NODES`, `HARD_MAX_DEPTH`, `HARD_MAX_DIAGNOSTICS`, `HARD_MAX_PARSE_WORK`, `HARD_MAX_OUTPUT_BYTES`, and `HARD_MAX_FORMAT_WORK`, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. A caller value above the constant is clamped to the constant. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered limit for the same code path, for these limits: `HARD_MAX_SOURCE_BYTES`, `HARD_MAX_OUTPUT_BYTES`. The remaining limits here accept only a caller value at or below the constant (clamp down only), so the Miri run asserts the clamp and does not build the at-limit input. | unit-testing (TC-067) |
| NFR-005-AC-2 | For each strict-reader ceiling (`HARD_MAX_PARSE_ARTIFACT_BYTES`, `HARD_MAX_PARSE_ARTIFACT_DEPTH`, `HARD_MAX_PARSE_ARTIFACT_STRING_BYTES`, `HARD_MAX_PARSE_ARTIFACT_WORK`, and `ParseArtifactLimits::OWNER_MAXIMA`), a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. A caller value above a ceiling is clamped to it. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered limit for the same code path, for these limits: `HARD_MAX_PARSE_ARTIFACT_BYTES`, `HARD_MAX_PARSE_ARTIFACT_WORK`. Depth and string ceilings are small and run at the owner value under Miri. | unit-testing (TC-068) |
| NFR-005-AC-3 | For the v4 canonical-text depth bound, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is `FormatErrorCode::InvalidGraph`. | unit-testing (TC-069) |
| NFR-005-AC-4 | For the CLI bounded input read, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is a `source_limit` diagnostic. | unit-testing (TC-070) |
| NFR-005-AC-5 | Every variant of `DiagnosticCode`, `FormatErrorCode`, `StrictParseArtifactReadError`, `ContextualParseError`, and `InfiniteDisposition` is either returned by at least one test through the public API or removed from the enum in the same change. | unit-testing (TC-071) |
| NFR-005-AC-6 | The unit and property suites complete under Miri on the pinned nightly named in the Makefile, with zero undefined-behavior reports, using a documented proptest case count and isolation setting. | unit-testing (TC-072) |

## Dependencies

NFR-001 and NFR-004 supply the limits being proved. tl-syntax owns the graph
ceilings.
