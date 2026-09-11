---
id: SR-036
title: "Dependency review of the complete tl-parse specification corpus"
type: SpecReview
analysis: dependency
scope: "spec/"
review_set: all
---

## Summary

The review checked declared tl-syntax, Quire, Quoin, Engineering Assurance, and
ix-flow boundaries against the current parser specification. The compiled
tl-syntax identity is not represented consistently across the corpus.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1601 | medium | FR-002 and Cargo pin tl-syntax at `6ad7499f…`, while the change-assurance declaration still describes `953ee825…` as the compiled revision. A candidate cannot bind one dependency identity in code and another in its assurance statement. | FR-002, FR-007, NFR-002, assurance/change-assurance.json |
