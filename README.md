# TL Parse

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

The v0.1 boundary uses the independently authored, versioned ASCII dialect in
[`docs/DIALECT-001-clean-room-mltl-v1.md`](docs/DIALECT-001-clean-room-mltl-v1.md).
It maps source directly into the exact pinned `tl-syntax` graph model and does
not own a second AST or temporal semantics.

## Build

```bash
make test
make spec
```

## Development status

This crate is being developed spec-first. Its public API is not stable yet, and
registry publication is disabled until the v0.1 assurance review is complete.

Agent-assisted contributions are reviewed under the same requirements,
testing, provenance, and human release gates as every other contribution.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your
option.
