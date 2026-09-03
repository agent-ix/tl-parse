---
id: SR-012
title: Review of dangling-control probe isolation
type: SpecReview
analysis: code-review
scope: tests/shared_assurance.rs, spec/requirements/NFR-003-qualification-integrity.md, spec/test-matrix.md
review_set: all
---

# Review of dangling-control probe isolation

## Summary

The Agent C sibling sweep requested by tl-rewrite PR #17 FND-1705 found that
tl-parse TC-026 symlinked the repository `target` into its scratch. The mutated
driver therefore resolved its Quoin store through the symlink to the real
`target/assurance-store`. The refusal assertion pinned the error message, but
the fixture did not prove that its environment could complete an unmutated run.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1201 | medium | FIXED. The scratch owns an ordinary `target/`, shares only the already-produced `target/assurance` inputs, canonicalizes both stores and requires them to differ. | NFR-003-AC-4, TC-026 | implementation-bug-despite-evidence |
| FND-1202 | low | FIXED. The deliberately mutated driver must refuse the dangling scenario, while the original driver must succeed in the same scratch. | NFR-003-AC-4, TC-026 | correct-requirement-no-evidence |
| FND-1203 | low | FIXED. Symlink creation is fail-closed, and cleanup explicitly unlinks every shared input before removing only the owned scratch directories. | NFR-003-AC-4, TC-026 | implementation-bug-despite-evidence |

## Disposition

All three findings are fixed under issue #17. Creating `target/` after the
root-entry loop makes the ownership assertion fail if the skip set regresses.
The deliberate mutation must exit 2 naming the dangling scenario, then the
original driver must exit 0 in the same scratch.

This changes only the test environment. It does not change the parser, domain
producers, Quoin/Quire boundary, evidence contract, or manual-only hosted
workflow.

## Verification

Exact-head targeted and full local results are recorded on the pull request.
Hosted CI is not dispatched.
