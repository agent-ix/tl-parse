---
id: SR-004
title: Quality assurance and test coverage assessment
type: SpecReview
analysis: gap-analysis
scope: Makefile, scripts/**/*, src/**/*.rs, tests/**/*.rs, fuzz/**/*, evidence/**/*, spec/**/*
review_set: all
---

# Quality assurance and test coverage assessment

## Summary

The candidate is broadly and independently exercised but is not claimed to be
fully tested. The complete local gate, exact-candidate evidence collection,
post-seal evidence verification, policy mutation tests, schemas, MSRV, clippy,
rustdoc, dependency policy, corpus checks, and bounded fuzz smoke all pass.
Hosted CI remains manual-only and was not dispatched.

## Coverage Measurement

`cargo llvm-cov --all-targets --all-features --summary-only` measured the
following on 2026-08-31:

| Component | Line coverage | Function coverage |
|---|---:|---:|
| CLI binary | 82.69% | 50.00% |
| Diagnostics | 92.13% | 100.00% |
| Formatter | 83.91% | 83.33% |
| Lexer | 93.01% | 94.12% |
| Library facade | 100.00% | 100.00% |
| Parser | 87.96% | 100.00% |
| **Total** | **88.20%** | **84.62%** |

Coverage is supporting evidence, not a release threshold. The missing-line
report includes reachable CLI I/O and malformed-interval recovery paths as
well as defensive invariant branches that safe constructors may make
unreachable. Issue #8 requires tests for the reachable paths and an explicit
reachability disposition for the remainder.

## Qualification Coverage

- The Rust suite executes 25 passing tests across eight result groups with no
  ignored tests.
- Five Python policy suites exercise retained history, evidence outcomes,
  failure propagation, Draft 7 format checks, and strict traceability.
- Strict traceability reports 62/62 backed rows.
- The source-locked evidence record has 14 passing collection and post-seal
  outcomes, including both pinned external PGM-01 validations.
- Final QA found and fixed clean-environment tool lookup, cargo target
  placement, MSRV positive-output classification, and the transcript-runner
  single-file delegation gap.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-401 | medium | Disposable-worktree failure injection is still needed for collector cleanup and semantic output contracts. | Issue #7 |
| FND-402 | medium | Reachable CLI/error paths need additional coverage, and remaining defensive branches need an explicit reachability classification. | Issue #8 |
| FND-403 | low | A separately reviewed portable tool-lock profile is required before hosted CI returns; this does not authorize CI triggers or dispatch. | Issue #9 |
| FND-404 | medium | Upstream `tl-syntax` PR #6 must merge before the temporary git-source exception can be removed and release evidence regenerated. | `tl-syntax` PR #6 |

None of these items grants an automated release decision. The assurance claim
and human source-release decision remain open.
