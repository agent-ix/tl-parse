---
id: SR-008
title: Closing code review — shared assurance migration
type: SpecReview
analysis: code-review
scope: "PR #12; SR-006 findings FND-601..FND-605; the independent adversarial review's sixteen findings; exact-head gates"
review_set: subset
---

# SR-008: Closing code review — shared assurance migration

## Summary

An independent adversarial review was run against `6430e1b` with a single
instruction: find false greens. It found sixteen, two of them high, and the most
useful thing about both highs is that they were **claims this change made about
itself**, not defects in the code it added. SR-006 had graded one of them medium
and stated the other as a fact.

The first high: every document that described the deleted Make execution-control
guard said the replacement was structural — a recipe that did not run yields an
absent input the chain names. That is true of producers and false of everything
else. `.IGNORE:` neuters ten of the sixteen `ci` prerequisites and every
remaining check stays green.

The second high: `spec/evidence/suites.md` said five suites are backed by a test
that "actually invokes that suite's command". None of them does.

## Verdict

**CONDITIONAL.** All sixteen adversarial findings and all five SR-006 findings
are dispositioned below: fourteen FIXED, three ACCEPTED with rationale, four
DEFERRED to filed issues.

## What the adversarial review changed

**Two false claims, corrected rather than defended.** Both highs are prose. The
temptation with a prose finding is to argue that the code is fine, and the code
*is* fine — the gap in the first was already disclosed and the binding in the
second is architecturally correct. But a reader of NFR-003 would have concluded
the class was covered, and a reader of the suite registry would have concluded
tests run those commands. Neither was true, and both now say what is.

**Five measurement defects that would have read green.** The twelve-state census
counted a free-text `kind` label from `expectations.json` rather than a measured
outcome, so removing `suspect` from the chain and re-adding the word to a fixture
label left the test passing. The Quire export verdict asked only whether any
nested value was non-empty — true of every real export — so one reporting 0 of 72
rows backed attested `passed`. The round-trip sweep reported `pass` regardless of
how many sources the parser rejected. The frozen-schema census walked seven named
directories, so a validator in `assurance/` was invisible. `PROOF-msrv` sealed
ambient cargo's version while declaring `rustup run 1.75.0 cargo check`.

**Two guards claimed as verified and exercised by nothing.** NFR-003-AC-3 said a
control naming a non-existent scenario is refused, verified by TC-026; nothing
tested it. `mirror_references`'s file-scan branch had no control while its
structural branch did. Both now have tests.

## Findings

Residual after this round.

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-801 | high | `.IGNORE:` makes `fmt-check`, `lint`, `test`, `check-corpus`, `fuzz-build`, `fuzz-smoke`, `deny`, `audit-unsafe`, `rustdoc` and the `quire validate` half of `spec` report success without running. Not closed here: the migration contract removes this machinery and reintroducing a guard would contradict it. Every claim about it is corrected and `agent-ix/tl-parse#11` carries the reproduction | `Makefile` | correct-requirement-no-evidence |
| FND-802 | medium | Nothing binds `target/assurance/*` to the working tree, so breaking `docs/ATTRIBUTION.md` leaves a chain run over stale bytes green. Inherent to the producer/consumer split: something must run the producer, and freshness is Make's job. `make ci` always regenerates | `Makefile`, `scripts/assurance_chain.py` | correct-requirement-no-evidence |
| FND-803 | low | The sealed snapshot's `unbacked_rows: []` is vacuous because Quire's status classification is skipped upstream. Not this repository's to fix; `agent-ix/quire-contract-ir#21` | `spec/test-matrix.md` | wrong-requirement |

## Dispositions

### SR-006 findings

| ID | Severity | Disposition | Where |
| --- | --- | --- | --- |
| FND-601 Make guard removal | medium → **high** | **FIXED as a claim, DEFERRED as a gap**, `agent-ix/tl-parse#11` | Regraded by the adversarial round, which demonstrated it. The claim is corrected in four places; the gap stands by contract |
| FND-602 chain cannot verify Make ran the command | medium | **ACCEPTED** | Inherent. What is true is that the caller states what the bytes say |
| FND-603 two compat cases separable only by reason text | low | **ACCEPTED** | Stated under `discrimination` in `expectations.json`. Re-deriving a discriminator means writing the mapping this migration removed |
| FND-604 four suite rows unbacked | low | **FIXED**, differently than written | The count was right and the description was wrong; see adversarial 2 |
| FND-605 `structural()` excludes spans | low | **ACCEPTED** | The adversarial round confirmed it sound: `NodeKind` carries intervals, proposition ids and operand `NodeId`s, so only the span is dropped |

### Adversarial findings

| # | Severity | Disposition | Where |
| --- | --- | --- | --- |
| 1 structural-replacement claim false for 10 of 16 gates | **high** | **FIXED (claim)**, gap DEFERRED to `agent-ix/tl-parse#11` | `NFR-003`, `AA-001`, `assurance/change-assurance.json`, `Makefile` header all name the uncovered targets explicitly |
| 2 "actually invokes that suite's command" false for all five | **high** | **FIXED** | `spec/evidence/suites.md` carries a per-suite table of what each test really does; SR-007 records the correction |
| 3 twelve-state census counts a label | medium | **FIXED** | TC-026 reads `mapped_states` from matched cases and asserts the census's own `undemonstrated_states`/`undemonstrated_outcomes` are empty |
| 4 schema census walks 7 directories | medium | **FIXED** | Walks the whole tree except `evidence/`, `target/`, `.git`, `.venv-assurance`, with a three-file allowlist |
| 5 Quire export cannot report anything but passed | medium | **FIXED** | `not_computed` on absent totals or zero backed; `failed` on a status lie; TC-024 pins 68/72. Probes: zero-backed → exit 1, status lie → exit 1 |
| 6 nothing binds producer output to the tree | medium | **ACCEPTED**, recorded as FND-802 | Inherent to the split; `make ci` regenerates |
| 7 `PROOF-msrv` seals ambient cargo's version | medium | **FIXED** | Observed via `rustup run 1.75.0 cargo --version`. The isolation shims now answer a version query anywhere in argv, and the test still fails on `cargo build` |
| 8 roundtrip ignores rejected sources | medium | **FIXED** | `vacuous` when fewer than half the generated sources reach comparison |
| 9 `unbacked_rows` vacuous upstream | low | **DEFERRED**, `agent-ix/quire-contract-ir#21` | Recorded as FND-803 |
| 10 empty JSON gives a traceback at exit 1 | low | **FIXED** | `_load_json` names the file and the target; exit 2, distinct from a mismatch |
| 11 dangling-control guard untested | low | **FIXED** | New test renames only the `pairs_with`; guard exits 2 naming the orphan |
| 12 "only file permitted to name them" false | low | **FIXED** | `schemas/README.md` names the three allow-listed files |
| 13 producers always exit 0 | low | **FIXED** | Both return `ExitCode::FAILURE` on a failing row, so `make conformance` and `make roundtrip` are gates rather than print statements |
| 14 mirror file-scan branch has no control | low | **FIXED** | New test writes a mirror reference into `requirements-assurance.txt`, requires detection, and restores the file |
| 15 control threshold weaker than its docstring | low | **FIXED** | Requires the control to break at least a quarter of parenthesised cases |
| 16 venv has no prerequisite on the pin | low | **FIXED** | `$(ASSURANCE_PYTHON): requirements-assurance.txt`, rebuilt from scratch |

### Accepted without change

- **Adversarial 6 / FND-802.** The chain attests to bytes a producer wrote. It
  cannot also certify that those bytes are current, because something has to run
  the producer and Quoin's contract is that the caller states the result.
- **SR-006 FND-602.** The same boundary from the Make side.
- **SR-006 FND-603.** Two compatibility cases separable only by upstream reason
  text.

## Exact-head gates

Run at the final implementation head, not carried from SR-006.

| Gate | Result |
| --- | --- |
| `make ci` | exit 0 |
| `make spec` | exit 0 |
| Rust tests | 37 passed, 0 failed, 0 ignored |
| Quire coverage | 68/72 rows backed (94%); `spec/test-matrix.md` 28/28; rust 38/38/38 |
| Quire validate `--strict` | 44/44 docs grammar-clean, 0 findings |
| Shared pins | 4/4 compatible, 0 artifact mismatches, 0 mirror references, 0 pin disagreements |
| Compatibility census | 11/11 cases, 12 envelopes, 709 files read, 0 bytes moved, 0 uncommitted, 2 positive controls accepted |
| Compatibility mutation probes | 5/5 detected |
| Assurance chain | 17 scenarios, 7 controls, 7 adapter probes, 0 mismatches |
| Attested results | all seven proofs `passed`, each read from the producer's own output |
| Audited receipt | `incomplete` — `decision_missing`, `unresolved_unknown` |
| Round-trip sweep | 40,000 checks, 0 drift; control breaks 2,390/2,390 |
| Corpus conformance | 7/7 — 1 `pass`, 6 `malformed` |
| Fuzz | 64-run bounded smoke with LeakSanitizer; separate 181 s campaign, 1,237,541 runs, 0 crashes |
| `git diff cf43f40 HEAD -- evidence/` | empty |
| Hosted CI | not dispatched |

## Mutation probes, closing set

Twenty-four. Twenty-three detect.

| Probe | Gate | Result |
| --- | --- | --- |
| Every producer output set to `fail` | chain | exit 2 |
| Malformed rows rewritten to `pass` | chain | exit 1 |
| Producer input removed | chain | exit 2, names the target |
| Quire export replaced with `{}` | chain | exit 1 |
| Quire export doctored to 0 backed | chain | exit 1 |
| Quire export given a status lie | chain | exit 1 |
| MSRV `build-finished.success` false | chain | exit 1 |
| Empty producer JSON | chain | exit 2, names the target |
| Adapter protocol check removed | chain | exit 1 |
| Adapter outcome map collapsed | chain | exit 1 |
| Empty-stream refusal removed | chain | exit 1 |
| Corpus manifest malformed count changed | chain | exit 1 |
| `npm.ix` in a requirement | pins | exit 1 |
| Wrong consumed-artifact digest | pins | exit 1 |
| Upstream pin disagreement | pins | exit 1 |
| Retained evidence byte altered | compat | exit 1 |
| Derived fixture hand-edited | compat | exit 2 |
| Attribution digest doctored | attribution | exit 1 |
| Authorship basis rewritten to compiled | attribution | exit 1 |
| Attribution stale against the tree | attribution | exit 1 |
| Traced test marked `#[ignore]` | census | exit 1 |
| Corpus fixture byte altered | check-corpus | exit 1 |
| Round-trip control made unable to fire | roundtrip | exit 1 |
| Dangling `pairs_with` (both names renamed) | chain | **not detected — broken probe** |

The last row is recorded rather than dropped. Renaming both the control's
`pairs_with` and the scenario's own name leaves the pairing consistent, so the
guard correctly says nothing; the probe mutates two things and proves neither.
Renaming only the `pairs_with` gives exit 2, and
`a_control_naming_a_scenario_that_does_not_exist_is_refused` now does exactly
that as a committed test. A probe that changes two things at once can pass for a
reason unrelated to the gate.

Plus the five compatibility probes the view runs itself, 5/5, with no exception
handling that could inflate the count.

## Reviewer feedback

No GitHub review has been received on PR #12. `mergeStateStatus` is `BLOCKED`
with `reviewDecision: REVIEW_REQUIRED`, which is the CODEOWNERS requirement: the
same account authored the change and cannot approve it. The adversarial review
recorded above was run independently against the exact head and its findings are
dispositioned here rather than discarded.
