---
id: SUR-001
title: tl-parse v0.1 evidence suites
type: SuiteRegistry
---

# tl-parse v0.1 Evidence Suites

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Complete local candidate gate | `make ci` | Rust/Cargo/Python/Quire tooling | Integration |
| SUITE-002 | Requirements and assurance validation | `quire validate --scope . 'spec/**/*.md' 'docs/*.md'` | Quire 0.31.0 | Analysis |
| SUITE-003 | Requirement coverage | `quire coverage --scope . --strict` | Quire 0.31.0 | Analysis |
| SUITE-004 | Rustdoc warnings | `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` | rustdoc | Analysis |
| SUITE-005 | Corpus integrity | `make check-corpus` | sha256sum | Static |
| SUITE-006 | Minimum supported Rust boundary | `cargo +1.75.0 check --all-targets --all-features` | Rust 1.75.0 | Analysis |
| SUITE-007 | PGM-01 evidence validation | `make evidence-tool` | JSON Schema/PGM validator | Analysis |
| SUITE-008 | Hosted candidate confirmation | Manual `workflow_dispatch` once for a finalized PR revision | GitHub Actions | Integration |

Hosted CI intentionally has no push or pull-request trigger. Local `make ci`
is the iteration gate; a hosted run is dispatched deliberately for a finalized
revision so parallel PR work does not generate repeated billable runs.
