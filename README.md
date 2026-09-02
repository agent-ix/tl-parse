# TL Parse

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

The v0.1 boundary uses the independently authored, versioned ASCII dialect in
[`docs/DIALECT-001-clean-room-mltl-v1.md`](docs/DIALECT-001-clean-room-mltl-v1.md).
It maps source directly into the exact pinned `tl-syntax` graph model and does
not own a second AST or temporal semantics.

The crate compiles against `tl-syntax` at
`953ee825e5060335b4c79682f5f41a78c5a1bfae`, the head of that repository's
`main`. The dialect was authored from the earlier revision `740182f1`, which is
a separate and historical fact; `docs/ATTRIBUTION.md` records a SHA-256 table
for both. The dependency still resolves by exact git revision because
`tl-syntax` has no registry release, and source release remains blocked while
that is true.

## Build

```bash
make ci
```

`make ci` is the complete local iteration gate. Hosted GitHub Actions is
manual-only (`workflow_dispatch`) and is deliberately run only for a finalized
PR revision.

The thin CLI accepts a file or stdin:

```bash
cargo run --bin tl-parse -- validate --profile closed formula.mltl
printf 'p0 U[1,2] true' | cargo run --bin tl-parse -- format --profile online -
```

The checksum-protected hostile-input corpus is in `corpus/v1`; fuzz seeds and
the `cargo-fuzz` target are under `fuzz/`.

## Assurance

Verification results are produced by this repository's own tools, transcribed
and retained by [Quoin](https://github.com/agent-ix/quoin), and described by
static facts exported from [Quire](https://github.com/agent-ix/quire-rs).
Neither tool executes a producer. `make assurance` classifies the toolchain
through the packaged Engineering Assurance compatibility matrix, reads the
retained evidence under `evidence/` through that release's compatibility
mapping, and drives the seal/intake/receipt chain.

Retained evidence is immutable and is no longer verified by a repository-local
verifier: Git history and pull-request review are its integrity boundary.

## Development status

Its public API is not stable yet, and registry publication is disabled until
the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
