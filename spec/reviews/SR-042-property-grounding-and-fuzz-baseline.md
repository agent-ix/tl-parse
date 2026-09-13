---
id: SR-042
title: "spec-correctness — tl-parse property grounding and fuzz baseline"
type: SpecReview
analysis: spec-correctness
scope: "spec/requirements, spec/test-matrix.md, tests, and fuzz"
review_set: all
---

## Summary

Quire 0.31.0 classifies all 49 binding criteria on `9ca856b`: 18 are
extractable, one is a review-gated candidate, and 30 are not extractable. The
existing Rust suite already binds all 49 criteria through 59 compiled test or
fuzz symbols, so this pass emits no duplicate test and instead records the
complete grounding disposition. The existing generated properties for
parse-format-parse and v2 direct-lowering equivalence pass.

A bounded `make fuzz-smoke` run also passes for both checked-in cargo-fuzz
targets with LeakSanitizer enabled. This is seed-plus-smoke evidence, not an
exhaustive input proof, a coverage plateau, or a retained Quoin evidence record.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4201 | low | The sole candidate, StR-002-VC-1, is corroborated by the existing generated parse-format-parse fixed-point property. A second test would duplicate TC-016, so none is emitted. | StR-002-VC-1, FR-004-AC-2, `tests/property.rs:85` |
| FND-4202 | low | The 30 not-extractable criteria retain their existing finite witnesses, corpus/integration demonstrations, or source inspections. They are not recast as generated properties merely to change the classifier census. | FR-001 through FR-008, NFR-001 through NFR-003, StR-001 through StR-003 |
| FND-4203 | medium | Quoin 0.23.1 has no libFuzzer-output adapter, and the shared architecture forbids scraping arbitrary stdout or adding a local generic evidence envelope. This review records the exact bounded run, but durable normalized campaign retention remains parent-epic work. | tl-syntax#26, engineering-assurance#7, `quoin evidence record --help` |

## Classifier census

| Dimension | Counts |
| --- | --- |
| Criteria | 49, all with row ids |
| Extraction | 18 extractable; 1 candidate; 30 not-extractable |
| Property | 12 universal; 3 invariant; 2 idempotence; 2 round-trip; 29 example; 1 unclassified |
| Archetype | 34 FR; 9 NFR; 6 StR |
| Extracted spans | 8 domains; 0 preconditions; 8 oracles |
| Harness | Rust integration tests with proptest 1.5.0; cargo-fuzz 0.13.2 with libFuzzer |

## Extractable and candidate grounding ledger

| Criterion | Classifier | Grounding disposition |
| --- | --- | --- |
| FR-002-AC-2 | universal / extractable | Already covered. The domain is successful `parse` results over the two semantic profiles; the oracle is pinned-graph validation plus exact profile preservation in `tests/parser.rs:204`. |
| FR-002-AC-3 | invariant / extractable | Already covered. The negative domain is missing operands/delimiters and trailing input; TC-008 asserts retained recovery diagnostics and no partial document at `tests/parser.rs:219`. |
| FR-003-AC-2 | universal / extractable | Existing finite boundary witnesses. The declared source/token/node/depth/diagnostic/work limits are a closed set exercised by TC-011 and TC-012 beginning at `tests/parser.rs:243`; no parallel generator is emitted. |
| FR-004-AC-1 | idempotence / extractable | Existing finite operator rendering and formatting-idempotence witnesses in TC-014 and TC-015 at `tests/format.rs:20` and `tests/format.rs:43`. |
| FR-004-AC-2 | universal / extractable | Existing generated property. `source_strategy` supplies bounded valid formulas and both profiles; TC-016 reparses canonical output and compares profile, root and node kinds at `tests/property.rs:5` and `tests/property.rs:85`. |
| FR-005-AC-2 | round-trip / extractable | Criterion describes its fuzz technique. TC-019 binds the checked seed population at `tests/corpus.rs:94`, and `fuzz/fuzz_targets/parser.rs:8` performs diagnostic serialization plus successful canonical round trips. |
| FR-006-AC-2 | round-trip / extractable | Static/integration demonstration rather than an encode/decode law. TC-023 reads producer-owned structured bytes and proves Quoin/Quire do not execute producers at `tests/shared_assurance.rs:753`. |
| FR-006-AC-6 | universal / extractable | Existing closed corpus/integration witness. TC-027 checks the malformed count, exact state and retained bytes at `tests/shared_assurance.rs:1062`. |
| FR-006-AC-7 | universal / extractable | Static repository-boundary demonstration. TC-028's fail-closed census verifies that no local evidence framework is executable at `tests/shared_assurance.rs:1135`. |
| FR-007-AC-1 | universal / extractable | Already covered at the public API. TC-033 checks unique first-node ordering, parser spans, signals and exact caller documents at `tests/contextual.rs:49`. |
| FR-007-AC-2 | universal / extractable | Already covered over the public typed refusal. TC-034 removes one catalog binding and matches both proposition identity and parser span at `tests/contextual.rs:74`. |
| FR-007-AC-3 | universal / extractable | Already covered. TC-035 passes a complete shared requirement context and compares it verbatim while separately checking the parser span at `tests/contextual.rs:102`. |
| FR-008-AC-2 | universal / extractable | Existing generated property plus span witness. TC-040 generates W/M/U/R/Boolean expressions and compares direct lowering at `tests/clean_ascii_v2.rs:282`; TC-041 checks exact spans at line 310. |
| FR-008-AC-3 | universal / extractable | Existing finite negative-domain table. TC-042 covers each declared spelling, interval and resource refusal with exact code/span and no document at `tests/clean_ascii_v2.rs:465`. |
| FR-008-AC-5 | invariant / extractable | Criterion describes its fuzz technique. TC-044 binds the seed census and bounded outcomes at `tests/clean_ascii_v2.rs:553`; `fuzz/fuzz_targets/clean_ascii_v2.rs:11` owns arbitrary-input execution. |
| NFR-003-AC-1 | invariant / extractable | Existing shared-assurance integration check. TC-023 derives every result from producer bytes and refuses absent/empty/unreadable output at `tests/shared_assurance.rs:753`. |
| NFR-003-AC-4 | universal / extractable | Existing finite version-control population check. TC-029 covers normalized collisions and empty-population refusal beginning at `tests/shared_assurance.rs:588`. |
| StR-001-VC-2 | universal / extractable | Existing stakeholder witness through the same public parse/validation boundary as FR-002-AC-2 at `tests/parser.rs:204`. |
| StR-002-VC-1 | idempotence / candidate | Candidate confirmed, already covered by the generated fixed-point TC-016 at `tests/property.rs:85`; no duplicate is emitted. |

## Second-pass disposition

The 30 not-extractable records all have existing bound evidence. Twenty-five are
finite unit, property, corpus, CLI, wire, or shared-assurance witnesses:
FR-001-AC-1/2; FR-002-AC-1/4; FR-003-AC-1/3; FR-004-AC-3;
FR-005-AC-1/3; FR-006-AC-1/3/5; FR-007-AC-4/5; FR-008-AC-1/4/6;
NFR-001-AC-1/2; NFR-003-AC-2/3/5; StR-002-VC-2; and
StR-003-VC-1/2. Five are source/provenance inspections: FR-001-AC-3,
FR-007-AC-6, NFR-002-AC-1/2, and StR-001-VC-1. No row is ungrounded,
orphaned, or missing a public observable.

## Bounded fuzz execution record

The run used candidate `9ca856b4c040fc2c3329b6defd26a1c9b57de748`,
rustc `1.97.0-nightly (e22c616e4 2026-04-19)`, cargo-fuzz 0.13.2 and
`ASAN_OPTIONS=detect_leaks=1`. The repository script copied only checksum-listed
seeds into scratch directories and retained failure artifacts only on non-zero
exit.

| Target | Checked seeds | Runs | Final coverage/features | Peak RSS | Outcome |
| --- | ---: | ---: | --- | ---: | --- |
| `parser` | 4 | 64 | 980 / 2360 | 38 MiB | pass; no crash artifact |
| `clean_ascii_v2` | 5 | 64 | 1252 / 3204 | 42 MiB | pass; no crash artifact |

The feature counts are observations from this one smoke run. They are not stable
coverage goals and do not establish a plateau. The script's seccomp guard first
refused the sandboxed attempt with exit 125; the recorded run was repeated
outside seccomp so LeakSanitizer stayed enabled.

## Run report

| Route | Count |
| --- | ---: |
| Emitted | 0 |
| Emitted with finding | 0 |
| Not settled | 0 |
| Already covered | 49 |
| Existing generated properties | 3 test functions binding 4 criteria |

The four criteria bound by the three existing generated functions are
FR-004-AC-2 and StR-002-VC-1 (TC-016), FR-008-AC-2 (TC-040), and the
second-pass round-trip FR-008-AC-4 (TC-043). The remaining rows keep their
appropriate finite, fuzz, integration, or inspection evidence and are not
misreported as generated properties.
