---
id: SR-034
title: "Failure-domain review of the complete tl-parse specification corpus"
type: SpecReview
analysis: failure-domain
scope: "spec/"
review_set: all
---

## Summary

The review examined parser rejection, graph construction, canonicalization,
producer boundaries, and hostile-input resource limits. The corpus explicitly
records a failure domain whose non-producer gates can be bypassed through Make
execution controls, but it remains an open qualification hazard.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1401 | high | NFR-003 documents that `.IGNORE:` can make `fmt-check`, `lint`, `test`, corpus, fuzz, documentation, and spec-validation gates report success without execution; the producer-byte backstop does not cover those gates. The residual must remain an explicit release-decision risk until the shared control tracked by tl-parse#11 exists. | NFR-003, FR-006, SUITE-001, SUITE-002 |
