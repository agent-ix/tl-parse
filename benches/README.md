# Parser Criterion procedure

`parser_roundtrip` measures parsing followed by canonical formatting through
public closed trace, past, and infinite trace APIs. It runs ten cases from six
checked-in UTF-8 files. The harness checks every input against `inputs/SHA256SUMS`
before it measures anything. The shared median and near-cap inputs are exercised
through all three APIs; the near-cap case uses a caller node limit of 128.

Run the producer from a clean checkout with the committed lockfile:

```sh
CARGO_TARGET_DIR="$PWD/target" cargo bench --locked --bench parser_roundtrip
```

Criterion 0.5.1 uses 20 samples per case, a 500 ms warmup, and at least one
second of measurement. Save `target/criterion/parser_roundtrip/` with the
producer commit, dependency lock, exact input digests, Rust toolchain, host,
and command. For a paired comparison, run baseline and candidate on the same
quiet host and toolchain, with the same harness and inputs, retaining both raw
sample and estimate files. Repeat the pair before treating an apparent
regression as confirmed. Report missing, failed, noisy, or unmatched pairs as
inconclusive; a benchmark result alone says nothing about functional
correctness or campaign acceptance.
