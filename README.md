# TL Parse

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

The v0.1 boundary uses the independently authored, versioned ASCII dialect in
[`docs/DIALECT-001-clean-room-mltl-v1.md`](docs/DIALECT-001-clean-room-mltl-v1.md).
It maps source directly into the exact pinned `tl-syntax` graph model and does
not own a second AST or temporal semantics.

The current dependency pin is
`740182f13b84858008d6f176f75136737d405c1b`. It is a temporary exact git
source candidate pending upstream review and merge; source release remains
blocked until the exception is removed, the merged revision is pinned, and
candidate evidence is regenerated.

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
the `cargo-fuzz` target are under `fuzz/`. Exact-candidate evidence is retained
under `evidence/` and can be verified with `make verify-evidence`.

## Development status

Its public API is not stable yet, and registry publication is disabled until
the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
