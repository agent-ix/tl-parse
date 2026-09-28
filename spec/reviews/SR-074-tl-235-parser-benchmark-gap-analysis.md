---
id: SR-074
title: TL-235 parser benchmark acceptance gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-parse@5942996b48d63a395fe027b550a4a829baa26b15; TL-235, Cargo.toml, Cargo.lock, benches/README.md, benches/parser_roundtrip.rs, benches/inputs/SHA256SUMS, benches/inputs/{bounded-small,infinite-fairness,infinite-small,past-small,shared-median,shared-near-node-cap}.txt, spec/assurance/AP-001.md, spec/requirements/NFR-004-infinite-trace-parser-determinism.md, spec/infinite-trace-test-matrix.md"
review_set: subset
---

## Summary

Ticket: TL-235. Mapped this focused parser port to the ticket's parser measurement request and wider campaign exit. No plan bundle or benchmark-specific local Test Matrix row is present; the existing NFR-004/TC-066 checks deterministic bounds, not runtime performance.

## Verdict

CONDITIONAL for this parser port. Its ten runnable combined parse-and-format cases do not alone close the parser-only measurement requirement. TL-235's cross-crate, retained campaign and comparison exit remains open independently of this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The only new parser timing path includes formatting, so the ticket's parse hot-path scaling has no isolated measurement in this PR. | benches/parser_roundtrip.rs:46; benches/parser_roundtrip.rs:113; TL-235 |

## Coverage

The six inputs cover bounded, past, infinite, fairness, shared median, and near-node-cap shapes. Manifest digests match all six bytes; the smoke executes all ten declared cases. `quire coverage --scope . --json` reports 124/128 existing matrix rows backed, with four existing gaps unrelated to this benchmark-only diff. No `spec/**` or `plan/**` file changes in this PR, and no plan bundle targets this focused port. The broader ticket additionally requires rewrite, evaluation, horizon, separate trace-length and interval-width scaling, per-run retention, prior-run comparison, and regression classification; none is claimed complete from this parser-only head. Optional semantic review was not run.
