---
id: SR-043
title: "Rust review — tl-parse property and fuzz baseline"
type: SpecReview
analysis: code-review
scope: "tests/property.rs, tests/clean_ascii_v2.rs, tests/corpus.rs, fuzz/fuzz_targets, and scripts/run_fuzz_smoke.sh at 9ca856b"
review_set: subset
---

## Summary

The existing property and fuzz implementation was reviewed under
`agent-skills/rust-review/SKILL.md` at `9ca856b`. This branch changes no Rust,
dependency, public API, fuzz target, runner, workflow, or unsafe surface. The
three generated property functions and both cargo-fuzz targets remain
load-bearing: they compare graph structure, direct lowering, canonical fixed
points, typed bounded outcomes and strict serialized reports rather than merely
asserting success.

Both fuzz crates forbid unsafe code. Their panics and `expect` calls are fuzz
oracles, not production panic paths; successful parse/format preconditions are
checked immediately before dependent unwraps. Collections and work are bounded
by the parser/formatter limits, proptest recursive sizes, and the 64-execution
smoke cap. There is no async runtime, lock, blocking bridge, persistence integer
conversion, foreign ABI, or production filesystem change in scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4301 | low | No Rust defect was found in the reviewed property/fuzz scope. Test-only panics are the failure oracle, unsafe remains forbidden, generator sizes are bounded, and the exact existing test and fuzz lanes pass. | `tests/property.rs`, `tests/clean_ascii_v2.rs`, `fuzz/fuzz_targets/parser.rs`, `fuzz/fuzz_targets/clean_ascii_v2.rs` |
| FND-4302 | medium | The runner's process output is not a declared structured domain report and Quoin has no libFuzzer adapter. Treating exit status or scraped stdout as a retained proof would violate the shared evidence boundary; the run can be review-recorded but not truthfully entered as normalized retained evidence yet. | SR-042 FND-4203, tl-syntax#26, engineering-assurance#7 |

## Verification

- `cargo test --locked --test property --test clean_ascii_v2`: 9 passed.
- `make fuzz-smoke` outside seccomp: both 64-run targets passed with
  LeakSanitizer enabled and no crash artifact.
- rustc `1.97.0-nightly (e22c616e4 2026-04-19)`; cargo-fuzz 0.13.2.
- Hosted CI was not dispatched.
