# Parser fuzz targets

This isolated `cargo-fuzz` package exercises UTF-8 dialect parsing, bounded
statistics, report serialization, and successful canonical round trips. The
`parser` target covers `tl-parse.clean-ascii/v1`; the `clean_ascii_v2` target
covers `tl-parse.clean-ascii/v2` lowering and its primitive-only canonical text.
Their checked seeds are independently authored under `MIT OR Apache-2.0` and are
protected by `corpus/<target>/SHA256SUMS`.

Seed consumption is part of the normal Rust test suite. `make fuzz-smoke`
invokes the repository-owned `examples/fuzz_campaign.rs` producer for both
targets with fixed 64-execution and 300-second bounds, LeakSanitizer enabled,
and digest-verified copies of every declared seed. Each run emits a typed
`tl-parse.fuzz-campaign/v1` result; crash artifacts are represented by bounded
metadata identities while lossless binary attachment remains tracked by
`agent-ix/quoin#363`. The tool's `validate` mode is also the Rust-owned result
adapter: it refuses mutations to the protocol, target, trace, bounds, manifest,
tool, sanitizer, process, outcome, limitation, or artifact identities before
the existing Python driver invokes Quoin. Longer libFuzzer campaigns are
supplementary population evidence, not a universal proof.
