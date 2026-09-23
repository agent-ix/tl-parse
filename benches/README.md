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
   declaration to its `Cargo.toml`; no other package fields change. This is a
   copied measurement harness, not a claim that the baseline release carried
   it.
2. In the archived directory, run `cargo bench --locked --bench
   parser_roundtrip -- --save-baseline <baseline-name>` with
   `CARGO_TARGET_DIR=<shared-target>`.
3. From a clean candidate checkout, run `cargo bench --locked --bench
   parser_roundtrip -- --baseline <baseline-name>` with the same target
   directory and Rust toolchain. Keep host and release profile fixed.
4. Run `python3 scripts/benchmark_report.py --criterion-dir
   <shared-target>/criterion --baseline-name <baseline-name>
   --baseline-commit <full-sha> --candidate-commit <full-sha>
   --output <report.json>`. The script requires 20 samples for each of the
   seven workloads and records the normalized sample distributions, median
   confidence intervals, standard deviation, input hashes, source-tree IDs,
   machine details, and toolchain. It refuses an above-threshold or ambiguous
   result until the run is repeated and reviewed.

Criterion warms each case for 500 ms and measures 20 samples over at least one
second. The campaign threshold is a median regression over 20% whose 95%
change confidence interval is entirely above 20%. A single paired run above
that threshold requires repetition; unmatched hardware or noisy overlapping
intervals remain inconclusive. Timing is separate from functional resource
limit and correctness gates.
