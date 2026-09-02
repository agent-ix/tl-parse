---
id: SR-009
title: Closing gap analysis — shared assurance migration
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, docs/**/*.md, assurance/**, tests/**/*.rs, examples/**/*.rs, scripts/**, Makefile
review_set: subset
---

# SR-009: Closing gap analysis — shared assurance migration

## Summary

Coverage at the final head is 68/72 rows (94%), unchanged in headline from
SR-007, but the composition changed: two tests were added for guards that were
claimed as verified and exercised by nothing, taking the Rust binding census
from 36/36/36 to 38/38/38 and the compiled requirement-tagged test count from 35
to 37.

The substantive gap-analysis change in this round is not a number. SR-007 stated
that five suites were backed by tests that invoke their commands. That was
false, and a gap analysis that mis-describes its own bindings is worse than one
that reports a lower figure.

## Verdict

**CONDITIONAL.** One high (FND-702, regraded from medium and deferred by
contract), one medium, two lows. No unbacked matrix row is undisclosed.

## Coverage

| Figure | Base `cf43f40` | SR-007 head | Final head |
| --- | --- | --- | --- |
| Rows backed | 62/62 (100%) | 68/72 (94%) | 68/72 (94%) |
| `spec/test-matrix.md` | 25/25 | 28/28 | 28/28 (100%) |
| `spec/evidence/suites.md` | 9/9 | 5/9 | 5/9 (55%) |
| Rust bound/tagged/candidates | 26/26/26 | 36/36/36 | 38/38/38 (100%) |
| Requirement documents | 11/11 | 12/12 | 12/12 (100%) |
| Compiled requirement-tagged tests | 28 | 35 | 37 |
| Documents grammar-clean | — | 44/44 | 45/45 (100%) |

## The binding correction

Five suite rows are backed. What backs them is each suite's **retained output**,
not an invocation of its command:

| Suite | Bound test | What the test does |
| --- | --- | --- |
| SUITE-003 | TC-024 | reads the `quire coverage --json` export and pins `totals` |
| SUITE-005 | TC-018 | runs `sha256sum --check` over `corpus/v1` |
| SUITE-006 | TC-023 | reads `msrv.jsonl` and asserts the attested result |
| SUITE-007 | TC-023 | runs the chain directly, not `make assurance` |
| SUITE-009 | TC-023 | reads both producers' retained results |

That is the architecture working — verdicts come from producer bytes — but it is
a weaker claim than "invokes the command", and both the registry and SR-007 now
say so. Four rows remain unbacked and are named.

## Traceability of the new requirement

FR-006's seven criteria each have an owning test, and two criteria gained a
second test in this round:

| Criterion | Tests | What would break it |
| --- | --- | --- |
| FR-006-AC-1 | TC-022 ×2 | a locally restated version rule; a mirror reference in a scanned file; a pin disagreement across the four files naming it |
| FR-006-AC-2 | TC-023 ×2 | an attested result not derived from producer bytes; a producer executed by Quoin or Quire |
| FR-006-AC-3 | TC-024 | an impact snapshot that is not the export; totals that moved; a status lie |
| FR-006-AC-4 | TC-025 | a moved evidence byte; a misattributed record; no accepted positive control |
| FR-006-AC-5 | TC-026 ×2 | a state undemonstrated by measurement; a negative without a control; a control naming a scenario that does not exist |
| FR-006-AC-6 | TC-027 | a malformed row reported as a pass; a count disagreeing with the corpus manifest |
| FR-006-AC-7 | TC-028 | a surviving machinery file; a changed frozen schema; a reference to one anywhere in the tree |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-901 | high | `.IGNORE:` neuters ten of the sixteen `ci` prerequisites and every remaining check stays green. Regraded from SR-007's medium by demonstration. Not closed: the migration contract removes this machinery. Every claim about it is corrected and `agent-ix/tl-parse#11` carries the reproduction | `Makefile` |
| FND-902 | medium | Four suite-registry rows are unbacked, and the five that are bound bind to output rather than invocation. Both facts are now stated in the registry itself | `spec/evidence/suites.md` |
| FND-903 | low | `[status-column-matches-nothing]`: the declaration expects `Status`, the matrix authors `Coverage Status`, so status classification is skipped and complete-but-unbacked rows cannot be detected. Program-wide; `agent-ix/quire-contract-ir#21` | `spec/test-matrix.md` |
| FND-904 | low | `[catch-all-universal]`: 2 of 7 documents binding extractable criteria name a specific property shape for none. Carried from the base revision, unchanged by this migration | `spec/requirements/FR-003-diagnostics-limits.md` |

## Reverse traceability

No source file added by this change is without an owning criterion:

| Added | Owning criterion |
| --- | --- |
| `examples/corpus_conformance.rs` | FR-006-AC-2, FR-006-AC-6 |
| `examples/roundtrip_sweep.rs` | FR-006-AC-2 |
| `scripts/assurance_chain.py` | FR-006-AC-2, AC-5, AC-6 |
| `scripts/check_shared_pins.py` | FR-006-AC-1 |
| `scripts/legacy_evidence_view.py` | FR-006-AC-4 |
| `assurance/pins.json` | FR-006-AC-1 |
| `assurance/change-assurance.json` | FR-006-AC-2 through AC-7 |
| `tests/shared_assurance.rs` | all seven |

Three scripts were kept from the old machinery and are domain gates rather than
evidence infrastructure: `check_attribution.py` (NFR-002-AC-1),
`rust_test_census.py` (FR-006-AC-2 via `PROOF-test-census`), and
`check_checksum_manifest.py` (FR-005-AC-1).

## Plan status

`PLAN-002` Task-001 and Task-002 are `done`; Task-003 is `in_progress` until
merge. `PLAN-001`'s `Task-007-human-release` stays `not_started`, correctly —
that gate belongs to a person, and the receipt says so: `incomplete`, with
`decision_missing`, because no ix-flow decision event exists and none was
synthesized.
