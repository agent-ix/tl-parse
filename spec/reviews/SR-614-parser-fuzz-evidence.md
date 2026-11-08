---
id: SR-614
title: "Evidence record - tl-parse fuzz runs and v4 locus corpus (TL-220, TL-29)"
type: SpecReview
analysis: evidence
scope: "fuzz/, parser, clean_ascii_v2, clean_ascii_v3 and unbounded_parse_roundtrip targets, corpus/infinite-loci, FR-017-AC-2, TC-064"
review_set: subset
---

## Summary

Recorded libFuzzer runs of the targets listed below, three RNG seeds of 60 seconds
each per target (7,259,189 executions in total). No run produced a crash, timeout, out-of-memory or sanitizer finding.
These are bounded observations of finite runs: a run without a crash and a coverage
curve that had not flattened are not evidence of correctness (FR-021-AC-5).

## Verdict

**PASS** for the recorded evidence: every run completed and the v4 locus corpus passes. No fuzz target or gate was added in this repository; `make fuzz-build` and `make fuzz-smoke` are unchanged.
A target is credited only by recorded runs, not by existing.

## Run record

Source revision d81be56cc5d1 (campaign tool state clean for `src/`, `Cargo.toml`, `Cargo.lock`);
tools: rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-fuzz 0.13.1, host aarch64-apple-darwin; sanitizer: address (cargo-fuzz default); libFuzzer flags
`-max_total_time=60 -max_len=1024 -rss_limit_mb=8192`; seeds 101, 102, 103 (distinct, run in that order on one
growing corpus per target, or on a throwaway copy of the checked-in corpus for
targets whose checked-in seeds are tested one-by-one). Time is libFuzzer's
reported run time and excludes build time. Machine-readable record:
`fuzz/evidence/2026-11-08/campaign.json`.

| Target | Seed | Seconds | Executions | Coverage edges | Features | Corpus entries | Findings |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| parser | 101 | 61 | 399,683 | 1,887 | 7,311 | 659 | none |
| parser | 102 | 61 | 289,770 | 2,047 | 7,862 | 616 | none |
| parser | 103 | 61 | 666,755 | 2,115 | 8,769 | 735 | none |
| clean_ascii_v2 | 101 | 61 | 361,493 | 2,372 | 10,677 | 767 | none |
| clean_ascii_v2 | 102 | 61 | 232,404 | 2,390 | 11,369 | 735 | none |
| clean_ascii_v2 | 103 | 61 | 174,085 | 2,392 | 11,937 | 784 | none |
| clean_ascii_v3 | 101 | 61 | 690,941 | 1,594 | 6,327 | 840 | none |
| clean_ascii_v3 | 102 | 61 | 692,922 | 1,737 | 6,854 | 764 | none |
| clean_ascii_v3 | 103 | 61 | 776,934 | 1,738 | 7,597 | 815 | none |
| unbounded_parse_roundtrip | 101 | 61 | 1,120,945 | 2,516 | 8,539 | 967 | none |
| unbounded_parse_roundtrip | 102 | 61 | 901,377 | 2,619 | 9,865 | 978 | none |
| unbounded_parse_roundtrip | 103 | 61 | 951,880 | 2,670 | 10,422 | 1,020 | none |

## Seeded-fault check

A target is credited only if it fails on a deliberately introduced fault. Each
fault was applied temporarily to the library source (and reverted), and the
target was run from an empty or seed-only corpus.

| Target | Fault | Result |
| --- | --- | --- |
| infinite-loci test (`tests/infinite_loci.rs`) | interval locus end shortened by one byte in `src/infinite.rs` (`interval_spans` push) | reviewer-run: the test fails on the first mismatched span; reverted. No libFuzzer target was changed, so no seeded fault was run against the four fuzz targets here |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-61401 | low | No crash, timeout or out-of-memory result in any recorded run of the four existing parser targets, including the `unbounded_parse_roundtrip` fixed-point target, on throwaway copies of their checked seeds. Coverage was still growing in most runs, so no plateau is claimed. | campaign.json |
| FND-61402 | medium | The source-locus corpus covered one unbounded form (`F`, `G`) and one fairness form. `corpus/infinite-loci/cases.json` adds all eight unbounded-capable operators, a u32-maximum bound, mixed closed and unbounded intervals, an empty envelope, a trailing-semicolon premise, two premises, and eight refusals with typed code and exact byte locus; spans were counted independently of the parser. `tests/infinite_loci.rs` checks them and that canonical text re-parses to itself with unchanged interval and premise text. | corpus/infinite-loci, TC-064, FR-017-AC-2 |
| FND-61403 | low | On macOS libFuzzer counts the address sanitizer's freed memory toward RSS, so the default 2048 MB limit is too low; recorded runs pass `-rss_limit_mb=8192`. Not a target defect. | fuzz/README |

## Limitations

These runs are three 60-second repetitions through plain `cargo fuzz run`, not the
`run_v4_campaign.py` runner of the 2026-09-23 V4 record, and carry no feature-identity
snapshots, so FR-021-style campaign claims are not made. No new fuzz target was
added in this repository, so no new seeded-fault check was run here.

