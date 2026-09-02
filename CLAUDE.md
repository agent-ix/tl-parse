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
make conformance    # replay the hostile-input corpus through the crate
make roundtrip      # sweep the parse-format-parse fixed point
make test-census    # bind requirement-tagged tests to compiled tests
make fuzz-build     # compile the checked-in cargo-fuzz target locally
make fuzz-smoke     # execute a bounded local libFuzzer seed smoke run
make spec           # validate specifications and strict coverage
make rustdoc        # build warning-free public docs
make ci             # complete local iteration gate

# shared assurance
make assurance-env     # build .venv-assurance from requirements-assurance.txt
make assurance-inputs  # the ONLY target that runs a producer
make pins              # classify the toolchain through the packaged matrix
make assurance-chain   # seal, retain, and verify through Quoin
make assurance         # pins + assurance-chain
```

`make assurance-inputs` is the only target that runs a producer. Everything
downstream consumes what it wrote and refuses to create it; an absent input is
an error naming that target, never a skip.

GitHub Actions is intentionally manual-only. Do not add `push` or
`pull_request` triggers. Run local `make ci` while iterating and dispatch the
hosted workflow once for a finalized PR revision.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses only stable 100-character-width settings. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
src/lexer.rs           # bounded closed-dialect lexer
src/parser.rs          # direct tl-syntax graph parser
src/format.rs          # iterative bounded canonical formatter
src/bin/tl-parse.rs    # thin validate/format CLI
examples/              # the two domain producers: corpus replay, round-trip sweep
tests/                 # unit, property, corpus, CLI, and shared-assurance tests
corpus/v1/             # checksum-protected hostile-input fixtures
fuzz/                  # isolated cargo-fuzz target and checked seeds
assurance/             # pins.json and change-assurance.json: declarations only
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```

## Assurance boundary

Evidence retention, integrity checking, audit, attestation and receipts are
Quoin's. Static specification, obligation and coverage facts are Quire's. This
repository owns its parser, formatter, diagnostics, corpus and fuzz target, and
declares their results in structured formats the shared tools transcribe.
Neither Quire nor Quoin executes a producer, and `tests/shared_assurance.rs`
asserts that behaviourally rather than by inspection.

This repository retains no evidence of its own. There is no local verifier,
anchor file, manifest, or tool lock; Git history and pull-request review are the
integrity boundary. The twelve historical records under `evidence/` were deleted
on 2026-09-02 under the owner's pre-stable release of the preservation
constraint (`agent-ix/engineering-assurance#7`, `agent-ix/tl-parse#13`). Do not
reintroduce an `evidence/` directory, a `schemas/` freeze, or a compatibility
view over them — `tests/shared_assurance.rs` fails if any of them comes back.
The constraint re-applies when this repository moves toward stable releases.

## The tl-syntax pin

Two different facts, and they no longer coincide.

The crate **compiles against** `953ee825e5060335b4c79682f5f41a78c5a1bfae`, the
head of tl-syntax `main`. The dialect was **authored from**
`740182f13b84858008d6f176f75136737d405c1b`, which is historical and does not
move. `docs/ATTRIBUTION.md` records both as separate facts. `Cargo.toml` and
`Cargo.lock` enforce the compiled pin; there is no local digest table and no
re-derivation gate. Both were dropped under `agent-ix/tl-parse#15` because
740182f1 lives on a deleted upstream branch, so its digests could never be
re-derived by anyone and recorded an unverifiable claim in a verifiable-looking
shape.

The earlier form of the `deny.toml` git-source exception was scoped to "until
tl-syntax PR #6 merges or 2026-09-07". That merge has happened — as PR #10,
which superseded #6 — so the pin moved onto `main` and the expiry no longer
applies. The exception itself remains because tl-syntax has no registry
release.
