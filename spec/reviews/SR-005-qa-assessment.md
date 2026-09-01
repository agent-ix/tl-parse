---
id: SR-005
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
| CLI binary | 92.18% | 78.26% |
| Diagnostics | 93.26% | 100.00% |
| Formatter | 83.91% | 83.33% |
| Lexer | 93.01% | 94.12% |
| Library facade | 100.00% | 100.00% |
| Parser | 95.55% | 100.00% |
| **Total** | **91.85%** | **90.36%** |

Coverage is supporting evidence, not a release threshold. The updated run adds
asserted exit/diagnostic coverage for CLI dispatch, default stdin, explicit
profiles, file and stdin read errors, bounded streaming input, write failures,
plain-text error rendering, every malformed-interval parser position, and both
lexer- and parser-originated diagnostic truncation.

### Remaining defensive branches

- Parser discriminant fallbacks after an immediately preceding exhaustive
  operator/prefix match cannot receive another token kind; forcing them would
  require mutating private parser state.
- Parser validation-failure and formatter malformed-graph branches require a
  `FormulaDocument` that contradicts the same pinned constructors used to
  build it. They remain fail-closed defenses against future upstream contract
  drift rather than synthetic unsafe-fixture targets.
- Source-span construction fallbacks cannot fail while source limits remain
  clamped below `u32::MAX` and offsets remain ordered. Node/token index
  fallbacks are likewise guarded by private append/cursor invariants.
- JSON serialization errors require allocation failure for the closed,
  scalar/collection wire model; the process cannot recover reliably enough to
  inject that condition. The error mapping remains fail closed.
- File metadata/read races and real non-broken stdout device failures are
  platform conditions. Deterministic injected reader/writer tests assert the
  same error classes; the broken-pipe exception is exercised separately.
- Test-only panic arms and `Write::flush` on a writer used solely through
  `writeln!` are harness invariants, not product branches.

## Qualification Coverage

- The Rust suite executes 28 passing tests across eight result groups with no
  ignored tests.
- Six Python policy suites exercise retained history, evidence outcomes,
  exact collector faults/cleanup/target placement, failure propagation, Draft
  7 format checks, and strict traceability.
- Strict traceability reports 62/62 backed rows.
- The source-locked evidence record has 14 passing collection and post-seal
  outcomes, including both pinned external PGM-01 validations.
- Final QA found and fixed clean-environment tool lookup, cargo target
  placement, MSRV positive-output classification, and the transcript-runner
  single-file delegation gap.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-401 | medium | Resolved: disposable committed worktrees inject failure before and after staging, exercise exact clean-environment tool lookup, prove cleanup and immutable anchors/assurance, and keep cargo-fuzz output outside the candidate. Every non-silent retained lane has a positive-output mutation, including MSRV. | Issue #7 |
| FND-402 | medium | Resolved: reachable CLI I/O, dispatch, rendering, malformed-interval, and diagnostic-limit paths have asserted outcomes; remaining defensive branches have the reachability disposition above. | Issue #8 |
| FND-403 | low | Resolved for the current boundary: `tools.lock` v2 supports explicit reviewed profiles, exact path/digest re-derivation, declared byte-identical aliases, retained profile identity, and a manual dispatch input. A runner profile must still be committed and reviewed before anyone dispatches hosted CI. | Issue #9 |
| FND-404 | medium | Upstream `tl-syntax` PR #6 must merge before the temporary git-source exception can be removed and release evidence regenerated. | `tl-syntax` PR #6 |

None of these items grants an automated release decision. The assurance claim
and human source-release decision remain open.
