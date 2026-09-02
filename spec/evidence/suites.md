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

SUITE-007 replaces the former local PGM-01 evidence validation suite. There is
no retained-evidence suite of any kind any more: the historical records this
repository kept were deleted under the owner's pre-stable release of the
preservation constraint (`agent-ix/engineering-assurance#7`), and the
compatibility view that read them went with them. The former SUITE-009
authored-diff integrity check went with the collector whose staging it
protected.

## Backing

Five of the nine suites have a bound test. What that binding is, precisely,
matters more than the count, because an earlier version of this section claimed
those tests "actually invoke that suite's command" and **none of them does**:

| Suite | Bound test | What the test actually does |
|---|---|---|
| SUITE-003 | TC-024 | reads the `quire coverage --json` export and pins its totals; it does not run `--strict` |
| SUITE-005 | TC-018 | runs `sha256sum --check` over `corpus/v1` only, not `fuzz/corpus/parser` |
| SUITE-006 | TC-023 | reads `msrv.jsonl` and asserts the attested result, rather than running the MSRV check |
| SUITE-007 | TC-023 | runs `scripts/assurance_chain.py` directly, so it covers the chain but not `pins` |
| SUITE-009 | TC-023 | reads the two producers' retained results rather than running `make conformance roundtrip` |

So these tests bind to the **retained output** of a suite, not to its
invocation. That is the architecture working as intended — the whole point is
that verdicts come from producer bytes — but it is a weaker claim than "invokes
the command", and it is stated here as the weaker claim it is.

Four suites have no bound test at all: SUITE-001 is the composite gate,
SUITE-002 is `quire validate`, SUITE-004 is `rustdoc`, and SUITE-008 is a manual
hosted dispatch. The only available way to make them read as backed is to assert
that the Makefile *contains* the command, which is the binding shape PR #6's
reviewers rejected because it is satisfied by the string rather than by the
behaviour.

Before this migration all nine rows were backed by a single test whose `// Trace:`
comment named SUITE-001 through SUITE-009 at once. Five output-bound rows and
four honestly empty ones is a better record than that, and the difference is
worth reading for what it is rather than as a coverage regression.
