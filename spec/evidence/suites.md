---
id: SUR-001
title: tl-parse v0.1 evidence suites
type: SuiteRegistry
---

# tl-parse v0.1 Evidence Suites

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Format, lint, tests, licenses, unsafe audit | `make ci` | Rust/Cargo tooling | Integration |
| SUITE-002 | Requirements and assurance validation | `quire validate --scope . 'spec/**/*.md' 'docs/*.md'` | Quire 0.31.0 | Analysis |
| SUITE-003 | Requirement coverage | `quire coverage --scope . --strict` | Quire 0.31.0 | Analysis |
| SUITE-004 | Rustdoc warnings | `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps` | rustdoc | Analysis |
| SUITE-005 | Corpus integrity | `make check-corpus` | sha256sum | Static |
| SUITE-006 | Minimum supported Rust boundary | `cargo +1.75.0 check --all-targets` | Rust 1.75.0 | Analysis |
| SUITE-007 | PGM-01 evidence validation | `make evidence-tool` | JSON Schema/PGM validator | Analysis |
