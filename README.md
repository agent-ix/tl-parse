# TL Parse

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

The v0.1 boundary uses the independently authored, versioned ASCII dialect in
[`docs/DIALECT-001-clean-room-mltl-v1.md`](docs/DIALECT-001-clean-room-mltl-v1.md).
It maps source directly into the exact pinned `tl-syntax` graph model and does
not own a second AST or temporal semantics.

The crate compiles against `tl-syntax` at
`842d82553f045eb69a7f38745756d968254fc25e`, an exact commit reachable from the
reviewed `main` history when admitted, not a moving branch head. That revision
carries the contextual and semantic contracts. The dialect was authored from
the earlier revision `740182f1`, which is
a separate and historical fact; `docs/ATTRIBUTION.md` records both, and
`Cargo.lock` is what enforces the compiled one. The dependency still resolves by
exact git revision because
`tl-syntax` has no registry release, and source release remains blocked while
that is true.

The additive `parse_clean_ascii_v2` API accepts bounded derived future
operators `W` and `M`. The additive `parse_clean_ascii_v3` and
`format_clean_ascii_v3` APIs provide the deliberately separate
origin-complete past profile with `O`, `H`, strong `Y`, `S`, and `T`. See
[`docs/DIALECT-002-clean-ascii-v2.md`](docs/DIALECT-002-clean-ascii-v2.md) and
[`docs/DIALECT-003-clean-ascii-v3.md`](docs/DIALECT-003-clean-ascii-v3.md) for
their closed grammars and wire identities. The v1 API and CLI remain unchanged.

The implementation is organized around closed `dialect::{v1,v2,v3}` policies.
The lexer, parser, formatter, and diagnostic layers share traversal and resource
accounting, while each dialect owns its accepted spellings, precedence,
associativity, semantic profile, owner schema, and lowering permission. Every
successful parse is re-admitted from canonical bytes by the pinned
`tl-syntax::FormulaDocument::from_json_bytes` boundary.

`PastParseReport::from_json_bytes(bytes, ParseArtifactLimits)` is the bounded
canonical reader for `tl-parse.past-parse-report/v1`. It rejects unknown or
duplicate fields, trailing data, noncanonical JSON, incorrect identities,
out-of-range report state, and invalid embedded owner documents.

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
through the packaged Engineering Assurance compatibility matrix and drives the
seal/intake/receipt chain.

This repository retains no evidence of its own. Verification evidence is what
the chain produces at the reviewed revision; Git history and pull-request review
are the integrity boundary for the source.

## Development status

Its public API is not stable yet, and registry publication is disabled until
the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
