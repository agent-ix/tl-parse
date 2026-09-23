---
id: SR-065
title: "code-review review of TL-195/196/197 followups (PR #47)"
type: SpecReview
analysis: code-review
scope: "Makefile, examples/fuzz_campaign.rs, fuzz/lsan_suppressions.txt, spec/test-matrix.md"
review_set: subset
---

## Summary

Reviewed `git diff main...HEAD` for PR #47 (branch
`peter/tl-195-196-197-followups`, commits f060c7e/b4d21b9): a TestMatrix
header fix (TL-196), a no-code-change investigation (TL-197), and a new
LeakSanitizer suppression file wired into the fuzz lane (TL-195). All claims
in the PR description were independently reproduced rather than taken on
trust. No vendoring, no duplicated helpers, no stubs. `cargo test --release
--example fuzz_campaign`, `make fmt-check`, `make lint`, `make fuzz-smoke`,
and `make spec` all pass on this branch.

## Verdict

**PASS** — all three sub-changes verified independently; no high or medium
findings.

## Assurance Context

`spec/assurance/AP-001.md` (tl-parse v0.1 text-boundary assurance profile)
applies: its `impact-hostile-growth` scenario names fuzz seeds as a
detect-before-harm control, and TL-195 changes the fuzz lane's sanitizer
configuration directly. Evaluated at HEAD (b4d21b9) against baseline main
(7b8c755, the merge-base for this PR).

- Architecture/impact context examined: the `impact-hostile-growth` scenario
  (malformed input panics or exceeds a declared resource bound) is what
  `make fuzz-smoke` exists to detect; this review re-ran that gate rather than
  trusting its prior result, and separately reproduced the raw (unsuppressed)
  LeakSanitizer report to confirm the suppression's scope by direct
  observation, not by reading its comments.
- Measurement: `cargo test --release --example fuzz_campaign` (8/8 pass),
  `make fuzz-smoke` (both fuzz targets exit 0, `domainOutcome: pass`,
  suppression counts exactly 1×8B + 1×48B per target run, matching the
  documented leak), `make lint`, `make fmt-check`, `make spec` (`quire
  validate --strict` and `quire coverage --strict` both exit 0).
- Exceptions: none active or newly introduced by this diff.
- Unavailable context: none — AP-001's stated evidence surfaces (exact
  grammar tests, corpus, fuzz seeds, budget-boundary tests) were all
  reachable and re-run in this review.
- Independence: `AP-001` reserves source-release approval to `@kreneskyp`;
  this review does not substitute for that approval and makes no release
  determination.

## Findings

| ID      | Severity | Summary                                                                                                                                                                                                 | Refs |
| ------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---- |
| FND-001 | low | TL-195 suppression verified narrow by direct experiment, not by reading its comments: running the compiled `parser` fuzz binary with `ASAN_OPTIONS=detect_leaks=1` and no suppression file reproduces the exact claimed leak — 8B direct + 48B indirect, both rooted at `main → fuzzer::FuzzerDriver → std::__1::thread::thread<...>` / `std::__1::__thread_struct::__thread_struct()`, i.e. libFuzzer's own internal watchdog-thread startup, before any corpus byte reaches `LLVMFuzzerTestOneInput`. `grep -rn 'std::thread\|thread::spawn' src fuzz/fuzz_targets` is empty, and Rust's `std::thread::spawn` does not go through the C++ `std::__1::thread` class this pattern matches, so a genuine leak in tl-parse's own parser/lexer/formatter — which would be rooted in `LLVMFuzzerTestOneInput` via Rust allocator frames — cannot share a stack frame with either suppression pattern under the current single-threaded fuzz-target architecture. Both `make fuzz-smoke` runs (parser, clean_ascii_v2) suppress exactly one occurrence of each pattern per run, consistent with a fixed, deterministic startup allocation rather than a growing/broad match. The ambient-override guard was independently verified at both layers: `LSAN_OPTIONS=x make help` fails at Makefile parse time (`$(error ...)`, exit 2) before any target runs, and the compiled `fuzz_campaign` example refuses an externally-set `LSAN_OPTIONS` (and, unchanged, `ASAN_OPTIONS`) at runtime, reporting `refused_ambient_override`/`unavailable` rather than silently proceeding. No masking scenario could be constructed. | `fuzz/lsan_suppressions.txt`, `examples/fuzz_campaign.rs:1019-1025,1214-1224`, `Makefile:57-62` |
| FND-002 | low | TL-197's "no fix needed" conclusion independently reproduced rather than trusted: `quire coverage --scope . --strict` exits 0 on both `main` (7b8c755) and this branch's HEAD. `quire coverage --help` documents `--strict` as "Exit 1 when any row is unbacked or any status is contradicted" — i.e. only the `unbacked-row`/`status-lie` checks, not `untracked-symbol` — confirming the FR-013-AC-2/TC-055/TC-056/TC-057 `untracked-symbol` lines this run reports do not gate. This matches SR-043 FND-4303, which already accepted these tags as legitimately owned by upstream tl-syntax (Task-002) and not to be duplicated locally. No code change was warranted, and none was made. | `spec/reviews/SR-043-clean-ascii-v3-gap-analysis.md:34`, quire 0.31.0 `coverage --help` |
| FND-003 | low | TL-196's two header renames fix a real, currently-red gate: `quire validate --scope . "spec/test-matrix.md"` exits 1 on `main` today (`table columns [...,"Status"] do not match asserted columns [...,"Coverage Status"]`, TestMatrix archetype structural validation) and exits 0 on this branch. As a known, already-disclosed side effect (this exact header has oscillated between "Status" and "Coverage Status" across at least four prior commits touching this file, and is documented as a program-wide, not-locally-repairable conflict between the TestMatrix archetype and the traceability module's configured status column — SR-003 FND-301, SR-007 FND-703, SR-032 FND-3204, tracked as `agent-ix/quire-contract-ir#21`), `quire coverage --strict` now emits a `status-column-matches-nothing` note for `spec/test-matrix.md` that was absent on `main`. This does not affect the gate outcome (FND-002: `--strict` exits 0 either way) and is not a new problem this PR introduces carelessly — it is the documented, accepted trade-off of fixing structural validation, consistent with the two prior sibling fixes cited in the PR description (tl-mltl TL-198, tl-syntax TL-203). No action needed beyond what the PR already states. | `spec/test-matrix.md:14,28`, `spec/reviews/SR-032-issue-27-code-review.md:53` |
