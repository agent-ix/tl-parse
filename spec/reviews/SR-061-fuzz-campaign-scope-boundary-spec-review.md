---
id: SR-061
title: "Scope-boundary review — structured bounded-fuzz campaigns"
type: SpecReview
analysis: scope-boundary
scope: "FR-005, FR-006, NFR-003, MP-001, SUITE-010"
review_set: all
---

# Scope-boundary review — structured bounded-fuzz campaigns

## Summary

Responsibility is explicit: tl-parse owns target selection, seed validation,
process supervision, classification, and structured bytes; cargo-fuzz executes
the target; Quoin owns intake and retention; and the human release owner alone
decides release. No Quire, Quoin, or local generic framework executes or
reconstructs a producer result.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-5201 | low | The in-scope structured campaign result and out-of-scope binary attachment are allocated to their owning components with no local store or adapter substitution. | FR-005; FR-006; NFR-003; Quoin #363 |
| FND-5202 | low | The producer/shared-tool boundary is testable: tl-parse writes result bytes, Quoin retains them, Quire supplies only static facts, and neither shared tool invokes cargo-fuzz. | FR-006-AC-2; NFR-003-AC-1; TC-047 |
