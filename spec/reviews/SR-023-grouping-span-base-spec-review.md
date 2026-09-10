---
id: SR-023
title: "Base specification review — grouping span provenance"
type: SpecReview
analysis: base
scope: "FR-002, TM-001, SR-022"
review_set: base
---

# Base specification review — grouping span provenance

## Summary

The owner selected the base review set for the grouping-span amendment. The
review checked EARS grammar, identifier integrity, criterion-to-test linkage,
trace tags, and scope containment. After independent review exposed the missing
ancestor half of the contract, the amendment now specifies both delimiter-free
grouped-node spans and balanced grouping extents for enclosing operators without
adding a second public AST.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2301 | high | Resolved in the current specification: the first criterion covered only the grouped child's lexical span and left enclosing operator extents unspecified. FR-002-AC-4 and TC-030 now require both facts. | FR-002-AC-4, TC-030, tests/parser.rs, tl-parse#26 |
