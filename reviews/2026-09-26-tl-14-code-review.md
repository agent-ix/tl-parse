---
id: SR-066
title: TL-14 code and Rust review of tl-parse PR 53
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-parse@e7fe85f506ccfad2eb7835a7a8485505ff04619d; CLAUDE.md, Cargo.toml, Cargo.lock, README.md, corpus/infinite-trace/*, docs/ATTRIBUTION.md, docs/DIALECT-004-clean-ascii-v4.md, fuzz/Cargo.toml, fuzz/Cargo.lock, fuzz/README.md, fuzz/corpus/clean_ascii_v3/*, fuzz/corpus/unbounded_parse_roundtrip/*, fuzz/fuzz_targets/clean_ascii_v3.rs, fuzz/fuzz_targets/unbounded_parse_roundtrip.rs, spec/infinite-trace-test-matrix.md, spec/requirements/FR-015, FR-016, FR-017, NFR-004, src/diagnostic.rs, src/dialect/{mod,v1,v3,v4}.rs, src/infinite.rs, src/lexer.rs, src/lib.rs, tests/infinite_trace_corpus.rs, tests/infinite_v4.rs, tests/owner_infinite_corpus.rs"
review_set: subset
---

## Summary

Reviewed TL-14's frozen v4 candidate with the Rust lane of code-review against FR-015/016/017, NFR-004 and AP-001. The candidate adds a distinct owner-graph parser and formatter, but changes legacy refusal bytes and loses the syntax refusal class when diagnostics are suppressed.

## Verdict

FAIL: two high-severity behavioral findings require a fix round. The provisional TL-15 pin also remains a dependency hold until TL-15 lands and this candidate is repinned and rechecked. Passing focused checks are review evidence only.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Shared lexer changes alter v1/v2/v3 diagnostic report bytes for malformed text, violating FR-015's legacy byte-preservation promise. | src/lexer.rs:217; src/lexer.rs:256; FR-015-AC-3 | implementation-bug-despite-evidence |
| FND-002 | high | A malformed v4 formula with zero diagnostic budget maps to ResourceIncomplete, although FR-016 requires syntax refusals to map to Unsupported. | src/infinite.rs:213; tests/infinite_v4.rs:238; FR-016-AC-2 | implementation-bug-despite-evidence |

## Review Evidence

At `e7fe85f`, the shared lexer recognizes `{`, `}`, `;` and `:` as tokens for every dialect; at `origin/main`, these were `UnexpectedCharacter`. For example, v1 parsing `{` still refuses, but its diagnostic code and `found` payload change. The new `p` atom tail branch similarly changes `p0x` from proposition plus unknown identifier to one unknown identifier. Existing TC-060 asserts only refusal, not exact legacy report bytes. Preserve legacy lexical reporting while adding v4 tokens, and compare representative v1/v2/v3 report bytes against the base.

`InfiniteParseReport::disposition()` selects the first stored diagnostic and returns `ResourceIncomplete` when none is stored. `parse_clean_ascii_v4("!", InfiniteTraceV1, "event_position", ParseLimits { max_diagnostics: 0, .. })` has a syntax error, no graph and truncated diagnostics, so the method returns `ResourceIncomplete`; the current test explicitly codifies it. Preserve a typed refusal cause independent of diagnostic retention. A later work-limit error can likewise be masked by an earlier syntax diagnostic because only the first code is examined.

The diff has 41 changed files and no `.github/`, `assurance/`, `plan/`, campaign, or aggregate-gate changes. PR #53 is the feature-only candidate separated from mixed draft #52 by path inspection. Reviewed parser/formatter bounds, owner identity, fairness order, exact spans, strict report reading, corpus checks and both real fuzz target bodies. No new vendored upstream source was identified; the only owner reference is the exact Cargo dependency. No changed CI workflow or safety lint was weakened.

Scoped checks at this head: `cargo fmt --check` passed; `cargo clippy --all-targets --all-features --offline -- -D warnings` passed; seven named integration test binaries passed with 69 tests; targeted Quire validation of the five changed spec artifacts reported 5/5 grammar-clean. An initial offline test attempt failed before compilation because Cargo had not fetched `cfc2761`; the normal targeted run fetched exactly that commit and passed. No aggregate gate, `cargo deny`, hosted CI, qualification, or assurance campaign was run.

## Assurance Context

AP-001 (`spec/assurance/AP-001.md`) applies to this exact source/dialect/dependency/corpus candidate: silent reinterpretation and hostile input growth are its material impact scenarios. Evaluated `origin/main` f8ce56211c83eb98a24bd3fdc31792f569da3aa3 against reviewed source e7fe85f506ccfad2eb7835a7a8485505ff04619d and the changed production, test, corpus, documentation and spec paths in scope. The profile's release-owner approval is unavailable and outside this feature review; no exception was claimed. Architecture, measurement, independent assurance, and evidence-producer decisions were not supplied for this candidate. Focused tests and source inspection do not constitute release assurance or a merge gate.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |
| FND-002 | fixed | 26ada2912baa10d4060143d950545bfbbec1d6b9 |
| FND-003 | still-open | Introduced at 877769fa09a9663617bc12fc2f66a04811a41948; three documents restate Cargo's compiled pin. |
| FND-003 | fixed | 0c76e275c550092664b0d233c5e25315def022f5 |

Round 1 reviewed `26ada2912baa10d4060143d950545bfbbec1d6b9`. FND-001 after excerpt: `src/lexer.rs:221-224`: `b'{' if self.dialect == Dialect::V4 => Some(TokenKind::LeftBrace),` (and corresponding `}`, `;`, `:` guards); v1/v2/v3 `p` tails use their prior lexical path.

FND-002 after excerpt: `src/infinite.rs:230-232`: `match self.refusal.map(|refusal| refusal.code) { None if self.document.is_some() => InfiniteDisposition::NoTemporalVerdict, None => InfiniteDisposition::Failed, ... }`; typed cause and locus survive zero diagnostic retention.

Round 1 verdict: PASS for the reviewed TL-14 feature fixes at `26ada2912baa10d4060143d950545bfbbec1d6b9`. All substantive code findings are fixed. The unmerged TL-15 dependency remains a separate hold.

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-003 | high | The compiled TL-15 revision is hardcoded in multiple dialect and attribution documents in addition to Cargo's pin; the new CLI assertion guards this duplicate instead of removing it. | docs/DIALECT-004-clean-ascii-v4.md:18; docs/DIALECT-001-clean-room-mltl-v1.md:24; docs/ATTRIBUTION.md:29; tests/cli.rs:51 | duplicate-authority |

Round 2 reviewed `877769fa09a9663617bc12fc2f66a04811a41948` against prior trailing head `6d67172`. The changed paths were `docs/ATTRIBUTION.md`, `docs/DIALECT-001-clean-room-mltl-v1.md`, `docs/DIALECT-004-clean-ascii-v4.md`, and `tests/cli.rs`. `Cargo.toml` and both Cargo lockfiles remain the compiled dependency authority. A landed TL-15 SHA different from `cfc2761` requires synchronized edits to those authorities plus three prose copies and digest expectations; missing one leaves a false provenance claim. The test at `tests/cli.rs:53` detects some drift but preserves the duplicate. Reference the compiled pin authority instead of restating its SHA in several documents. This is the `/code-review` duplication finding; the Rust test lane and acceptance-to-tests gap check found no separate defect.

Round 2 focused checks: `cargo test --offline --test cli` passed 4/4, `cargo fmt --check` passed, `git diff --check 6d67172..HEAD` passed, and targeted Quire validation of the three changed documents passed 3/3. No `spec/**`, source implementation, fuzz, corpus, CI, or assurance fixture file changed in this round. No aggregate gate was run.

Round 2 verdict: FAIL with FND-003 open. The prior round 1 fixed findings remain fixed.

## Round 3 disposition evidence

Reviewed `0c76e275c550092664b0d233c5e25315def022f5` against `877769fa09a9663617bc12fc2f66a04811a41948`. FND-003 after excerpt (`docs/DIALECT-004-clean-ascii-v4.md:18-20`):

```text
The compiled tl-syntax revision is the landed TL-15 commit selected by
`Cargo.toml` and recorded in `Cargo.lock`. `ATTRIBUTION.md` records its
provenance.
```

`DIALECT-001` and `ATTRIBUTION` likewise no longer restate the compiled SHA. The compiled revision appears in `Cargo.toml`, both Cargo lockfiles, and the public `TL_SYNTAX_REVISION` constant; the CLI test asserts their agreement and rejects the SHA in the three documents. Read-only GitHub PR #93 metadata confirms its merge commit is `6aa9b11e29040d64b437da87c9944e3dedd34a86`, matching the repin. No source-behavior, spec, corpus, fuzz, CI, or assurance fixture file changed beyond the revision constant. The Rust and acceptance-to-tests lanes found no new defect.

Scoped checks at this head: `cargo test --offline --test cli --test infinite_v4 --test owner_infinite_corpus` passed 38/38; `cargo fmt --check`, scoped offline Clippy with `-D warnings`, `git diff --check 877769fa..HEAD`, and targeted Quire validation of the three changed documents passed. No aggregate gate was run.

Round 3 verdict: PASS for the reviewed TL-14 feature and repin. Every substantive FND in this SR file has a latest fixed disposition. The dispatching lead or coder must copy this updated reviewer-owned SR file from scratchpad to `reviews/` in a trailing commit before merging; this reviewer does not modify the frozen branch.
