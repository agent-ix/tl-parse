---
id: SR-231
title: "spec-correctness - cargo-mutants configuration, kill-rate thresholds and coverage reporting (tl-parse, TL-231, TL-234)"
type: SpecReview
analysis: spec-correctness
scope: "mutation configuration, thresholds, scripts and Makefile verification targets of tl-parse"
review_set: all
---

## Summary

TL-231 and TL-234 add a reproducible cargo-mutants configuration, a kill-rate gate, a diff gate and llvm-cov
line and branch coverage reporting to tl-parse. TL-371 (2026-11-09) replaces the sampled floors with the first full
run, adds the per-survivor reason check and puts `make mutants-diff` in `make ci`. This record is the dated
evidence; `docs/verification-tooling.md` carries no figures. The core group (`src/parser.rs`, `src/lexer.rs`) is at
97.21% and the crate at 99.02%; floors are 97.0 and 98.8.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | First full `make mutants` (cargo-mutants 27.0.0, `--no-shuffle`, 922 mutants, 2026-11-09): the 60 s test-timeout floor counted 277 timeouts that were load, not hangs (host load average 13 to 19), so the run was repeated for every missed and timed-out mutant with a 300 s floor (`--iterate`, 317 mutants). Merged result before any new test: core 87.95% (197 caught, 21 missed, 6 timeout, 16 unviable), crate 81.97% (691 caught, 144 missed, 8 timeout, 79 unviable). Wall time: 10161 s for the first run and 3361 s for the repeat. | `mutation-thresholds.toml` |
| FND-2312 | low | `tests/mutation_survivors.rs` adds behavior tests for the strict scanner (escape widths, depth and byte boundaries, open strings), the dialect keyword boundary, recovery actions, exact work and node accounting for every dialect, the v2/v3/v4 report readers, the v4 formatter (derived operators, depth along every operator shape, limits) and the CLI file limit. Every survivor was retested with the full suite after the tests landed: 152 mutants (55 min) and then 65 (48 min). | `tests/mutation_survivors.rs` |
| FND-2313 | low | Result after the new tests, with equivalent mutants excluded as `.cargo/mutants.toml` now does: core 97.21% (209 caught, 0 missed, 6 timeout, 16 unviable), crate 99.02% (807 caught, 0 missed, 8 timeout, 79 unviable). Floors are the measured rates truncated: core 97.0, crate 98.8. 28 mutants are excluded as equivalent or unreachable, each with a reason in `.cargo/mutants.toml`. | `mutation-thresholds.toml`, `.cargo/mutants.toml` |
| FND-2314 | low | Every gate was shown able to fail: `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate, on an empty population or a failed baseline, and exits 2 on unreadable input; `make coverage COVERAGE_MIN_LINES=99` exits non-zero. `scripts/run_mutants.py` accepts cargo-mutants exit 0, 2 and 3 so the gate always prints its table. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage re-measured from a clean `target/llvm-cov-target` on 2026-11-09, all features, `ci_guard` excluded. Stable `make coverage`: 87.78% lines / 86.43% regions / 84.42% functions. Nightly `make coverage-branch`: 83.27% lines / 58.99% regions / 83.90% functions / 86.27% branches. The two toolchains report different line totals and the nightly region figure is much lower, so `COVERAGE_MIN_LINES` is 80, below the lower line figure; both targets pass at that floor. A reviewer measured lower totals on a fresh clone than these; the difference could not be reproduced here, so the floor sits below both readings that were available. | `Makefile` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro), heavily loaded (load average 35 to 45). Toolchain: rustc 1.98.1,
nightly rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3
(engine 0.52.1). Mutation order is fixed by `--no-shuffle`; there is no seed in this mode. The per-mutant timeout
is the cargo-mutants automatic value (three times the unmutated baseline, minimum 300 s in this crate's
`.cargo/mutants.toml`, raised from 60 s in TL-371). A mutant is timed out when its test run exceeds the larger of 300 s and three times the baseline; the floor cannot tell a hang from a run that a loaded host slowed past it, so a timeout is the detection of a hang only on an unloaded machine, and timeouts are counted as unkilled either way. Wall times are not equality inputs. The 2026-11-09 rates are a full run plus retests of every survivor; a mutant a new test kills stays killed, so the merge is the rate a fresh full run would report.

## Survivors

Eight mutants are neither killed nor equivalent. Each makes the suite hang, so the timeout is the detection and no
assertion can run; they are listed with that reason under `[[survivor]]` in `mutation-thresholds.toml`:
`scan_token`, `scan_identifier` and `scan_number` offsets that never advance (`+=` to `*=`), `parse_dialect`
without its stop guard, `Parser::advance` replaced by `()`, `Parser::charge_work` returning `false`, and
`scan_json_string` returning a constant index. They count against the rates above. The Campaign runner cannot contain `make mutants` (TL-28,
2026-09-24); `make mutants` is the supported path.
