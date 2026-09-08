---
id: SR-014
title: "Base specification review — grouping span provenance"
type: SpecReview
analysis: base
scope: "FR-002, TM-001, SR-013"
review_set: base
---

# Base specification review — grouping span provenance

## Summary

The owner selected the base review set for the grouping-span amendment. The
review checked EARS grammar, identifier integrity, criterion-to-test linkage,
trace tags, and scope containment. The amendment specifies lexical child-span
preservation without inventing a separate grouping-extent data model.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1401 | low | No base-review defect found: FR-002-AC-4 resolves to TC-029, the parser test observes the public document, and the wording does not claim an unavailable side-table extent. | FR-002-AC-4, TC-029, tests/parser.rs:156 |
