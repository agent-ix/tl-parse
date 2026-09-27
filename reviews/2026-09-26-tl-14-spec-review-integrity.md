---
id: SR-069
title: TL-14 specification integrity review
type: SpecReview
analysis: integrity
scope: "agent-ix/tl-parse@e7fe85f506ccfad2eb7835a7a8485505ff04619d; spec/requirements/FR-015-infinite-trace-text-dialect.md, FR-016-infinite-trace-loci-and-diagnostics.md, FR-017-infinite-trace-format-and-corpus.md, NFR-004-infinite-trace-parser-determinism.md, spec/infinite-trace-test-matrix.md, docs/DIALECT-004-clean-ascii-v4.md"
review_set: subset
---

## Summary

Checked TL-14's new requirement statements for internal consistency, atomicity, testability and ownership of parser versus tl-syntax behavior. The formatter behavior and acceptance criterion give incompatible answers for a valid shared owner graph.

## Verdict

FAIL: FR-017-AC-1 is contradictory as written. Dependency and profile boundaries are otherwise explicit in this selected scope.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | An admitted graph may be valid in tl-syntax yet unrepresentable in v4 text, but FR-017-AC-1 requires every admitted graph to round-trip as text. | spec/requirements/FR-017-infinite-trace-format-and-corpus.md:40; FR-017-AC-1 | wrong-requirement |

## Coverage

FR-015/016/017 and NFR-004 each have concrete verification methods and TM-002 trace. The selected requirement documents separate parser syntax, diagnostics, formatting and resource determinism from owner graph identity and fairness; no cycle exists among their changed relationships. The behavior's typed `unrepresentable_graph` is appropriate, but it must be reflected in the criterion so a valid graph is not simultaneously required to format and required to refuse. No new external CLI, paginated API, authenticated API, or generation mode assumption applies.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |

Round 1 reviewed `26ada2912baa10d4060143d950545bfbbec1d6b9`. FND-001 after excerpt: `FR-017:18-21`: `emit canonical text for graphs representable in the v4 dialect ... or return a typed `unrepresentable_graph` refusal.`

Round 1 verdict: PASS for this method at `26ada2912baa10d4060143d950545bfbbec1d6b9`; no substantive finding remains open.
