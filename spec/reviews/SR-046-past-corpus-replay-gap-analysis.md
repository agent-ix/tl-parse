---
id: SR-046
title: "Gap analysis — shared past/history parser replay"
type: SpecReview
analysis: gap-analysis
scope: "PLAN-010 Task-005 parser allocation; TC-056; FR-013-AC-2; FR-013-AC-3"
review_set: subset
---

# Gap analysis — shared past/history parser replay

## Summary

Traced every parser-owned Task-005 source, graph, profile, compatibility, and
digest obligation to compiled tests.

## Verdict

**VALIDATED.** All eight source/document pairs replay through clean-ascii/v3,
all preserve profile/schema identity, every canonical source is a fixed point,
and both prior dialects remain closed against the new vocabulary. The exact
owner manifest and all replay files are digest pinned.

No parser-owned obligation or untraced production behavior remains in this
Task-005 allocation. Evaluation, rewrite, and target semantics remain with their
declared owners and are not duplicated here.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4601 | low | No open parser-owned gap remains after exact v3 replay and v1/v2 refusal controls. | `tests/past_history_corpus.rs`; TC-056 |
