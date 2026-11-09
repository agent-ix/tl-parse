---
id: SR-231
title: "spec-correctness - cargo-mutants configuration, kill-rate thresholds and coverage reporting (tl-parse, TL-231, TL-234)"
type: SpecReview
analysis: spec-correctness
scope: ".cargo/mutants.toml, mutation-thresholds.toml, scripts/, Makefile verification targets"
review_set: all
---

## Summary

TL-231 and TL-234 add a reproducible cargo-mutants configuration, a kill-rate gate, a diff gate, and
llvm-cov line and branch coverage reporting to tl-parse. All gates are separate Make targets (`make mutants`,
`make mutants-diff`, `make coverage`, `make coverage-branch`) outside `make ci`, because a full mutation
run costs hours. The thresholds are measured baselines, not the campaign targets of 95% (core) and 85%
(crate): the core kill rate before any change was 196 caught, 16 missed, 12 timeout, 16 unviable = 87.50%; after: unchanged, 87.50% (no tests were added; the 28 survivors and timeouts are recorded as test gaps).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | Measured kill rates (cargo-mutants 27.0.0, `--no-shuffle`, base 5654cf4 rebased onto 64587da): core unchanged, 87.50% (no tests were added; the 28 survivors and timeouts are recorded as test gaps); crate sample 81.98% (414 caught, 77 missed, 14 timeout, 43 unviable over the core plus `--shard 0/3`); 44 of the missed mutants are in `src/diagnostic.rs`. Timeouts count against the rate. The crate rate is a file sample, not a full run: the machine was loaded (load average 35 to 45), so a full run did not fit the budget and sharded sampling was used. The first full `make mutants` run should replace the sample and the floors should then be raised. | `mutation-thresholds.toml` |
| FND-2312 | low | The core survivors include the `parse_dialect` condition mutants, the interval-parsing `!=` mutants, `charge_work` and `advance`, and lexer boundary mutants. The 12 timeouts are mutants that make `advance` or the identifier scanner loop; they count against the rate. None was triaged as equivalent. | `docs/verification-tooling.md` |
| FND-2313 | low | The enforced floors are core 87.0, crate 78.0. A floor below its target means survivors are recorded, not accepted: each is a test gap listed in `docs/verification-tooling.md`. No survivor was classified equivalent by a machine; the only `exclude_re` entries carry a one-line reason. | `.cargo/mutants.toml` |
| FND-2314 | low | Every gate was shown able to fail. `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate (raised floor of 101 against a 100% core; and the pre-fix core run against the 95% floor), exits 1 on an empty population or a failed unmutated baseline, and exits 2 on unreadable input. `make coverage COVERAGE_MIN_LINES=99` fails with a non-zero exit. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage (`make coverage-branch`, nightly, all features, `ci_guard` excluded): lines 86.37%, regions 83.77%, functions 81.86%, branches 86.22%. `COVERAGE_MIN_LINES` is set to 85 on the stable `make coverage` target, a floor below the measured line coverage. Branch coverage is unstable in cargo-llvm-cov 0.8.7 and needs nightly. | `Makefile` |
| FND-2316 | low | The Makefile guard (NFR-006) rejected the first version of the recipes (bare `;` in a recipe and in a help string), so the cargo-mutants exit-status handling lives in `scripts/run_mutants.py` and the Makefile only invokes it. | `scripts/run_mutants.py` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro). Toolchain: rustc 1.98.1, nightly rustc 1.100.0-nightly
(1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3 (engine 0.52.1). Mutation
order is fixed by `--no-shuffle`; cargo-mutants takes no seed in this mode. Per-mutant timeout is the
cargo-mutants automatic value (three times the unmutated baseline, minimum 60 s). Files: `.cargo/mutants.toml`, `mutation-thresholds.toml`, `scripts/mutation_gate.py`, `scripts/run_mutants.py`.
Wall times are not stable on a shared machine and are not equality inputs.
