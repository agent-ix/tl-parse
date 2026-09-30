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
| SUITE-002 | Requirements validation | `quire validate --scope . 'spec/**/*.md' 'docs/*.md' --strict --summary` | Quire | Analysis |
| SUITE-003 | Requirement coverage | `quire coverage --scope . --strict` | Quire | Analysis |
| SUITE-004 | Rustdoc warnings | `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` | rustdoc | Analysis |
| SUITE-006 | Minimum supported Rust boundary | `rustup run 1.98.1 cargo check --locked --all-targets --all-features` | Rust 1.98.1 | Analysis |
| SUITE-008 | Hosted candidate confirmation | Manual `workflow_dispatch` once for a finalized PR revision | GitHub Actions | Integration |
| SUITE-009 | Parser conformance and round-trip | `make conformance roundtrip` | tl-parse corpus runner and round-trip sweep | Integration |
| SUITE-010 | Bounded fuzz smoke | `make fuzz-smoke` | Rust fuzz smoke runner and cargo-fuzz/nightly toolchain | Fuzz |

Hosted CI intentionally has no push or pull-request trigger. Local `make ci` is
the iteration gate; a hosted run is dispatched deliberately for a finalized
revision so parallel PR work does not generate repeated billable runs.
