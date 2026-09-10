---
id: SR-027
title: Dependency review of issue 27 controls
type: SpecReview
analysis: dependency
scope: "NFR-002, NFR-003, TM-001 additions for tl-parse issue 27"
review_set: all
---

## Summary

Both additions are enablement controls and are independent: attribution content
and its test can land without the workflow scanner, and the scanner can land
without changing the dependency pin. No cycle or feature dependency is added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2704 | low | No dependency defect found; TC-031 and TC-032 may be implemented in either order and converge at the local candidate gate. | TC-031, TC-032 |
