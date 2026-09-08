---
id: SR-015
title: "Integrity review of the complete tl-parse specification corpus"
type: SpecReview
analysis: integrity
scope: "spec/"
review_set: all
---

## Summary

Trace links and requirement artifacts are structurally valid, and each current
criterion has a matrix reference. The matrix's aggregate status is inconsistent
with its detailed case statuses for the same context-binding scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1501 | medium | Matrix aggregation is not internally consistent: FR-007 and StR-003 are marked planned even though all six referenced TC rows are marked implemented. This weakens the corpus's single interpretation of readiness. | FR-007, StR-003, TM-001 |
