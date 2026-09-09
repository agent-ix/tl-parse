---
id: SR-021
title: "Base specification review — tracked SpecReview identity uniqueness"
type: SpecReview
analysis: base
scope: "agent-ix/tl-parse#23 review response; NFR-003-AC-4; TC-029; spec/reviews"
review_set: base
relationships:
  - target: ix://agent-ix/tl-parse/NFR-003
    type: reviews
---

# SR-021: Base specification review — tracked SpecReview identity uniqueness

## Summary

The base review checked identity allocation, requirement grammar, failure and
edge cases, criterion-to-test traceability, and the dependency order between
open pull requests #23 and #26. PR #23 owns SR-013 through SR-021; PR #26 must
rebase and allocate later identities after this branch lands.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-2101 | high | PRs #23 and #26 independently allocate SR-013 and SR-014, and strict validation accepts the combined duplicate identities. The reviewed contract adds a tracked SpecReview uniqueness census and fixes #23 as the first allocation owner. | NFR-003-AC-4, TC-029, tl-parse#23, tl-parse#26 |
| FND-2102 | medium | Eight new reviews used analysis-only filenames, hiding their identities from directory listings and ordinary Git conflicts. Their filenames now carry SR-013 through SR-020. | spec/reviews |
| FND-2103 | medium | A line comparison that preserves YAML quote characters can miss that `SR-091` and `"SR-091"` are the same frontmatter identity; an empty scan can also pass vacuously. Both cases are explicit negative controls. | NFR-003-AC-4, TC-029 |

## Dispositions

All findings are resolved at specification level. Implementation is limited to
first-party Rust inside the existing shared-assurance test target, plus the
sealed Quire population totals that the added criterion and matrix row change.
No dependency, parser behavior, hosted trigger, release authority, or generic
assurance framework is authorized.
