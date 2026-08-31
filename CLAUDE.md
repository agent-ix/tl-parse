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
make check-corpus   # verify hostile-input and fuzz-seed checksums
make evidence-tool  # exercise evidence outcome and schema tooling
make verify-evidence # verify retained exact-candidate evidence
make spec           # validate specifications and strict coverage
make rustdoc        # build warning-free public docs
make ci             # complete local iteration gate
```

GitHub Actions is intentionally manual-only. Do not add `push` or
`pull_request` triggers. Run local `make ci` while iterating and dispatch the
hosted workflow once for a finalized PR revision.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
src/lexer.rs           # bounded closed-dialect lexer
src/parser.rs          # direct tl-syntax graph parser
src/format.rs          # iterative bounded canonical formatter
src/bin/tl-parse.rs    # thin validate/format CLI
tests/                 # unit, property, corpus, CLI, and evidence contracts
corpus/v1/             # checksum-protected hostile-input fixtures
fuzz/                  # isolated cargo-fuzz target and checked seeds
evidence/              # exact-candidate retained evidence
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```

The exact `tl-syntax` candidate is pinned at
`740182f13b84858008d6f176f75136737d405c1b`. Its temporary git-source
exception expires on 2026-09-07 or upstream merge, whichever comes first;
release requires repinning and regenerating evidence.
