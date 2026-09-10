---
id: SR-025
title: Failure-domain review of issue 27 controls
type: SpecReview
analysis: failure-domain
scope: "NFR-002, NFR-003, TM-001 additions for tl-parse issue 27"
review_set: all
---

## Summary

The provenance criterion fixes both revision identities and separates consumed
from unconsumed API families. The workflow criterion covers comments, duplicate
and alias identities, automatic triggers, and missing or wrong runtime versions.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2702 | low | No unstated failure-domain issue found in the added scope; each relevant identity and negative workflow mutation has an explicit refusal oracle. | NFR-002-AC-2, NFR-003-AC-5 |
