# TL Parse

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

The v0.1 boundary uses the independently authored, versioned ASCII dialect in
[`docs/DIALECT-001-clean-room-mltl-v1.md`](docs/DIALECT-001-clean-room-mltl-v1.md).
It maps source directly into the `tl-syntax` graph model and does
not own a second AST or temporal semantics.

The dialect's clean-room authorship boundary is recorded in
`docs/ATTRIBUTION.md`. `tl-syntax` has no registry release, so the dependency
resolves from git, and source release remains blocked while that is true.

The additive `parse_clean_ascii_v2` API accepts bounded derived future
operators `W` and `M`. The additive `parse_clean_ascii_v3` and
`format_clean_ascii_v3` APIs provide the deliberately separate
origin-complete past profile with `O`, `H`, strong `Y`, `S`, and `T`. See
[`docs/DIALECT-002-clean-ascii-v2.md`](docs/DIALECT-002-clean-ascii-v2.md) and
[`docs/DIALECT-003-clean-ascii-v3.md`](docs/DIALECT-003-clean-ascii-v3.md) for
their closed grammars and wire identities. The v1 API and CLI remain unchanged.

The additive `parse_clean_ascii_v4(source, profile, clock, limits)` API accepts
the exact `mltl.infinite-trace/v1` and `event_position` identities. It builds
the owner's unbounded formula graph and optional ordered fairness premises.
`format_clean_ascii_v4` emits canonical text; the v4 report has a bounded,
strict JSON reader. See [`docs/DIALECT-004-clean-ascii-v4.md`](docs/DIALECT-004-clean-ascii-v4.md).

The implementation is organized around closed `dialect::{v1,v2,v3,v4}` policies.
The lexer, parser, formatter, and diagnostic layers share traversal and resource
accounting, while each dialect owns its accepted spellings, precedence,
associativity, semantic profile, owner schema, and lowering permission. Every
successful bounded parse is re-admitted from canonical bytes by
`tl-syntax::FormulaDocument::from_json_bytes`; v4 uses the owner's distinct
`InfiniteFormulaDocument` and fairness strict readers.

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

The hostile-input corpus is in `corpus/v1`; fuzz seeds and
the `cargo-fuzz` targets are under `fuzz/`. `make fuzz-smoke` runs both targets
through a bounded cargo-fuzz smoke run.

## Development status

Its public API is not stable yet, and registry publication is disabled until
the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
