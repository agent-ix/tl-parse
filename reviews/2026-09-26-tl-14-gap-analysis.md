---
id: SR-067
title: TL-14 acceptance and test gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-parse@e7fe85f506ccfad2eb7835a7a8485505ff04619d; spec/requirements/FR-015-infinite-trace-text-dialect.md, FR-016-infinite-trace-loci-and-diagnostics.md, FR-017-infinite-trace-format-and-corpus.md, NFR-004-infinite-trace-parser-determinism.md, spec/infinite-trace-test-matrix.md, src/infinite.rs, src/lexer.rs, src/dialect/{mod,v1,v3,v4}.rs, tests/{infinite_v4,infinite_trace_corpus,owner_infinite_corpus,versioned_dialects,clean_ascii_v3,format,parser}.rs, fuzz/fuzz_targets/{clean_ascii_v3,unbounded_parse_roundtrip}.rs and checked seeds"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-parse/TM-002
    type: references
---

## Summary

Manually mapped the nine TL-14 acceptance criteria and TM-002's TC-058 through TC-066 to executable assertions and production paths. Every TC tag exists and the seven named test binaries pass, but the legacy-byte part of FR-015-AC-3 has no assertion and the malformed/zero-diagnostic FR-341 mapping is asserted contrary to FR-016.

## Verdict

FAIL: current tests do not establish the complete promised legacy behavior or the typed syntax refusal mapping. There is no `plan/` bundle in this branch, so plan-task completion was not asserted.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | TC-060 checks that old dialects refuse v4 syntax but never compares their existing report or canonical bytes with the base. | tests/infinite_v4.rs:370; FR-015-AC-3; TM-002 | correct-requirement-no-evidence |
| FND-002 | high | The zero-diagnostic malformed syntax test expects ResourceIncomplete, contradicting the FR-016 Unsupported mapping. | tests/infinite_v4.rs:238; FR-016-AC-2; TM-002 | implementation-bug-despite-evidence |

## Coverage

TC-058 covers future/past primitive admission and lowered W/M; TC-059 covers same-graph ordered fairness, duplicates and empty envelope; TC-060 covers profile/clock cross-refusal but not legacy bytes; TC-061 covers interval and premise spans plus malformed UTF-8 prefix loci; TC-062 covers typed diagnostics and strict report read but codifies the wrong zero-budget disposition; TC-063 covers fixed points, owner identities, derived forms and unrepresentable graph refusals; TC-064 checks corpus IDs, human reasons, digests, canonical text and exact spans; TC-065's two fuzz targets call real parsers and have checked seeds, with 64-run evidence in the PR description; TC-066 covers repeated report bytes and exact/one-under parse and format limits. `tests/owner_infinite_corpus.rs` reads the pinned owner's corpus through its dependency, not a local copy. Reverse code-to-spec tracing found v4 parser, formatter, diagnostic, corpus and fuzz paths owned by these FR/NFRs. The new `dialect_v4_document_digest` API in `src/lib.rs` is supported by the dialect document and is not a duplicate of the owner graph contract.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |
| FND-002 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |

Round 1 reviewed `26ada2912baa10d4060143d950545bfbbec1d6b9`. FND-001 after excerpt: `tests/infinite_v4.rs:470-478`: `assert_eq!(normalized, expected_report, "{dialect} report for {source:?}")` and `assert_eq!(canonical.as_deref().unwrap_or("<none>"), expected_text, "{dialect} canonical text for {source:?}")`; 21 fixture rows cover all three legacy dialects.

FND-002 after excerpt: `tests/infinite_v4.rs:260`: `assert_eq!(truncated.disposition(), InfiniteDisposition::Unsupported);` with following typed `UnexpectedToken` and exact `1..1` span assertions.

Round 1 verdict: PASS for this method at `26ada2912baa10d4060143d950545bfbbec1d6b9`; no substantive finding remains open.
