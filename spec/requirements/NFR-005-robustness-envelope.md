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
| NFR-005-AC-1 | For each of `HARD_MAX_SOURCE_BYTES`, `HARD_MAX_TOKENS`, `HARD_MAX_NODES`, `HARD_MAX_DEPTH`, `HARD_MAX_DIAGNOSTICS`, `HARD_MAX_PARSE_WORK`, `HARD_MAX_OUTPUT_BYTES`, and `HARD_MAX_FORMAT_WORK`, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit, except `HARD_MAX_DIAGNOSTICS`, where the extra diagnostic is dropped and `diagnostics_truncated` is set rather than a refusal being returned. A caller value above the constant is clamped to the constant. "Accepted" means not refused by that limit's own typed error; two limits are shadowed at the owner value and the test records the shadowing: a source of `HARD_MAX_SOURCE_BYTES` bytes always costs more than `HARD_MAX_PARSE_WORK` work, and `HARD_MAX_FORMAT_WORK` exceeds the work of any output that `HARD_MAX_OUTPUT_BYTES` admits, so its test asserts the clamp, that the work at `HARD_MAX_OUTPUT_BYTES` is below it, and the boundary at a caller-lowered `max_work`. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered limit for the same code path, for these limits: `HARD_MAX_SOURCE_BYTES`, `HARD_MAX_OUTPUT_BYTES`. The remaining limits here accept only a caller value at or below the constant (clamp down only), so the Miri run asserts the clamp and does not build the at-limit input. | unit-testing (TC-067) |
| NFR-005-AC-2 | For each strict-reader ceiling (`HARD_MAX_PARSE_ARTIFACT_BYTES`, `HARD_MAX_PARSE_ARTIFACT_DEPTH`, `HARD_MAX_PARSE_ARTIFACT_STRING_BYTES`, `HARD_MAX_PARSE_ARTIFACT_WORK`, and `ParseArtifactLimits::OWNER_MAXIMA`), a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. A caller value above a ceiling is clamped to it. At the owner value the work ceiling equals four times the byte ceiling, so the byte ceiling is always the first refusal (work: unobservable at owner values) and the work boundary is tested at a caller-lowered ceiling on both native and Miri. The native run uses the owner value and is never skipped. The Miri run uses a caller-lowered limit for the same code path, for these limits: `HARD_MAX_PARSE_ARTIFACT_BYTES`, `HARD_MAX_PARSE_ARTIFACT_WORK`. Depth and string ceilings are small and run at the owner value under Miri. | unit-testing (TC-068) |
| NFR-005-AC-3 | For the v4 canonical-text depth bound, which is `HARD_MAX_DEPTH`, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is `FormatErrorCode::InvalidGraph`, and its message names the depth cause (`v4 graph exceeds format depth`). | unit-testing (TC-069) |
| NFR-005-AC-4 | For the CLI bounded input read, a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The refusal is a `source_limit` diagnostic. | unit-testing (TC-070) |
| NFR-005-AC-5 | Every variant of `DiagnosticCode`, `FormatErrorCode`, `StrictParseArtifactReadError`, `ContextualParseError`, and `InfiniteDisposition` is either returned by at least one test through the public API or removed from the enum in the same change. `ContextualParseError::InvalidCatalog` and `ContextualParseError::Identity` were removed because no input can produce them; `ContextualParseError::MissingParserSpan` was removed because `parse` spans every node, so no value of it is producible; a span-less node reached through the strict wire reader is refused with a string error that the test asserts. `InfiniteDisposition::Failed` is produced by mutating the public fields of a report so that it holds neither a graph nor a refusal. | unit-testing (TC-071) |
| NFR-005-AC-6 | The unit and property suites complete under Miri on the pinned nightly named in the Makefile, with zero undefined-behavior reports, using a documented proptest case count and isolation setting, and the `miri` target is a prerequisite of `ci`. The Miri run disables isolation so the corpus and fuzz-seed suites read the checked-in files, and it covers every suite except `cli`, which is excluded because it spawns the CLI binary. The depth tests in `format.rs` build smaller graphs under `cfg!(miri)`. One test stays ignored under Miri, with its reason: `syntax_owner_graph_depth_ceiling_is_exact_and_formatting_remains_iterative` builds the 8,191-node owner depth-ceiling graph, which has no caller-lowered limit and is too slow to interpret. Every excluded or ignored test runs natively in `make ci`. The `miri_target_runs_in_the_ci_composite_on_the_pinned_nightly` test greps the Makefile text, so it proves wiring only; the Miri run itself is the evidence of cleanliness. | unit-testing (TC-072) |

## Dependencies

NFR-001 and NFR-004 supply the limits being proved. tl-syntax owns the graph
ceilings.
