# Parser fuzz targets

This isolated `cargo-fuzz` package exercises UTF-8 dialect parsing, bounded
statistics, report serialization, and successful canonical round trips. The
`parser` target covers `tl-parse.clean-ascii/v1`; the `clean_ascii_v2` target
covers `tl-parse.clean-ascii/v2` lowering and its primitive-only canonical text.
Their checked seeds are independently authored under `MIT OR Apache-2.0` and are
protected by `corpus/<target>/SHA256SUMS`.

Seed consumption is part of the normal Rust test suite. Longer libFuzzer
campaigns are supplementary population evidence, not a universal proof.
