---
id: SR-047
title: "Base specification review — versioned dialect architecture"
type: SpecReview
analysis: base
scope: "FR-009, FR-002 dependency identity, MRS-001 allocation, TM-001 TC-046"
review_set: subset
---

# Base specification review — versioned dialect architecture

## Summary

Reviewed the complete FR-009 architecture and all five criteria against the
pinned tl-syntax owner contract, existing v1/v2/v3 compatibility boundaries,
and TC-046. The review included requirement identity, normative language,
dependency allocation, failure behavior, verification method, matrix status,
and contradictions with immutable owner limits.

## Verdict

**VALIDATED.** FR-009 is complete, internally consistent, allocated to the
correct owners, and runnable through TC-046. Every review finding was corrected.
The correction is focused: it does not change the nine-repository architecture
or add a new feature, subsystem, vocabulary, evaluator, or source language.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4701 | high | The first compatibility clause required formerly accepted 5,000-deep graphs to remain successful even though the pinned syntax owner immutably refuses graph depth above 4,096, making AC-3 incompatible with AC-4. Resolved by restricting byte-preservation to owner-admissible graphs and requiring typed refusal beyond the immutable ceiling. | FR-009 architecture; FR-009-AC-3; TC-046 |
| FND-4702 | medium | “Report bytes remain byte-identical” contradicted required advancement of the exact `tl_syntax_revision` field and provenance digests. Resolved by declaring those bound identity fields as the only byte changes. | FR-009-AC-3; NFR-002; `docs/ATTRIBUTION.md` |
| FND-4703 | low | FR-002 named a historical tl-syntax pin and the matrix classified TC-046 as planned. Resolved by updating the dependency identity and marking the implemented traced case covered. | FR-002; TM-001; TC-046 |

## Validation

- Focused Quire structural validation: 3/3 documents grammar-clean, zero grammar findings.
- FR-009 coverage: 5/5 criteria backed by TC-046 with no status lie or unbacked row.
- The installed Quire archetype requires the `Coverage Status` header while its
  advisory coverage configuration still looks for `Status`; the structurally
  valid archetype spelling is retained. This external module inconsistency does
  not alter any FR-009 row or backing result.
