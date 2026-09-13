---
id: SR-044
title: "Code review — shared past/history parser replay"
type: SpecReview
analysis: code-review
scope: "corpus/past-history, tests/past_history_corpus.rs, Makefile"
review_set: subset
---

# Code review — shared past/history parser replay

## Summary

Reviewed the retained corpus, v3 parse/format replay, old-dialect controls, and
digest gates against the Task-005 parser allocation.

## Verdict

**PASS after remediation.** The retained bytes match the owner manifest, every
source parses to the exact span-free formula-v2 graph, canonical formatting
returns the pinned source, and v1/v2 dialects refuse every past source.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4401 | high | Each v3 source now also runs through v1/v2 and must produce no document. | `tests/past_history_corpus.rs` |
| FND-4402 | medium | Span-bearing parser output is compared by semantic view and canonical text independently. | `tests/past_history_corpus.rs` |
| FND-4403 | low | Owner manifest and file digests now participate in the normal corpus gate. | `Makefile`; `corpus/past-history/` |
