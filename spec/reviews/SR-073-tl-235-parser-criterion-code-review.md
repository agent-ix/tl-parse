---
id: SR-073
title: TL-235 parser Criterion code and Rust review
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-parse@5942996b48d63a395fe027b550a4a829baa26b15; Cargo.toml, Cargo.lock, benches/README.md, benches/parser_roundtrip.rs, benches/inputs/SHA256SUMS, benches/inputs/{bounded-small,infinite-fairness,infinite-small,past-small,shared-median,shared-near-node-cap}.txt, spec/assurance/AP-001.md, spec/requirements/NFR-004-infinite-trace-parser-determinism.md, spec/infinite-trace-test-matrix.md"
review_set: subset
---

## Summary

Ticket: TL-235. Reviewed PR #54's current-main port of the parser Criterion producer using code-review's Rust lane. The six committed inputs match their SHA-256 manifest and all ten cases run through public 0.3 parser and formatter APIs.

## Verdict

CONDITIONAL: the producer measures parse plus canonical format, so its parser-named cases cannot independently attribute parser cost or scaling. Add a separate parse-only case series or label these results as round-trip measures and supply parser timing elsewhere before calling the ticket's parsing benchmark delivered. This focused PR does not establish TL-235 campaign acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Every parser-named Criterion sample times canonical formatting after parsing; no independent parse-only series can reveal a parser regression masked by a formatter improvement. | benches/parser_roundtrip.rs:46; benches/parser_roundtrip.rs:113; benches/README.md:3 |

## Review Evidence

`bounded`, `past`, and `infinite` each call their parse API, then the matching format API before returning only formatted length. The sole `bench_function` dispatches those combined functions. TL-235 asks for parsing formulas of increasing size among four hot paths. The ten cases offer useful end-to-end latency and exercise small, median, and near-cap inputs, but cannot isolate the parser contribution. Criterion's throughput label uses source bytes correctly for the combined operation.

The new harness owns its six local benchmark fixtures, uses `include_str!` for those local files only, hashes all six before timing, and does not copy a sibling repository's source. The lock retains `tl-parse` and `tl-syntax` 0.3.0 and pins Criterion 0.5.1. No production parser path, safety lint, CI workflow, requirement, or matrix artifact changed. The `cargo bench --locked --bench parser_roundtrip -- --test` smoke passed all ten cases; `cargo fmt --check`, locked all-target/all-feature Clippy with `-D warnings`, `git diff --check`, and independent SHA-256 recomputation passed. Full `make ci` is not established by this review: the PR reports its fuzz-smoke nested cargo-fuzz selected stable rustc for `-Zsanitizer`. The PR also reports Quoin workflow review could not start because `@agent-ix/ix-spec-workflows` was unavailable.

## Assurance Context

AP-001 applies to an exact source, dialect, dependency, corpus, and toolchain candidate. This review evaluated base `183fa59150d3a8fa76a5129242fe50453983dae7` and head `5942996b48d63a395fe027b550a4a829baa26b15` with the changed manifest, lock, benchmark and six fixtures. The producer uses public parsing APIs and validates fixture bytes, but no retained paired measurement, independent campaign replay, full aggregate gate, architecture decision, exception, or release-owner approval was supplied for this head. Its measurements alone cannot establish semantic correctness, resource safety, or release acceptance.
