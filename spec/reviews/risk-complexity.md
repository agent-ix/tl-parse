---
id: SR-018
title: "Risk and complexity review of the complete tl-parse specification corpus"
type: SpecReview
analysis: risk-complexity
scope: "spec/"
review_set: all
---

## Summary

The highest technical risk is qualification-integrity drift across a composite
Make gate; its volatility is low because the failure mode is already known, but
the consequence is material for candidate review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1801 | high | The known Make execution-control bypass is a high technical-risk, low-volatility hazard: it can make multiple non-producer checks appear green while the retained-input chain remains intact. Preserve the named human release decision and tl-parse#11 mitigation dependency. | NFR-003, AP-001, SUITE-001 |
