---
id: NFR-016
title: Keep the numeric dialect deterministic and resource-bounded
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-parse/NFR-001
    type: constrains
  - target: ix://agent-ix/tl-parse/FR-029
    type: constrains
---

# NFR-016: Keep the numeric dialect deterministic and resource-bounded

## Statement

The v5 parser and formatter shall return identical reports and canonical text for
identical source, catalog, profile and limits, and shall bound source, token,
node, depth, work and output by the existing hard maxima.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Repeat-run byte difference | 0 | 0 | Test |
| Allocation proportional to unbounded input | none | none | Test |
| Recursion on arithmetic depth | iterative | no stack overflow at depth 257 | Test |
| Existing dialect byte difference | 0 | 0 | Test |

## Verification

Corpus and property tests run twice and compare bytes; the depth-257 hostile case
and the work-limit cases run under Miri.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-016-AC-1 | Repeated parse and format runs are byte-identical and bounded by the hard maxima on the hostile corpus. | unit-testing (TC-1558) |
