---
id: SR-030
title: Scope-boundary review of issue 27 controls
type: SpecReview
analysis: scope-boundary
scope: "NFR-002, NFR-003, TM-001 additions for tl-parse issue 27"
review_set: all
---

## Summary

tl-parse owns its attribution documents, local tests, and hosted workflow. It
assumes the immutable upstream commit bytes and released npm package identity,
while verifying the consumed API statement, workflow text, and local executable
version without claiming upstream semantics or release authority.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2707 | low | No boundary allocation gap found; upstream tl-syntax and the npm distribution remain external while every changed repository artifact has a tl-parse owner. | NFR-002, NFR-003 |
