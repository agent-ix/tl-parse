---
id: SR-067
title: "TL-246 parser V9 historical baseline code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-parse@02e167e6728e0daf574dc637700d970a9dcb22e7; benches/parser_roundtrip.rs, benches/inputs/{SHA256SUMS,*.txt}, Cargo.toml, Cargo.lock; compared with agent-ix/tl-parse@86d43068b927e3973481222b6f3a7072d81afb81 and tl-mltl campaign/V9_CRITERION.md, campaign/v9_criterion.py, campaign/source-closure.json, src/bin/tl_campaign_check/v9_replay.rs"
review_set: subset
---

## Summary

Ticket: TL-246. Reviewed draft tl-parse PR #55 at exact head 02e167e6728e0daf574dc637700d970a9dcb22e7 against its exact parent 225a3300963199e523958f4abe4c16cc9dcca641. The sole changed file, `benches/parser_roundtrip.rs`, has blob 75e03a9e134ab69b85310f8be0c4b2cc591ae8c2, identical to current main's benchmark file. The six pinned input files and SHA256SUMS, Cargo.toml, and Cargo.lock are unchanged from the parent; the lock retains tl-syntax fed48a2fab3f5277131a7012e1a8d10993330cb1. The current candidate lock has a different tl-syntax revision, which is intentional for distinct historical and current source graphs. The parse-only and parse/format timed scopes are separate, and V9's current runner selects the ten parser_roundtrip cases. Direct compilation under the historical lock and all 20 Criterion smoke cases passed. This review makes no V9 performance or Campaign acceptance claim.

## Verdict

**PASS** for this one-file historical harness staging change. No code finding. Published tl-mltl Campaign controls still pin the prior baseline and do not yet use PR #55; coherent repinning, paired measurements, and retained replay remain TL-246 integration work.

## Assurance Context

`spec/assurance/AP-001.md` applies to the exact historical source/dependency candidate. Evaluated the reviewed head against historical parent 225a3300963199e523958f4abe4c16cc9dcca641 and current harness blob 75e03a9e134ab69b85310f8be0c4b2cc591ae8c2. The profile's material impacts are silent reinterpretation and hostile growth; this benchmark change does not alter parser behavior or its resource limits. The harness verifies immutable input digests and runs real parser/formatter APIs before timing. The changed source, input, manifest, and lock identities were inspected directly. Measurement evidence is limited to compile and Criterion smoke, with no timed distributions, paired host results, independent replay, or source-release approval. No exception was invoked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verification

- `cargo bench --locked --offline --bench parser_roundtrip --no-run` on an archive of the exact PR head: pass.
- Compiled Criterion executable `--test --noplot`: 20/20 parse-only and parser-roundtrip cases pass.
- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --offline --bench parser_roundtrip -- -D warnings`: pass.
- `cargo deny check`: pass for advisories, bans, licenses, and sources.
- Full `cargo test --locked --offline` in a fresh archive did not pass: ten `tests/shared_assurance.rs` cases require `target/assurance/parser-conformance.jsonl` from `make assurance-inputs`, which is absent by design in a fresh archive. All observed parser and corpus tests before that suite passed. The first attempt also had temporary-directory permission failures; rerunning with `TMPDIR=/private/tmp` resolved those. The benchmark change was directly verified by the Criterion smoke run above.
