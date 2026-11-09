# Verification tooling

How tl-parse's mutation, coverage evidence is produced and gated (TL-231, TL-234). This page describes the targets and what
the gates enforce; it carries no measured results. Floors live in `mutation-thresholds.toml` and the
`COVERAGE_MIN_LINES` variable of the Makefile, and dated measurements live in `spec/reviews/SR-231-*`
and the Linear tickets.

Tools (inputs): cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7 (branch coverage needs a nightly toolchain with
llvm-tools), Python 3.11 or newer (the gate reads TOML with `tomllib`).

## Targets

| Target | What it does |
| --- | --- |
| `make mutants` | Full cargo-mutants run (`--no-shuffle`, so the population and order are fixed), then `scripts/mutation_gate.py` against `mutation-thresholds.toml`. Takes hours; run it weekly. |
| `make mutants-diff` | cargo-mutants on `git diff $(MUTANTS_BASE)...HEAD` (default `origin/main`) only; every mutant in the diff must be killed. Only committed changes are mutated; uncommitted edits are not. |
| `make coverage` | `cargo llvm-cov` line, function and region coverage on the pinned toolchain, failing below `COVERAGE_MIN_LINES`. |
| `make coverage-branch` | The same on nightly, adding branch coverage. |


None of the mutation or coverage targets is part of `make ci`.

## Mutation gate

Kill rate is `caught / (caught + missed + timeout)`: a timeout counts as not killed, and unviable mutants
count on neither side. Each group in `mutation-thresholds.toml` has a floor; the gate prints a table and
exits 1 when a group is below its floor, when a group has no viable mutants, or when the unmutated
baseline failed. The campaign targets are a core group and a whole-crate group (the core is `src/parser.rs` and `src/lexer.rs`).

Excluded from every threshold: nothing (the crate has no CI-plumbing module). One equivalent mutant is listed in `exclude_re`: `minimum` in `src/diagnostic.rs` returns the same value for `<` and `<=`.

`scripts/run_mutants.py` accepts cargo-mutants exit statuses 0, 2 (survivors) and 3 (timeouts) so that
the gate always runs and prints its table; any other status is passed on.
`scripts/mutation_gate.py` and `scripts/run_mutants.py` are the same in tl-syntax, tl-parse, tl-rewrite and
tl-mltl (tl-mltl alone has `--source-env`); a shared copy is a separate follow-up.

## Limits of the recorded measurements

Crate-wide kill rates recorded so far are core plus one `--shard 0/3` file sample, not a full run, and the
floors are the sampled rate minus a margin. The first full `make mutants` run should replace them, and floors
only ratchet up. tl-parse core is below its mutation target and no tests were added in this change; see SR-231 and TL-371. Open survivors are listed in SR-231.

## Coverage

Stable and nightly toolchains report different line totals, so `COVERAGE_MIN_LINES` is set below the lower of
the two. It is a floor, not a target. Coverage reports exclude CI-plumbing code.
