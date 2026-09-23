# Parser Criterion lane

`parser_roundtrip` times real parsing followed by canonical formatting through
the public V1 closed-trace, V3 past, and V4 infinite APIs. The seven cases use
the exact text files in `inputs/`, checked against `SHA256SUMS` at startup.
The shared Boolean case has 64 distinct atoms and 63 conjunctions (127 graph
nodes) under a caller limit of 128 nodes. This approaches that declared test
limit; it does not approach the library's 10,000-node hard maximum. The V4
fairness case uses the retained infinite corpus text.

For a same-host comparison, use one `CARGO_TARGET_DIR` and Criterion 0.5.1:

1. Archive the exact baseline parser commit into a temporary directory with
   `git archive <baseline-commit> | tar -x -C <baseline-dir>`. Do not edit the
   archived `src/` tree. Copy `benches/` and the candidate `Cargo.lock` into
   that directory. Add the candidate's `criterion` dev-dependency and `[[bench]]`
   declaration to its `Cargo.toml`. Set its `tl-syntax` revision to the exact
   candidate revision so both builds use one syntax dependency; no other
   package fields change. This is a copied measurement harness, not a claim
   that the baseline release carried it.
2. In the archived directory, run `cargo bench --locked --bench
   parser_roundtrip -- --save-baseline <baseline-name>` with
   `CARGO_TARGET_DIR=<shared-target>`.
3. Run `cargo clean --release -p tl-parse` when switching worktrees. This
   forces Cargo to rebuild the parser and benchmark binary while retaining
   Criterion's saved baseline in the shared target directory. From a clean
   candidate checkout, run `cargo bench --locked --bench
   parser_roundtrip -- --baseline <baseline-name>` with the same target
   directory and Rust toolchain. Keep host and release profile fixed.
4. Preserve the first Criterion directory, then repeat steps 2 and 3. Run
   `python3 scripts/benchmark_report.py --initial-criterion-dir
   <first-criterion-copy> --criterion-dir <shared-target>/criterion
   --baseline-name <baseline-name>
   --baseline-commit <full-sha> --candidate-commit <full-sha>
   --output <report.json>`. The script requires 20 samples for each of the
   seven workloads and records the normalized sample distributions, median
   confidence intervals, standard deviation, input hashes, source-tree IDs,
   machine details, and toolchain. It keeps both runs and classifies a
   one-run spike separately from a repeat-confirmed regression.
   If one case remains noisy, preserve and run a third paired baseline/candidate
   measurement, then pass that directory with `--additional-criterion-dir`.
   The report retains all three distributions and never drops an earlier spike.

Criterion warms each case for 500 ms and measures 20 samples over at least one
second. The campaign threshold is a median regression over 20% whose 95%
change confidence interval is entirely above 20%. A single paired run above
that threshold requires repetition; unmatched hardware or noisy overlapping
intervals remain inconclusive. Timing is separate from functional resource
limit and correctness gates.

The retained 2026-09-22 report measured parser commit
`462374f6bc7419b227ed10596cf903177f5da728` against
`976050cfb10df05ac263be581d02f2b64c6cce74`, with both builds pinned to
tl-syntax `5ced12e22917c56bb2ebd161a3e519cddf7a668a`. A later exact
tl-syntax repin is a separate candidate and is not covered by those timings.
The final syntax-pinned parser candidate is measured separately in
`reports/2026-09-22-parser-roundtrip-final-graph.json` against that same
archived baseline. Its report records three paired measurements, because two
cases were noisy after the second pair.

The final parser commit `98f7e1f29e459862f3704b5dc663b0d5d90bff5e`
with tl-syntax `8bcbce984f7ec3d86a92f90d866e842cc98b39fb` is measured
in `reports/2026-09-22-parser-roundtrip-final-98f7e1f.json`. The archived
baseline `src/` matched the exact `976050cfb10df05ac263be581d02f2b64c6cce74`
source tree. Both worktrees used the copied benchmark harness (SHA-256
`36d1f05f8e3a46fedec037da3423c35adba8c0104391230040d52a39bf0682b8`),
inputs, lockfile, syntax pin, Rust 1.98.1, and shared target directory. The
archive's manifest changed only to add the Criterion benchmark declaration
and to match the candidate's syntax revision. Each baseline/candidate switch
used `cargo clean --release -p tl-parse`; build output confirmed compilation
from the intended worktree. Three paired runs retained all 20 samples per
case. Two above-threshold first-run spikes did not recur in either repeat;
the report finds no repeat-confirmed regression above 20%.
