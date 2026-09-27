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

Round 1 reviewed `26ada2912baa10d4060143d950545bfbbec1d6b9`. FND-001 after excerpt: `src/lexer.rs:221-224`: `b'{' if self.dialect == Dialect::V4 => Some(TokenKind::LeftBrace),` (and corresponding `}`, `;`, `:` guards); v1/v2/v3 `p` tails use their prior lexical path.

FND-002 after excerpt: `src/infinite.rs:230-232`: `match self.refusal.map(|refusal| refusal.code) { None if self.document.is_some() => InfiniteDisposition::NoTemporalVerdict, None => InfiniteDisposition::Failed, ... }`; typed cause and locus survive zero diagnostic retention.

Round 1 verdict: PASS for the reviewed TL-14 feature fixes at `26ada2912baa10d4060143d950545bfbbec1d6b9`. All substantive code findings are fixed. The unmerged TL-15 dependency remains a separate hold.
