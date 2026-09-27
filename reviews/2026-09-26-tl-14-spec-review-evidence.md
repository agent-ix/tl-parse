---
id: SR-072
title: TL-14 requirement evidence review
type: SpecReview
analysis: evidence
scope: "agent-ix/tl-parse@e7fe85f506ccfad2eb7835a7a8485505ff04619d; spec/requirements/FR-015-infinite-trace-text-dialect.md, FR-016-infinite-trace-loci-and-diagnostics.md, FR-017-infinite-trace-format-and-corpus.md, NFR-004-infinite-trace-parser-determinism.md, spec/infinite-trace-test-matrix.md, tests/{infinite_v4,infinite_trace_corpus,owner_infinite_corpus}.rs, fuzz/fuzz_targets/{clean_ascii_v3,unbounded_parse_roundtrip}.rs"
review_set: subset
---

## Summary

Compared the declared Test/Fuzz verification methods and TM-002 coverage with executable tests and the two named fuzz targets. The verification family is suitable, but TC-060 does not measure legacy bytes and TC-063 cannot prove a universal formatter claim that conflicts with the stated typed refusal.

## Verdict

FAIL for evidence alignment until FR-015-AC-3 receives exact legacy byte comparison and FR-017-AC-1 is reconciled with valid unrepresentable graphs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | TC-060's old-dialect checks assert only rejection, not unchanged document, report or canonical bytes. | tests/infinite_v4.rs:370; FR-015-AC-3; TM-002 | correct-requirement-no-evidence |
| FND-002 | medium | TC-063 tests formatter refusal for valid owner DAGs while FR-017-AC-1 universally requires canonical text for every admitted graph. | tests/infinite_v4.rs:715; FR-017-AC-1; TM-002 | wrong-requirement |

## Coverage

TC-058/059/060 and TC-061/062/063/064/065/066 all carry real tracking tags in executing test or fuzz target code. The two targets call the v3 and v4 parsers, bound input and parser work, and have checked seed files. Scoped Rust tests at the reviewed head passed 69/69; this is evidence for those assertions only. No aggregate, campaign or release-assurance verification was run.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |
| FND-002 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |

Round 1 reviewed `26ada2912baa10d4060143d950545bfbbec1d6b9`. FND-001 after excerpt: `tests/infinite_v4.rs:470-478`: `assert_eq!(normalized, expected_report, "{dialect} report for {source:?}")` and the corresponding canonical text byte assertion.

FND-002 after excerpt: `TM-002 TC-063`: `Prove representable fixed points, exact fairness order and typed refusal of unrepresentable owner graphs`.

Round 1 verdict: PASS for this method at `26ada2912baa10d4060143d950545bfbbec1d6b9`; no substantive finding remains open.
