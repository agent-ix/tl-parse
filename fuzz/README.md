# Parser fuzz target

This isolated `cargo-fuzz` package exercises UTF-8 dialect parsing, bounded
statistics, report serialization, and successful canonical round trips. Its
checked seeds are independently authored under `MIT OR Apache-2.0` and are
protected by `corpus/parser/SHA256SUMS`.

Seed consumption is part of the normal Rust test suite. Longer libFuzzer
campaigns are supplementary population evidence, not a universal proof.
