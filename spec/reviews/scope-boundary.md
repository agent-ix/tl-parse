---
id: SR-019
title: "Scope-boundary review of the complete tl-parse specification corpus"
type: SpecReview
analysis: scope-boundary
scope: "spec/"
review_set: all
---

## Summary

The master specification clearly excludes a second AST, evaluation, monitoring,
and qualification authority, and names shared upstream tools. It does not yet
allocate every requirement to an owning component and responsibility class.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1901 | medium | The corpus has scope prose but no complete responsibility allocation for FR-001 through FR-006 and NFR-001 through NFR-003, nor an assumed-versus-guaranteed table for tl-syntax, Quire, Quoin, Engineering Assurance, and ix-flow. Add that allocation before using the corpus to split cross-repository work. | MRS-001, FR-006, NFR-003 |
