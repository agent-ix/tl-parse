---
id: SR-010
title: Code review — drop the retained legacy evidence
type: SpecReview
analysis: code-review
scope: "evidence/**, schemas/**, scripts/legacy_evidence_view.py, tests/fixtures/legacy-compat/**, scripts/assurance_chain.py, scripts/check_shared_pins.py, assurance/**, Makefile, .github/workflows/ci.yml, tests/shared_assurance.rs, spec/**/*.md, README.md, CLAUDE.md"
review_set: subset
---

# SR-010: Code review — drop the retained legacy evidence

## Summary

This change deletes 721 files. Every other question about it is secondary to one:
**does anything still need what was removed?** The review is scaled to the change
— it is a deletion, not a design — but that single question was answered by
grepping this repository's own tree rather than by copying a sibling's answer,
because the brief for this work records that `quire-contract-codegen` froze a
different set of schemas for a live reason and inheriting its list would have
been wrong.

## Verdict

**PASS.** Four findings: one medium and three low. All four are dispositioned
below — one FIXED, three ACCEPTED with rationale — and none blocks. Exact-head
gates are green.

## Authority

The repository owner released the evidence-preservation constraint for the
pre-stable phase on 2026-09-02. It is recorded in
`agent-ix/engineering-assurance#7` under "Preservation constraint released for
the pre-stable phase", by an agent transcribing the decision rather than making
it. The epic's completion criterion and its mandatory control were amended before
this change, so no live constraint was violated. Tracked in this repository as
`agent-ix/tl-parse#13`.

## What was deleted, measured

| Group | Files | Lines | Bytes at base |
| --- | --- | --- | --- |
| `evidence/` | 709 | 13,981 | 4,083,773 |
| `schemas/` | 3 | 186 | — |
| `tests/fixtures/legacy-compat/` | 8 | 321 | — |
| `scripts/legacy_evidence_view.py` | 1 | 462 | — |
| **Total deleted** | **721** | **14,950** | — |
| Edited in place | 16 | +115 / −414 | — |

All 709 evidence files were byte-identical through the #10 migration; none was
rewritten before deletion, and none is rewritten now.

## The load-bearing check: does anything still need this?

### The two schemas

`schemas/tl-parse-evidence-input-v1.schema.json` and
`schemas/tl-parse-evidence-manifest-v1.schema.json` were frozen by the #10
migration for exactly one stated reason: retained envelopes named them by path
and SHA-256, and `assurance/change-assurance.json` carried a preservation
constraint saying so. `schemas/README.md` stated the same and added that nothing
validates against them.

That statement was verified rather than trusted:

| Probe | Result |
| --- | --- |
| `grep -rn` for both filenames across `src/`, `scripts/`, `tests/`, `assurance/` | only the freeze declaration, the README, and `tests/shared_assurance.rs`'s own digest pins |
| `grep -rni 'include_str\|include_bytes\|schema'` across `src/` | two `include_str!` calls, both on `docs/*.md`; no schema is compiled in |
| any Python `jsonschema` import | none — the Makefile itself records that nothing in this repository imports `jsonschema` any more |

Both are dead. This repository's frozen set was two and both went. It did **not**
inherit a sibling's list: `quire-contract-codegen` keeps
`pgm01-derivation-evidence-envelope-v1.schema.json` because `src/oracle.rs`
includes it as a live output contract, and this repository has no such file and
no such reference.

`schemas/` is therefore empty and gone, and `schemas` is dropped from
`record.subject.scope` in the change-assurance declaration.

### The chain obligation

`PROOF-legacy-compatibility` had four attachment points and all four are gone:
the `INPUTS` entry with its media type, the `derive_result` branch, the
`assurance-inputs` line that wrote its result, and the declaration's proof
obligation. Six proof obligations remain and `TC-023`'s count assertion moved
from seven to six, so a silently dropped obligation is still a test failure.

### The twelve verification outcomes — the one real risk

`TC-026` requires all twelve states to be demonstrated by cases that ran and
matched. Before this change the set was built from two sources: the chain's
`states_demonstrated`, and the compatibility census's `mapped_states`. Deleting
the census could have taken a state with it, and the test would have gone red
only after the deletion was irreversible.

This was measured **before** anything was deleted. The baseline chain run at
`150b440` reported:

```
states_demonstrated: [fail, inconclusive, malformed, not-computed, partial,
                      pass, stale, suspect, tampered, unavailable, unsupported,
                      vacuous]
```

Twelve of twelve, from the chain alone. The census contributed nothing unique, so
removing it moved no state out of reach. `TC-026` now reads one source instead of
two and still asserts all twelve.

### The malformed mapping, deliberately untouched

This repository maps `malformed` to a **passing** proof. That is not an oversight
inherited from the evidence machinery — it is domain behaviour. Six of seven
corpus fixtures are malformed by design, and the count is checked against
`corpus/v1/manifest.json`'s own declaration rather than against the producer, so
a producer that stopped reporting malformed rows cannot also move the number it
is compared to. `TC-027`, the three chain scenarios, the
`malformed-input-is-reported-as-malformed` control and the adapter's
`domainOutcome` carry-through are all unchanged. Nothing in the malformed corpus
or its oracle was touched.

### The pins

`assurance/pins.json` pinned four Engineering Assurance artifacts by digest:
`verification_semantics.py`, `schemas/pgm01-compatibility-view-v1.schema.json`,
and the two PGM-01 positive-control fixtures. Every one of them was read by the
deleted view and by nothing else — the pins file said so itself. They are removed
because `consumed_artifacts` is documented as "the digests of the artifacts it
actually reads", and pinning bytes this repository no longer reads would be a
claim about work it does not do. See FND-1001.

`compatibility-matrix.json` stays, digest-free and for the same recorded reason
as before. `check_shared_pins.py` still classifies all four components through
the packaged matrix, and the `tl-syntax` upstream-pin check is unaffected.

### The tl-syntax pin

`Cargo.toml:23` pins `953ee825e5060335b4c79682f5f41a78c5a1bfae`. Not touched.
`docs/ATTRIBUTION.md`, `scripts/check_attribution.py` and the `740182f1`
authorship basis are not touched.

### The Make execution-control guard

Not re-added. Its absence is recorded, not closed, and `agent-ix/tl-parse#11`
carries the measured numbers — `ci:` has 16 prerequisites and `.IGNORE:` neuters
ten. `ci:` is unchanged at 16 prerequisites by this change. `NFR-003` and
`AA-001` keep their disclosure of the gap verbatim.

## Claims removed rather than restated more weakly

The brief's rule is that a claim which argues from the deleted evidence is
removed with it, never softened. Applied:

| Claim | Disposition |
| --- | --- |
| `FR-006-AC-4` — retained bytes read through the mapping unmodified | deleted |
| `FR-005-AC-4` — every retained byte preserved, release authority a human's | deleted whole, including the release-authority half |
| `NFR-002-AC-2` — retained evidence read without a byte moving | deleted |
| `NFR-003-AC-4` — Git consulted for whether retained bytes are committed bytes | deleted |
| `TC-025` and its test | deleted |
| frozen-schema half of `FR-006-AC-7` / `TC-028` | deleted; the runner/envelope/store half kept |
| `PRESERVE-legacy-bytes`, `PRESERVE-frozen-schemas` | deleted |
| `UNKNOWN-derivation-evidence-not-mapped` | deleted; `engineering-assurance#21` is moot, not fixed |
| `AA-001` "Retained Evidence" section | replaced by a section that states the records were deleted and that the argument does not claim them |

`FR-005-AC-4`'s release-authority sentence deserves a note, because dropping it
looks like losing a control. It is not restated as a weaker acceptance criterion.
`NFR-002`'s "Automated release approvals: 0" metric moved from Method `Test` to
Method `Inspection`, which is what it now is — the test that backed it read
retained evidence. Claiming `Test` for a metric with no test would be the exact
false green this repository's own review history exists to catch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1001 | medium | `artifact_digest_mismatches` now iterates one artifact, which has no digest, so it returns an empty list because there is nothing to compare — not because a comparison passed. `TC-022` asserts that list is empty and is vacuous on that assertion. | `assurance/pins.json`, `scripts/check_shared_pins.py`, TC-022 |
| FND-1002 | low | `TC-028` lost its source census: the walk over `scripts`, `tests`, `examples`, `src`, `spec`, `docs`, `.github` plus the build files, with its `inspected > 30` anti-vacuity assertion, existed only to prove nothing referenced the frozen schemas. | `tests/shared_assurance.rs`, TC-028, FR-006-AC-7 |
| FND-1003 | low | `spec/plans/PLAN-002-*` and `spec/reviews/SR-002..SR-009` still name `scripts/legacy_evidence_view.py`, `evidence/`, `compat-view` and `FR-006-AC-4`. | `spec/plans/PLAN-002-shared-assurance-migration/`, `spec/reviews/` |
| FND-1004 | low | `make assurance-record` remains an operator target that calls `quoin evidence record --repo .`, which a reader could mistake for a surviving local evidence collector. | `Makefile` |

### Dispositions

| ID | Disposition | Rationale |
| --- | --- | --- |
| FND-1001 | **ACCEPTED** | The alternative is worse: keeping digests for artifacts this repository does not read would make `consumed_artifacts` a false statement, and the check would be re-hashing bytes nothing consumes. The vacuity is written into `assurance/pins.json` as `consumed_artifacts_note` rather than left for a reader to discover. `TC-022`'s other assertions — four components classified compatible, both mirror-scan branches observed refusing, the upstream-pin cross-check — are unaffected and not vacuous. |
| FND-1002 | **FIXED** | Fixed in shape rather than restored: the census had no subject once the schemas were gone. What replaced it is narrower and cheaper — `evidence`, `schemas`, `scripts/legacy_evidence_view.py` and `tests/fixtures/legacy-compat` are added to `TC-028`'s existing removed-by-name list, and `compat-view` to its Makefile-target list, so reintroducing any of them fails a test. |
| FND-1003 | **ACCEPTED** | These are dated records of work that happened. Editing them to remove mentions of something that existed at the time would be backdating the record, which is the one thing this change is forbidden to do. They are prose, not links, and `quire validate --strict` passes over every document. |
| FND-1004 | **ACCEPTED** | It writes into Quoin's own store under a reviewed change to `spec/evidence/`, not into a repository-local `evidence/` collector directory, and it is not a `ci` prerequisite. It is unrelated to the deleted machinery. |

## Exact-head gates

| Gate | Result |
| --- | --- |
| `make ci` | exit 0 |
| `cargo test` — `tests/shared_assurance.rs` | 9 passed, 0 failed (was 10) |
| `quire validate --scope . --strict` | 48/48 documents grammar-clean, 0 findings |
| `quire coverage --scope . --strict` | 63/67 rows backed (94%) |
| `scripts/assurance_chain.py` | 17 scenarios, 7 controls, 7 probes, all matched |
| `scripts/check_shared_pins.py` | 4/4 components compatible, accepted |
| Reference grep for deleted material | no live code, Make target, schema, fixture, spec row or chain obligation |

Hosted CI is intentionally undispatched and remains `workflow_dispatch` only.

## Assurance Context

**Claim boundary.** This change removes retained records and the machinery that
read them. It makes no claim that those records verified anything, and it makes
no new claim about the parser, the formatter, the corpus, or the chain.

**Authoritative policy.** `agent-ix/engineering-assurance#7`, section
"Preservation constraint released for the pre-stable phase", as amended
2026-09-02 by the repository owner. The control it releases re-applies unchanged
when this repository moves toward stable releases.

**Trust inputs.** The pinned Engineering Assurance 0.2.0 compatibility matrix,
`quire-cli` 0.31.0, `quoin` 0.23.1, `ix-flow` 0.0.4, and the `tl-syntax` revision
`953ee825`. All unchanged by this work.

**Failure posture.** Unchanged and fail-closed. An absent producer input is an
error naming `make assurance-inputs`, never a skip. A missing assurance
interpreter is a test failure, never a skip. The one detection this change
removes is the retained-evidence byte comparison, which had no subject left.

**Execution boundary.** Unchanged. Neither Quire nor Quoin executes a producer;
`TC-023` still proves it with stubs and a control.

**Retained-output identity.** This repository now retains nothing of its own. The
only verification evidence is what the chain produces at the reviewed revision,
sealed and verified through Quoin.
