# Parser fuzz targets

This isolated `cargo-fuzz` package exercises UTF-8 dialect parsing, bounded
statistics, report serialization, and successful canonical round trips. The
`parser` target covers `tl-parse.clean-ascii/v1`; the `clean_ascii_v2` target
is the v2 lowering and primitive-text path. `clean_ascii_v3` exercises the
past-time regression, and `unbounded_parse_roundtrip` exercises the v4 parser,
formatter, and canonical fixed point. `numeric_v5_roundtrip` (TL-421, unbuilt)
splits a catalog/profile selector byte from the source and asserts the v5
parse-format fixed point under the hard maxima. `clean_ascii_v6` (TL-429,
unbuilt) takes a profile bit and asserts the v6 round trip; its 58 seeds are
the 29 `corpus/v6` hostile cases under each profile bit. Each target has checked seeds.
Their checked seeds are independently authored under `MIT OR Apache-2.0`.

Seed consumption is part of the normal Rust test suite. `make fuzz-smoke`
invokes the repository-owned `examples/fuzz_campaign.rs` runner for both
targets with fixed 64-execution and 300-second bounds and LeakSanitizer
enabled; it fails on a nonzero fuzzer exit or any crash artifact. Longer
libFuzzer campaigns are supplementary population evidence, not a universal
proof.
