# TL Parse

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

The v0.1 boundary uses the independently authored, versioned ASCII dialect in
[`docs/DIALECT-001-clean-room-mltl-v1.md`](docs/DIALECT-001-clean-room-mltl-v1.md).
It maps source directly into the exact pinned `tl-syntax` graph model and does
not own a second AST or temporal semantics.

The crate compiles against `tl-syntax` at
`43dcd3646d14922e20dee1be17b258be00ebf027`, an exact reviewed commit, not a
moving branch head. That revision
carries the contextual and semantic contracts. The dialect was authored from
the earlier revision `740182f1`, which is
a separate and historical fact; `docs/ATTRIBUTION.md` records both, and
`Cargo.lock` is what enforces the compiled one. The dependency still resolves by
exact git revision because
`tl-syntax` has no registry release. Git source releases use exact revisions;
registry publication remains disabled.

The additive `parse_clean_ascii_v2` API accepts bounded derived future
operators `W` and `M`. The additive `parse_clean_ascii_v3` and
`format_clean_ascii_v3` APIs provide the deliberately separate
origin-complete past profile with `O`, `H`, strong `Y`, `S`, and `T`. See
[`docs/DIALECT-002-clean-ascii-v2.md`](docs/DIALECT-002-clean-ascii-v2.md) and
[`docs/DIALECT-003-clean-ascii-v3.md`](docs/DIALECT-003-clean-ascii-v3.md) for
their closed grammars and wire identities. The v1 API and CLI remain unchanged.

The additive `parse_clean_ascii_v4(source, profile, clock, limits)` API requires
`mltl.infinite-trace/v1` and `event_position`. It builds the distinct
`tl-syntax.formula-unbounded/v1` graph, including bounded and unbounded future
and past operators and optional ordered fairness premises. Use
`format_clean_ascii_v4` for canonical text and
`InfiniteParseReport::from_json_bytes` for strict report admission. See
[`docs/DIALECT-004-clean-ascii-v4.md`](docs/DIALECT-004-clean-ascii-v4.md).

The implementation is organized around closed `dialect::{v1,v2,v3,v4}` policies.
The lexer and diagnostics share resource accounting; v4 traverses the distinct
unbounded owner graph. Each dialect owns its accepted spellings, precedence,
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
the `cargo-fuzz` targets are under `fuzz/`. `make fuzz-smoke` runs both targets
through the bounded Rust campaign producer and writes one
`tl-parse.fuzz-campaign/v1` result per target for shared assurance intake.

## Assurance

Verification results are produced by this repository's own tools, transcribed
and retained by [Quoin](https://github.com/agent-ix/quoin), and described by
static facts exported from [Quire](https://github.com/agent-ix/quire-rs).
Neither tool executes a producer. The two fuzz-campaign documents are retained
byte-identically as separate Quoin proof inputs, and the same Rust campaign tool
validates their closed protocol before the existing Python driver hands the
mapped result to Quoin. `make assurance` classifies the toolchain through the
packaged Engineering Assurance compatibility matrix and drives the
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

Licensed under the MIT license. See [LICENSE](LICENSE).
