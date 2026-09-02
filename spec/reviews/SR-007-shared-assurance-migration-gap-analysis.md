---
id: SR-007
title: Gap analysis of the shared assurance migration
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, docs/**/*.md, assurance/**, tests/**/*.rs, examples/**/*.rs, scripts/**, Makefile
review_set: subset
---

# Gap analysis of the shared assurance migration

## Summary

Coverage moves from 62/62 rows (100%) at the base revision to 68/72 rows (94%).
That is not a regression in what is checked; it is a change in what is counted,
and the four newly-unbacked rows are named rather than closed with a tag.

Six matrix rows are new (TC-022 through TC-028 replace the four evidence-machinery
rows TC-022 through TC-025). One requirement is new (FR-006, seven criteria).
Two requirements were rewritten because they described machinery this change
deletes (NFR-003 entirely, NFR-002-AC-2 and FR-005-AC-4 in part).

## Verdict

**CONDITIONAL.** No high findings. The coverage delta is explained below and is
deliberate; two mediums and two lows are recorded.

## Coverage

| Figure | Base `cf43f40` | This head |
| --- | --- | --- |
| Rows backed | 62/62 (100%) | 68/72 (94%) |
| `spec/test-matrix.md` | 25/25 | 28/28 (100%) |
| `spec/evidence/suites.md` | 9/9 | 5/9 (55%) |
| Rust bound/tagged/candidates | 26/26/26 | 36/36/36 (100%) |
| Requirement documents | 11/11 | 12/12 (100%) |
| Compiled requirement-tagged tests | 28 | 35 |

**The whole of the delta is the suite registry, and it is an improvement.** At
the base, all nine suites were backed by a single test — `tests/evidence_contract.rs`
— whose `// Trace:` comment named SUITE-001 through SUITE-009 at once. That is
the binding shape PR #6's reviewers criticised repeatedly: a row can be backed by
a tag on a test that does not exercise it.

Five suites now have a bound test. **Corrected after the adversarial round:** an
earlier version of this section said those tests "actually invoke that suite's
command", and none of them does. They bind to the suite's *retained output* —
TC-024 reads the Quire export rather than running `quire coverage --strict`,
TC-023 reads `msrv.jsonl` rather than running the MSRV check, and so on. That is
the architecture working as intended, since verdicts are supposed to come from
producer bytes, but it is a weaker claim and `spec/evidence/suites.md` now states
it as the weaker claim with a per-suite table.

Four are unbacked entirely: SUITE-001 is the composite gate, SUITE-002 is `quire
validate`, SUITE-004 is `rustdoc`, SUITE-008 is a manual hosted dispatch. The
only available way to make them read as backed is to assert that the Makefile
*contains* the command.

Five output-bound rows and four honestly empty ones is a better record than nine
rows backed by one test that tagged every suite at once — but it is not the
"invokes the command" claim originally written, and the distinction is the point.

## Traceability of the new requirement

FR-006 has seven acceptance criteria and each is owned by exactly one test:

| Criterion | Test | What would break it |
| --- | --- | --- |
| FR-006-AC-1 | TC-022 | a version rule restated locally, a mirror reference, a pin disagreement across the four files that name it |
| FR-006-AC-2 | TC-023 | an attested result not derived from producer bytes; a producer executed by Quoin or Quire |
| FR-006-AC-3 | TC-024 | an impact snapshot that is not the Quire export, or an export missing a requirement |
| FR-006-AC-4 | TC-025 | a moved evidence byte, a misattributed record, an unaccepted positive control |
| FR-006-AC-5 | TC-026 | any of the twelve states undemonstrated, or a negative without a control |
| FR-006-AC-6 | TC-027 | a malformed row reported as a pass, or a count disagreeing with the corpus manifest |
| FR-006-AC-7 | TC-028 | a surviving machinery file, a changed frozen schema, a reference to one |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-701 | medium | Four suite-registry rows are unbacked. Deliberate, explained in the registry, and preferable to the single over-broad tag it replaces — but it is a real gap and a reader comparing 100% to 94% deserves the reason rather than the number | `spec/evidence/suites.md` |
| FND-702 | high | Raised from medium by the adversarial round, which demonstrated it rather than reasoning about it: `.IGNORE:` neuters `fmt-check`, `lint`, `test`, `check-corpus`, `fuzz-build`, `fuzz-smoke`, `deny`, `audit-unsafe`, `rustdoc` and half of `spec`, and every remaining check stays green. The claim that the structural replacement covered the class was false and is corrected in NFR-003, AA-001, the declaration and the Makefile header. Filed as `agent-ix/tl-parse#11` | `Makefile` |
| FND-703 | low | `[status-column-matches-nothing]` persists: the traceability declaration expects a `Status` column and the matrix authors `Coverage Status`, so status classification is skipped and complete-but-unbacked rows cannot be detected. Not this repository's to fix alone; tracked program-wide as `agent-ix/quire-contract-ir#21` | `spec/test-matrix.md` |
| FND-704 | low | `[catch-all-universal]`: 2 of 7 documents binding extractable criteria name a specific property shape for none of them. Carried over from the base revision, unchanged by this migration | `spec/requirements/FR-003-diagnostics-limits.md` |

## What the migration removed from the specification, and why

NFR-003 previously owned `tools.lock`, the local-CI runner, the propagation
probes, the collector, the finalizer, the verifiers and the retraction registry.
All of those are deleted, so the requirement was rewritten around what actually
exists now: the producer boundary, results derived from producer bytes, the
twelve-state vocabulary, and the absence of release authority. Its Scope section
states plainly what it no longer owns and what that costs.

FR-005-AC-4 and NFR-002-AC-2 both named the deleted verifier. Both now name the
compatibility view, which reads the same bytes without modifying them and asks
Git whether they are the committed bytes.

Nothing was removed from the specification without a replacement criterion or a
recorded reason. The four evidence-machinery test rows TC-022 through TC-025 are
replaced by seven rows covering the shared path, so the matrix grew by three.

## Plan status

`PLAN-002` is `in_progress`: Task-001 and Task-002 are `done`, Task-003 remains
`in_progress` until merge. `PLAN-001`'s `Task-007-human-release` stays
`not_started`, correctly — that gate belongs to a person and no automated step
in this change may close it. The receipt reflects that: it reads `incomplete`
with `decision_missing`, because no ix-flow decision event exists and none was
synthesized.
