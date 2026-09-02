---
id: SR-011
title: Gap analysis — drop the retained legacy evidence
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, docs/*.md, assurance/**, tests/**/*.rs, scripts/**, Makefile
review_set: subset
---

# SR-011: Gap analysis — drop the retained legacy evidence

## Summary

Five matrix rows were removed and five were backed, so coverage moves from 68/72
(94%) to 63/67 (94%). The headline percentage is unchanged, which is the least
interesting thing about it: what matters is that the four rows that were
deliberately unbacked before are the same four now, and no row became unbacked as
a side effect of the deletion.

## Verdict

**PASS.** One medium finding, carried forward from SR-010 rather than restated as
new. No undisclosed unbacked row.

## Coverage

| Figure | Before (`150b440`) | After |
| --- | --- | --- |
| Rows backed | 68/72 (94%) | 63/67 (94%) |
| `spec/test-matrix.md` | 28/28 (100%) | 27/27 (100%) |
| `spec/evidence/suites.md` | 5/9 (55%) | 5/9 (55%) |
| Rust bound/tagged/candidates | 38/38/38 | 37/37/37 (100%) |
| Requirement documents | 12/12 (100%) | 12/12 (100%) |
| Acceptance criteria | 35 | 31 |
| Compiled requirement-tagged tests | 37 | 36 |
| Documents grammar-clean | 46/46 | 48/48 (100%) |
| `status_lies` | 0 | 0 |

The five removed rows are `FR-005-AC-4`, `FR-006-AC-4`, `NFR-002-AC-2`,
`NFR-003-AC-4` and `TC-025`. Four acceptance criteria plus one test-case row;
72 − 5 = 67 and 68 − 5 = 63, so every removed row was backed and the arithmetic
closes with nothing unaccounted for.

Documents rose by two because this analysis and SR-010 are new. `schemas/README.md`
and `evidence/README.md`, both deleted, were never inside the validation glob
`spec/**/*.md docs/*.md`, so their removal moves no document count.

## The orphan-row check

The specific failure this change could have caused is an acceptance criterion
left in a requirement document with its only test deleted underneath it. Quire
reports that as an unbacked row and turns a clean gate red. Each of the four
criteria that traced only to `TC-025` was therefore removed from its requirement
document in the same change as the test:

| Criterion | Only test | Removed from |
| --- | --- | --- |
| `FR-006-AC-4` | TC-025 | `spec/requirements/FR-006-shared-assurance-intake.md` |
| `FR-005-AC-4` | TC-025 | `spec/requirements/FR-005-corpus-cli-evidence.md` |
| `NFR-002-AC-2` | TC-025 | `spec/requirements/NFR-002-provenance-boundary.md` |
| `NFR-003-AC-4` | TC-025 | `spec/requirements/NFR-003-qualification-integrity.md` |

The reverse direction was checked too: the summary rows in `spec/test-matrix.md`
that listed `TC-025` under FR-005, NFR-002 and NFR-003 were updated, so no
summary row names a test case that no longer exists.

`FR-006` now runs AC-1, AC-2, AC-3, AC-5, AC-6, AC-7 — a deliberate gap at AC-4
rather than a renumbering. Renumbering would have rewritten identifiers that the
change-assurance declaration, four review documents and the migration plan all
name, and would have made the deletion look like a revision.

## Requirement-to-test traceability after the change

| Requirement | Criteria | Tests |
| --- | --- | --- |
| FR-005 | AC-1..AC-3 | TC-018, TC-019, TC-020, TC-021 |
| FR-006 | AC-1..AC-3, AC-5..AC-7 | TC-022, TC-023, TC-024, TC-026, TC-027, TC-028 |
| NFR-002 | AC-1 | TC-020 |
| NFR-003 | AC-1..AC-3 | TC-023, TC-026 |

Every remaining criterion has at least one compiled, requirement-tagged test that
Cargo runs. `scripts/rust_test_census.py` reports `matched: true` with 36
compiled and 36 tagged and no ignored test, so no criterion is backed by a test
that exists only in source.

## What each surviving test would still catch

| Criterion | Test | A defect it would catch |
| --- | --- | --- |
| FR-006-AC-1 | TC-022 | a component classified by a local restatement of the matrix; a mirror registry reference in a scanned file or in the pins structure |
| FR-006-AC-2 | TC-023 | a proof attested from an exit code rather than producer bytes; Quoin or Quire executing a producer; a dropped proof obligation (the count is pinned at six) |
| FR-006-AC-3 | TC-024 | an impact snapshot naming an export it does not contain; an export that measured nothing; a moved coverage total |
| FR-006-AC-5 | TC-026 | any of the twelve states ceasing to be demonstrated; a negative with no positive control; a control naming a scenario that does not exist |
| FR-006-AC-6 | TC-027 | a malformed row count that disagrees with the corpus manifest; malformed dragging its proof to failed; the adapter dropping `domainOutcome` |
| FR-006-AC-7 | TC-028 | any named machinery file returning, including `evidence/`, `schemas/`, the compatibility view, the legacy fixtures, and a `compat-view` Make target |
| NFR-002-AC-1 | TC-020 | a dialect record whose compiled digest is not re-derived from the source Cargo resolved |
| NFR-003-AC-1..3 | TC-023, TC-026 | as above for the producer boundary and the twelve states |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1101 | medium | Same as SR-010 FND-1001: `TC-022`'s `artifact_mismatches` assertion is vacuous because `consumed_artifacts` has no digest-bearing entry left. | `assurance/pins.json`, TC-022, SR-010 FND-1001 |
| FND-1102 | low | Four suite-registry rows remain unbacked: SUITE-001 (`make ci`), SUITE-002 (`quire validate`), SUITE-004 (`rustdoc`), SUITE-008 (manual hosted dispatch). | `spec/evidence/suites.md` |

### Dispositions

| ID | Disposition | Rationale |
| --- | --- | --- |
| FND-1101 | **ACCEPTED** | Rationale in SR-010 and stated in `assurance/pins.json`. Carried forward, not double-counted as a new finding. |
| FND-1102 | **ACCEPTED** | Pre-existing and unchanged by this work. The reason is recorded in `spec/evidence/suites.md` and in SR-006 FND-604, SR-007 FND-701 and SR-009 FND-902: the only way to make them read as backed is a source-text grep, which is the binding shape earlier reviewers rejected. |

## Deliberate non-changes

- The malformed-input corpus, `corpus/v1/manifest.json`, and the count oracle
  that reads it. `FR-006-AC-6` and `TC-027` are untouched, and `malformed` still
  maps to a passing proof because six of seven fixtures are malformed by design.
- `agent-ix/tl-parse#11` and the Make execution-control disclosure in `NFR-003`
  and `AA-001`. The guard is not re-added and the recorded numbers are not
  re-litigated.
- The `tl-syntax` pin at `953ee825` and the `740182f1` authorship basis.
- `spec/plans/PLAN-002-*` and `spec/reviews/SR-002..SR-009`, which record work as
  it was done and are not backdated.
