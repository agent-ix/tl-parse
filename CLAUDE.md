# tl-parse

Parsing, formatting, and diagnostics for Mission-time Linear Temporal Logic.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses and sources
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make conformance    # replay the hostile-input corpus through the crate
make roundtrip      # sweep the parse-format-parse fixed point
make test-census    # bind requirement-tagged tests to compiled tests
make fuzz-build     # compile the checked-in cargo-fuzz target locally
make fuzz-smoke     # execute a bounded local libFuzzer seed smoke run
make spec           # validate specifications and strict coverage
make rustdoc        # build warning-free public docs
make ci             # complete local iteration gate
```

GitHub Actions is intentionally manual-only. Do not add `push` or
`pull_request` triggers. Run local `make ci` while iterating and dispatch the
hosted workflow once for a finalized PR revision.

## Specification workflow

All implementation and specification changes require a Quoin authoring and
review record before review: run `quoin write . --types <relevant-types>` before
authoring artifacts, then run `quoin review --target <affected-spec-scope>` and
record the selected review set. Validate with
`quire validate --scope . "spec/**/*.md"`; register each validated review
artifact with the workflow. Leave the final workflow acceptance for a human —
it is not an agent approval gate.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.98.1` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses only stable 100-character-width settings. CI fails on drift.
- `rust-toolchain.toml` pins the exact MSRV (1.98.1) + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
src/lexer.rs           # bounded closed-dialect lexer
src/parser.rs          # direct tl-syntax graph parser
src/derived.rs         # clean-ascii v2 derived-operator report
src/format.rs          # iterative bounded canonical formatter
src/bin/tl-parse.rs    # thin validate/format CLI
examples/              # domain producers: corpus replay, round-trip sweep, fuzz campaigns
tests/                 # unit, property, corpus, and CLI tests
corpus/v1/             # hostile-input fixtures
fuzz/                  # isolated cargo-fuzz target and checked seeds
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
