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
| `make mutants-diff` | A gate in `make ci`. cargo-mutants on `git diff $(MUTANTS_BASE)...HEAD` (default `origin/main`) only; every mutant in the diff must be killed. Only committed changes are mutated; uncommitted edits are not. |
| `make coverage` | `cargo llvm-cov` line, function and region coverage on the pinned toolchain, failing below `COVERAGE_MIN_LINES`. |
| `make coverage-branch` | The same on nightly, adding branch coverage. |


`make mutants-diff` is part of `make ci`, so a change cannot add logic its tests would not notice; `make mutants` and the coverage targets are not.

## Mutation gate

Kill rate is `caught / (caught + missed + timeout)`: a timeout counts as not killed, and unviable mutants
count on neither side. Each group in `mutation-thresholds.toml` has a floor; the gate prints a table and
exits 1 when a group is below its floor, when a group has no viable mutants, or when the unmutated
baseline failed. The campaign targets are a core group and a whole-crate group (the core is `src/parser.rs` and `src/lexer.rs`).

Excluded from every threshold: nothing (the crate has no CI-plumbing module). Equivalent mutants are listed in
`exclude_re` of `.cargo/mutants.toml`, each with a one-line reason, so they are never generated.

Every missed or timed-out mutant must also appear in a `[[survivor]]` table of `mutation-thresholds.toml` with a
one-line reason (FR-022-AC-5). A survivor that is not listed fails the gate, however high the kill rate. A listing
names the file and the mutant without its `file:line:col:` prefix, so it survives edits that move code. The
survivors listed today are non-terminating mutants: the suite hangs, the timeout is the detection, and no
assertion can run.

`minimum_test_timeout` in `.cargo/mutants.toml` is well above a loaded run, so a timeout means a hang and not a
busy machine.

`scripts/run_mutants.py` accepts cargo-mutants exit statuses 0, 2 (survivors) and 3 (timeouts) so that
the gate always runs and prints its table; any other status is passed on.
`scripts/mutation_gate.py` and `scripts/run_mutants.py` are the same in tl-syntax, tl-parse, tl-rewrite and
tl-mltl (tl-mltl alone has `--source-env`); a shared copy is a separate follow-up.

## Limits of the recorded measurements

The floors are the rates of the first full `make mutants` run (SR-231), and they only ratchet up.
The Campaign runner cannot contain `make mutants` (TL-28, 2026-09-24): cargo-mutants starts its own Cargo child
in a new process group, and the runner refuses a descendant whose process group differs from its producer. That
containment is unresolved, so `make mutants` is the supported way to run the mutation gate.

## Coverage

Stable and nightly toolchains report different line totals, so `COVERAGE_MIN_LINES` is set below the lower of
the two. It is a floor, not a target. Coverage reports exclude CI-plumbing code.
