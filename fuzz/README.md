# Parser fuzz targets

This isolated `cargo-fuzz` package exercises UTF-8 dialect parsing, bounded
statistics, report serialization, and successful canonical round trips. The
`parser` target covers `tl-parse.clean-ascii/v1`; the `clean_ascii_v2` target
is the v2 lowering and primitive-text path. `clean_ascii_v3` exercises past-time
regressions; `unbounded_parse_roundtrip` exercises v4 mixed-time parsing,
fairness, resource limits, and canonical-text fixed points.
Their checked seeds are independently authored under `MIT` and are
protected by `corpus/<target>/SHA256SUMS`.

Seed consumption is part of the normal Rust test suite. `make fuzz-smoke`
invokes the repository-owned `examples/fuzz_campaign.rs` producer for the
original two targets with fixed 64-execution and 300-second bounds, LeakSanitizer enabled,
and digest-verified copies of every declared seed. Each run emits a typed
`tl-parse.fuzz-campaign/v1` result; crash artifacts are represented by bounded
metadata identities while lossless binary attachment remains tracked by
`agent-ix/quoin#363`. The tool's `validate` mode is also the Rust-owned result
adapter: it refuses mutations to the protocol, target, trace, bounds, manifest,
tool, sanitizer, process, outcome, limitation, or artifact identities before
the existing Python driver invokes Quoin. Longer libFuzzer campaigns are
supplementary population evidence, not a universal proof.

## Recorded V4 libFuzzer run

On a clean source commit with nightly `cargo`/`rustc` and `cargo-fuzz` on `PATH`:

```bash
export PATH="$(dirname "$(rustup which --toolchain nightly cargo)"):$HOME/.cargo/bin:$PATH"
PYTHONPATH=fuzz python3 -m unittest fuzz.test_run_v4_campaign
python3 fuzz/run_v4_campaign.py --output fuzz/evidence/v4-2026-09-23 \
  --runs 1000 --seed 181 --seconds 30
```

This drives `unbounded_parse_roundtrip` through the public v4 parser,
formatter, and second parse. The runner verifies checked seed bytes, copies
them to scratch, and retains lossless gzip engine output, exact source/tool/lock pins,
budget, actual executions, stop reason, and crash artifact hashes. Any crash
requires minimization and same-revision replay before the lane can be treated
as clean. A no-crash result describes only its recorded finite run.
