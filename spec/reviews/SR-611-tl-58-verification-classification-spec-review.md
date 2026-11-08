---
id: SR-611
title: "Base spec review: verification classification of tl-parse acceptance criteria (TL-58)"
type: SpecReview
analysis: base
scope: "Verification cells of spec/requirements, the Verification Classification section of spec/test-matrix.md, and TC Type corrections in spec/infinite-trace-test-matrix.md"
review_set: base
---

## Summary

Every acceptance criterion's Verification cell now names a catalog method:
8 `property-based-testing`, 17 `unit-testing`, 16 `integration-testing`,
3 `fuzzing`, 1 `Inspection`. Property means a tagged proptest asserts the
criterion. Five infinite-trace test cases typed Property
(TC-058, TC-059, TC-061, TC-063, TC-066) had no generated-input test and are
retyped Unit. Gate: `make ci` passes. Applicable AssuranceProfile: none read
for this change; no required spec-review operation was identified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6111 | low | `quoin advise` flags 33 authored methods as differing from its recommendations. Recommendations are not verdicts, but 11 criteria are example-only where the advisor sees a universal shape and are candidates for new property tests: FR-002-AC-2, FR-003-AC-2, FR-005-AC-1, FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-008-AC-3, FR-015-AC-1, FR-016-AC-1, FR-016-AC-2, FR-017-AC-1. | spec/requirements |
| FND-6112 | low | TC-002, TC-003 and TC-008 are typed Unit while a proptest is also tagged to them. The criterion is classified property; the TC Type keeps the dominant example evidence. | spec/test-matrix.md |
| FND-6113 | low | FR-009-AC-5 is classified property because a proptest asserts the work and node bounds; the exact-limit and one-over cases are example tests. | spec/requirements/FR-009-organize-versioned-dialects.md |
| FND-6114 | low | Measurement rows NFR-001-M-1, NFR-001-M-2, NFR-002-M-1, NFR-004-M-1 and NFR-004-M-2 carry no classification; the test-matrix section states they are derived counts. | spec/test-matrix.md |

## Checklist result

ID formats, duplicates and link integrity: `quire validate --strict` and
`quire coverage --strict` pass with coverage unchanged at 97/104 rows backed.
No hash, digest or pin is introduced.
