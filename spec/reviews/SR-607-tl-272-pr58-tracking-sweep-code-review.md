---
id: SR-607
title: "tl-parse#58 tracking-ceremony sweep code and Rust review"
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-parse@72f862ec3bf54c8bdf96b7bf40e677eb57a9c267; diff against origin/main 86d4306; examples/fuzz_campaign.rs; tests/{cli,corpus,clean_ascii_v2,infinite_trace_corpus,infinite_v4,owner_infinite_corpus,past_history_corpus}.rs; tests/shared_assurance.rs (deleted); benches/parser_roundtrip.rs; scripts/rust_test_census.py; scripts/{assurance_chain,check_checksum_manifest,check_shared_pins}.py (deleted); Makefile; Cargo.toml; Cargo.lock; deny.toml; docs/ATTRIBUTION.md; docs/DIALECT-001-clean-room-mltl-v1.md; docs/DIALECT-004-clean-ascii-v4.md; requirements-assurance.txt; .github/workflows/ci.yml (context only)"
review_set: subset
---

# tl-parse#58 tracking-ceremony sweep code and Rust review

## Summary

Ticket: TL-272. PR: agent-ix/tl-parse#58, branch `chore/drop-pgm01-citations`,
reviewed at `72f862e` against `origin/main` `86d4306`. This file holds
code-review with its Rust lane folded in (rust-review idiom, test, panic and
resource checklist).

Owner rule applied: tracking ceremony is deleted. That covers checksums,
pins.json, pin checkers, assurance-chain scripts, digest-pinned manifests,
tool-identity probes, tests that assert pins or digests, and inventories.
Content-identity digests that belong to the product's own wire or API
semantics stay, as do Cargo.toml and Cargo.lock.

## Verdict

**CONDITIONAL: three medium and three low findings. No finding blocks on
correctness.** The sweep is sound. Every deletion checked was ceremony, and the
leftovers are consistency gaps.

- **`examples/fuzz_campaign.rs` still fuzzes and still fails closed.** It stages
  the repository seeds, runs `cargo fuzz run <target>` with 64 runs under a
  300-second process-group deadline, and sets `ASAN_OPTIONS=detect_leaks=1` with
  the TL-195 LSAN suppression. It exits 0 only when the fuzzer exits 0 and
  leaves no crash artifact. Each other case exits nonzero:
  - a nonzero exit, a signal (`code()` is None, so -1) or an artifact exits 1;
  - a timeout, an ambient sanitizer override, unavailable LeakSanitizer, or a
    setup error exits 2.

  This was checked on this host, where cargo-fuzz is absent: `make fuzz-smoke`
  exited nonzero and kept the scratch directory. What was deleted is ceremony:
  - the seed manifest digests;
  - the tool identity probes;
  - the versioned JSON protocol and its validate mode.

  The TC-047 unit tests now use a `tc_047_` prefix, which matches their trace.
  Before, they used `tc_046_`, which was wrong.
- **The corpus tests still test behaviour.** `corpus.rs` still replays every
  manifest fixture to its declared outcome. The fuzz-seed tests still count
  seeds (for example `paths.len() == 5`) and parse every seed.
  `infinite_trace_corpus.rs` still checks spans, codes and mutation oracles.
  `past_history_corpus.rs` and `owner_infinite_corpus.rs` still replay the
  shared tl-syntax corpora. The SHA256SUMS checks were not the only guard of
  corpus correctness. They guarded byte identity, which git already does.
  Dropping `deny_unknown_fields` on the past-history `Manifest` is needed
  because upstream still carries `files` and `implementation_revisions`.
- **Deleting `tests/shared_assurance.rs` is correct.**
  - TC-022 to TC-028 exercised the deleted chain.
  - TC-032 was a tool-identity probe: a census of the ix-flow package in the
    workflow.
  - TC-029 was a duplicate-SpecReview-id inventory. `quire validate` does not
    catch a duplicate SR id (this was checked by copying SR-013 to a scratch
    tree). That is repo hygiene, not product function, and the owner rule
    lists inventories.
- **Kept items that pass the value test:**
  - **`TL_SYNTAX_REVISION`** is a wire field. Every report emits it, and the v2,
    v3, v4 and contextual strict readers refuse a mismatch.
  - **The test that it matches Cargo.toml and both lockfiles** (`tests/cli.rs:44-49`)
    is the only thing that keeps that wire field true after a repin, so it stays.
  - **`CORPUS_REVISION`** is a manifest format label that
    `corpus_conformance` checks. It is not a pin.
  - **The record digests `dialect{,_v2,_v3}_digest`** are the dialect identity
    that the dialect documents publish.
  - **Cargo.toml `rev =`** stays under the owner rule.
  - **The Make execution-control guard** protects real gate integrity. Its
    self-test passes.
- **"exact pinned" prose** remains in StR-001-VC-2, FR-001, FR-002-AC-2, FR-007,
  FR-008, FR-009 and in `src/parser.rs` error strings. It describes the Cargo
  `rev =` pin, which is real, so it is accurate and no change is needed.
- **Gates:** see Review Evidence. There are no new failures.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Digest-assertion sweep is inconsistent. The v1 record, v1 document and attribution digest asserts were removed from tests/cli.rs. The hard-coded v2 and v3 record and document digest asserts stay. DIALECT-001 still claims record drift "is detectable by tests and evidence". The prose-document digests (`*_document_digest`, `attribution_document_digest`, and the unused `dialect_v4_document_digest`) hash markdown and back no wire field. Fix: keep the record-digest fns as dialect identity API and assert them uniformly (restore v1 or drop v2/v3). Delete the document-digest fns and their asserts. Fix the DIALECT-001:24 sentence. | tests/clean_ascii_v2.rs:234-240; tests/clean_ascii_v3.rs:52-58; src/lib.rs:57,124,138,152,159; docs/DIALECT-001-clean-room-mltl-v1.md:22-24 |
| FND-002 | medium | Deleting the compiled-pin delta sections of ATTRIBUTION.md also deleted the only clean-room authorship-basis statement for v3: past spellings were authored from tl-syntax MRS-003/FR-013 and no third-party grammar was consulted. That is provenance substance, not pin tracking. ATTRIBUTION.md:11 now says "The tl-parse dialect" (singular) was authored only from 740182f1, which overstates for v2 to v4. Fix: add a SHA-free per-dialect authorship line for v3 to DIALECT-003 or ATTRIBUTION.md, and scope ATTRIBUTION.md:11 to v1. | docs/ATTRIBUTION.md:11; docs/DIALECT-003-clean-ascii-v3.md:12-15 |
| FND-003 | medium | requirements-assurance.txt still installs the pinned `engineering-assurance@v0.2.1` git tag, which nothing in the repo imports now. Its header cites the deleted FR-006 and argues for keeping the pin. ci.yml calls `make assurance-env` and cannot be edited, so the target stays. Fix: reduce the file to a comment saying it is kept only because ci.yml calls the target, with no requirement lines. `pip install -r` of a comment-only file succeeds. | requirements-assurance.txt:1-27 |
| FND-004 | low | After the digest removal, `changed_source` is built and only compared with `source`. `assert_ne!` after a known byte write is a tautology and exercises nothing. Fix: delete the three lines, or feed `changed_source` to the parser and assert the oracle mismatch. | tests/infinite_trace_corpus.rs:160-162 |
| FND-005 | low | `copy_seeds` silently skips a non-regular seed entry (a symlink or a directory), so that seed is never consumed. FR-005-AC-2 says the smoke run "consumes every seed", and the old runner refused such entries. Fix: return an error for a non-regular entry, which maps to Unavailable, exit 2. | examples/fuzz_campaign.rs:112-123 |
| FND-006 | low | A stale header comment says Quoin digest binding covers producers "(the `assurance-inputs` chain)". That chain is deleted, and this script is now the only guard. Fix: drop the Quoin and assurance-inputs sentence. | scripts/check_make_execution_control.sh:5-7 |

## Review Evidence

These gates ran in a fresh detached worktree at `72f862e`, with one target
directory that was deleted afterwards:

| Gate | Exit |
| --- | --- |
| `cargo fmt --all -- --check` | 0 |
| `make lint` (clippy `--all-targets --all-features -D warnings`) | 0 |
| `cargo test --all-targets --all-features` | 0 |
| `make deny` | 0 |
| `make spec` | 2 |
| `make test-census` (97 tagged tests) | 0 |
| `make conformance` | 0 |
| `make test-execution-control-guard` | 0 |
| `make fuzz-smoke` | 2 |

**`make spec`: no new failures.** It fails here and on origin/main with the same
pre-existing TestMatrix `Coverage Status`/`Status` column assert. Main also
fails MP-001 frontmatter, which this PR deletes: main has 2 failed documents
and the PR has 1. `quire coverage --strict` gives an identical
unmatched-trace set on both, compared by diff.

**`make fuzz-smoke`: environment limit.** cargo-fuzz is not installed on this
host. The runner exited nonzero, which is the fail-closed path. A real campaign
was not exercised.

## Assurance Context

Reviewer-only. Nothing was edited or pushed. PR text was treated as untrusted
and each claim was re-measured.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
