---
id: SR-068
title: TL-14 base specification review
type: SpecReview
analysis: base
scope: "agent-ix/tl-parse@e7fe85f506ccfad2eb7835a7a8485505ff04619d; spec/requirements/FR-015-infinite-trace-text-dialect.md, FR-016-infinite-trace-loci-and-diagnostics.md, FR-017-infinite-trace-format-and-corpus.md, NFR-004-infinite-trace-parser-determinism.md, spec/infinite-trace-test-matrix.md, docs/DIALECT-004-clean-ascii-v4.md"
review_set: subset
---

## Summary

Reviewed the changed v4 requirements, dialect document and test matrix for IDs, specificity, cross references, boundaries, errors and six coverage rules. IDs and TC links validate, but the universal formatter criterion conflicts with its explicit unrepresentable-graph refusal.

## Verdict

FAIL: the acceptance criterion needs a representability condition or a separately testable refusal obligation before it can be interpreted consistently.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-017-AC-1 says every admitted v4 graph formats, while FR-017 Behavior explicitly refuses valid owner graphs with shared nodes or foreign node order. | spec/requirements/FR-017-infinite-trace-format-and-corpus.md:40; FR-017-AC-1 | wrong-requirement |

## Coverage

FR-015, FR-016 and FR-017 have complete input, output, behavior, error and dependency sections; NFR-004 names bounded resource axes and deterministic measurement. TM-002 maps all nine ACs to TC-058 through TC-066. Error paths, invalid profile/clock, malformed intervals, fairness uniqueness, source loci, canonical output, checked corpus and limit boundaries are represented. Exact legacy byte preservation lacks a test assertion; the separate gap-analysis artifact records it. No duplicate or malformed FR/NFR/TC IDs were found in this selected scope. Targeted `quire validate --scope .` reported 5/5 changed spec docs grammar-clean.
