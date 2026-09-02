---
id: SUR-001
title: tl-parse v0.1 evidence suites
type: SuiteRegistry
---

# tl-parse v0.1 Evidence Suites

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Complete local candidate gate | `make ci` | Rust/Cargo/Python/Quire/Quoin tooling | Integration |
| SUITE-002 | Requirements and assurance validation | `quire validate --scope . 'spec/**/*.md' 'docs/*.md' --strict --summary` | Quire 0.31.0 | Analysis |
| SUITE-003 | Requirement coverage | `quire coverage --scope . --strict` | Quire 0.31.0 | Analysis |
| SUITE-004 | Rustdoc warnings | `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --all-features` | rustdoc | Analysis |
| SUITE-005 | Corpus integrity | `make check-corpus` | sha256sum | Static |
| SUITE-006 | Minimum supported Rust boundary | `rustup run 1.75.0 cargo check --locked --all-targets --all-features` | Rust 1.75.0 | Analysis |
| SUITE-007 | Shared assurance intake | `make assurance` | quire-cli 0.31.0, Quoin 0.23.1, engineering-assurance 0.2.0 | Integration |
| SUITE-008 | Hosted candidate confirmation | Manual `workflow_dispatch` once for a finalized PR revision | GitHub Actions | Integration |
| SUITE-009 | Parser conformance and round-trip | `make conformance roundtrip` | tl-parse corpus runner and round-trip sweep | Integration |

Hosted CI intentionally has no push or pull-request trigger. Local `make ci` is
the iteration gate; a hosted run is dispatched deliberately for a finalized
revision so parallel PR work does not generate repeated billable runs.

SUITE-007 replaces the former local PGM-01 evidence validation suite. Retained
evidence is no longer verified by a repository-local verifier: its bytes are
immutable, Git history and pull-request review are the integrity boundary for
them, and they are read through the Engineering Assurance compatibility mapping.
The former SUITE-009 authored-diff integrity check went with the collector whose
staging it protected.
