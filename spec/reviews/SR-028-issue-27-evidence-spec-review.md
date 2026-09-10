---
id: SR-028
title: Evidence review of issue 27 controls
type: SpecReview
analysis: evidence
scope: "NFR-002, NFR-003, TM-001 additions for tl-parse issue 27"
review_set: all
---

## Summary

The catalog advisor accepts `Test` for both new obligations. Review corrected
NFR-002-AC-1 from `Inspection` to `Test` because TC-020 already executes its
document assertions; unrelated zero-count governance metrics retain inspection
by judgment rather than adopting inapplicable performance-benchmark advice.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2705 | low | No remaining evidence-method mismatch in the added scope; TC-031 and TC-032 provide concrete mutation-capable evidence for their respective obligations. | NFR-002-AC-2, NFR-003-AC-5 |
