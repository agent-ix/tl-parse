---
id: SR-047
title: "Failure-domain review — structured bounded-fuzz campaigns"
type: SpecReview
analysis: failure-domain
scope: "FR-005-AC-2, FR-006-AC-2, TC-046"
review_set: all
---

# Failure-domain review — structured bounded-fuzz campaigns

## Summary

The failure-domain lens covers target and seed identity, unavailable
prerequisites, launched-process failure, timeout, unexpected artifacts, and
unsafe artifact topology. No callback or user-supplied evaluation logic enters
this producer boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4701 | medium | **Fixed in the reviewed draft:** pass/fail/unavailable/suspect now have disjoint triggers, every non-pass exits non-zero, and recognized targets still emit structured output on setup or execution failure. | FR-005 Behavior; FR-005-AC-2 |
| FND-4702 | medium | **Fixed in the reviewed draft:** seed count/size, run count/time, and artifact count/size are explicitly bounded; symlinked, non-regular, or escaping seeds/artifacts cannot be silently admitted. | FR-005 Inputs; FR-005 Behavior |
| FND-4703 | medium | **Fixed in the reviewed draft:** structured artifact identities are retained, while lossless binary attachment is explicitly assigned to Quoin #363 and cannot be replaced with a local archive. | FR-005 Outputs; FR-005 Dependencies; FR-006 Dependencies |
