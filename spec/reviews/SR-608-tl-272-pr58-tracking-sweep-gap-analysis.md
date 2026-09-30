---
id: SR-608
title: "tl-parse#58 tracking-ceremony sweep gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-parse@72f862ec3bf54c8bdf96b7bf40e677eb57a9c267; diff against origin/main 86d4306; spec/spec.md; spec/test-matrix.md; spec/evidence/suites.md; spec/requirements/{FR-001,FR-005,FR-017,NFR-002}*.md; spec/requirements/{FR-006,NFR-003}*.md (deleted); spec/assurance/* (deleted); spec/plans/PLAN-001-v0.1/**; spec/plans/PLAN-002-shared-assurance-migration/** (deleted); spec/requirements/FR-009-organize-versioned-dialects.md (context); .github/workflows/ci.yml; Makefile"
review_set: subset
---

# tl-parse#58 tracking-ceremony sweep gap analysis

## Summary

Ticket: TL-272. This is a planless gap analysis of agent-ix/tl-parse#58 at
`72f862e`. It checks four things:

- AC-to-test traces after the deletions;
- dangling edges to removed artifacts;
- test matrix consistency;
- whether CI still resolves every step.

Plan completion: not assessed.

## Verdict

**CONDITIONAL: one medium and one low finding.** The spec stays structurally
consistent.

- **No dangling edges.** No live (non-review) frontmatter `relationships`
  target points at any of the deleted artifacts:
  - FR-006, NFR-003 and NFR-002-AC-2;
  - AA-001, AD-001, AP-001, CAC-001 and MP-001;
  - PLAN-002 and Task-006;
  - PGM-01 and SR-004.

  MRS-001 dropped its PGM-01 `depends_on`.
- **No removed TC is still referenced.** No test `// Trace:` names
  TC-022..029, TC-031 or TC-032. The matrix removed the FR-006 and NFR-003 rows
  and trimmed StR-002 and NFR-002 to the TCs that remain.
- **The trace diff is unchanged.** `quire coverage --strict` gives the same set
  of unmatched traces on main and on the PR. They are the pre-existing FR-013,
  FR-047 and TC-055/056/057/184 rows.
- **Rewritten ACs match their tests.**
  - FR-005-AC-1 and AC-2 match TC-018, TC-019 and TC-047.
  - FR-001-AC-3 and NFR-002-AC-1 match TC-020, which checks the "independently
    authored" and "MIT OR Apache-2.0" text.
  - FR-017-AC-2 matches TC-064.
- **Historical records reference deleted ids in prose only.** They are the
  retained `spec/reviews/SR-0xx` files and `PLAN-001/log.md`. They are archival
  and carry no edges.
- **CI still resolves (d).** `.github/workflows/ci.yml` calls only these:
  - `make assurance-env`, which still exists and builds from
    `requirements-assurance.txt`, which also still exists;
  - `make ci`, whose prerequisites all still exist (`fmt-check`, `lint`,
    `test`, `conformance`, `roundtrip`, `test-census`, `fuzz-build`,
    `fuzz-smoke`, `deny`, `audit-unsafe`, `test-execution-control-guard`,
    `spec`, `msrv` and `rustdoc`);
  - `cargo deny check licenses sources` and `cargo check --locked`.

  Every script those targets invoke is present. Two CI comments are now stale:
  one mentions "the assurance chain names an exact revision", and one step is
  named "Build the pinned assurance environment". Workflows are out of scope
  for edits. `make ci` still fails on its `spec` step, as it does on main,
  because of the pre-existing column-name assert.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-009 still requires that "provenance-document digests SHALL identify the newly pinned owner", and FR-009-AC-3 that "provenance digests bind that change". The PR removed the v1 dialect-document and attribution digest asserts, so only the v2 and v3 document digests back the clause, and the owner rule makes those ceremony (see SR-607 FND-001). Fix: drop the provenance-digest clauses from the FR-009 statement and AC-3, keeping the compiled-revision-field clause. | spec/requirements/FR-009-organize-versioned-dialects.md:54-55; spec/requirements/FR-009-organize-versioned-dialects.md:68 |
| FND-002 | low | The PR title is "drop dangling PGM-01 citations", but the PLAN-001 dependency DAG still starts at `PGM-01 + exact tl-syntax revision`. PLAN-004 still says "checksummed seeds". Fix: change the DAG root to `exact tl-syntax revision` and drop "checksummed". | spec/plans/PLAN-001-v0.1/plan.md:17; spec/plans/PLAN-004-clean-ascii-v2/plan.md:25 |

## Coverage

These ACs were examined and are clean:

- FR-001-AC-3
- FR-005-AC-1, FR-005-AC-2 and FR-005-AC-3
- FR-017-AC-2
- NFR-002-AC-1
- StR-002 and its row

The master requirements architecture paragraph and its responsibility table
were also examined and are clean.

The suite registry lost SUITE-005 and SUITE-007. That is correct, because
`check-corpus` and `assurance` are gone. The SUITE-010 wording matches the new
runner.

Optional semantic review was not run.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
