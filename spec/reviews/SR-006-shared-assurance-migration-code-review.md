---
id: SR-006
title: Code review of the shared assurance migration
type: SpecReview
analysis: code-review
scope: assurance/**, scripts/**, examples/**, tests/shared_assurance.rs, tests/fixtures/legacy-compat/**, Makefile, .github/workflows/ci.yml, Cargo.toml, docs/ATTRIBUTION.md, docs/DIALECT-001-clean-room-mltl-v1.md
review_set: subset
---

# Code review of the shared assurance migration

## Summary

The change moves tl-parse off its repository-local evidence framework and onto
the released Engineering Assurance, Quire, Quoin and ix-flow contracts. The
parser is not the subject: its lexer, precedence, diagnostics, spans, limits,
formatter, corpus, fuzz target and CLI are carried across unchanged and are
re-verified by the same tests plus two new producers.

Three things were designed against rather than discovered, because the sibling
migration in `tl-syntax` shipped them and an adversarial review had to find
them: attestations that state a hoped-for verdict instead of reading the
producer's bytes, a producer-isolation test that cannot fail, and an absent
tool being given a fabricated version.

## Verdict

**CONDITIONAL.** No high findings. Two mediums and three lows are recorded
below; the mediums are inherent-boundary statements rather than defects, and
are dispositioned in SR-008 after the independent adversarial round.

## What was preserved, and how that was checked

`git diff cf43f40 HEAD -- evidence/` is empty: all 709 retained files across 12
record directories are byte-identical. The two evidence schemas are frozen
rather than deleted because all 12 envelopes name them by SHA-256; TC-028 pins
both digests and censuses the source tree to prove nothing references them.

Parser-domain behaviour is re-established rather than assumed:

| Property | Evidence at this head |
| --- | --- |
| Round-trip fixed point | 40,000 checks (20,000 deterministic sources × 2 profiles), 0 drift |
| Round-trip non-vacuity | parenthesis-stripping control breaks 2,390 of 2,390 parenthesised cases |
| Corpus conformance | 7/7 fixtures, 1 `pass` + 6 `malformed`, every declared diagnostic code and span matched |
| Clean-room provenance | 4/4 digests re-derived from the resolved tl-syntax checkout |
| Test census | 35 requirement-tagged compiled tests, 0 ignored |
| Fuzz | target compiles; bounded smoke ran 64 runs with LeakSanitizer enabled |

## The design decisions worth reviewing

**Every attested result is read from producer bytes.** `derive_result()` reads a
field the producer wrote: row outcomes for the two domain streams, `entries` for
the census, `matched` for attribution and compatibility, a populated-document
check for the Quire export, and cargo's own `build-finished` message for MSRV.
`--message-format=json` is on the MSRV producer specifically so its verdict is a
field rather than a sentence. Probe: setting every producer output to failure
takes the chain to exit 2, and flipping only `build-finished.success` takes it
to exit 1.

**`malformed` maps to a passing proof, and this is the one deliberate
divergence from tl-syntax.** In that repository `malformed` meant the
producer's own row was malformed, which is a defect. Here it means the input
fixture was malformed and the parser said so, which is the parser working — and
six of seven corpus fixtures are malformed by design, so mapping it to `failed`
would report a permanently failing proof for a permanently correct parser.

The state is not thereby collapsed. Three chain scenarios hold it apart, and the
count is checked against the corpus manifest rather than against the producer's
own output, so a producer that stopped reporting malformed rows cannot also move
the number it is compared to. Probes: rewriting every `malformed` row to `pass`
gives exit 1; changing the manifest's malformed count gives exit 1.

**The producer boundary is asserted behaviourally, with a control.** Run A stubs
`cargo`, `rustup` and `rustc` with shims that log every invocation and requires
the log to be empty. Run B stubs `quoin` and requires the chain to *fail* and
the log to be non-empty — because an empty log and an unconsulted `PATH` are
otherwise the same observation. Asking a tool its version is allowed and is not
logged; asking it to do work is.

**An unobservable tool version is refused, not defaulted.** `tool_version()`
returns `None`, recorded as `null`, and `observe_tool_versions()` raises rather
than sealing an attestation naming a version nobody measured.

**A refusal is reported as a refusal.** All 12 retained envelopes are
`quire.derivation-evidence/v1`; the pinned mapping covers `quire.pgm01-evidence`
v1 and v2 and answers `incompatible` for every one. No local mapper was written
to make that read as a pass. `agent-ix/engineering-assurance#21`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-601 | medium | Deleting the Make execution-control guard leaves the false-success class uncovered for `ci` prerequisites that are pure gates with no retained output — `fmt-check`, `lint`, `deny`, `audit-unsafe`, `rustdoc`. The structural replacement (Quoin binds inputs by digest) covers only the producer targets. Filed as `agent-ix/tl-parse#11` and recorded as an open unknown in the declaration | `Makefile`, `assurance/change-assurance.json` |
| FND-602 | medium | The chain consumes what `make assurance-inputs` wrote and cannot verify that Make ran the command it printed. Inherent: something must run the producer and Quoin's contract is that the caller states the result. What is true is that the caller states what the bytes say | `Makefile`, `scripts/assurance_chain.py` |
| FND-603 | low | Two compatibility cases return `unreadable` under the real record id and are separable only by the upstream reason string. Asserted as such and stated under `discrimination` in `expectations.json` rather than resolved by re-deriving a discriminator locally | `tests/fixtures/legacy-compat/expectations.json` |
| FND-604 | low | Four of nine suite-registry rows are unbacked: `make ci`, `quire validate`, `rustdoc`, and the manual hosted dispatch. No test invokes them and the only way to make them read as backed is a source-text grep, which is the binding shape PR #6's reviewers rejected. Stated in `spec/evidence/suites.md` | `spec/evidence/suites.md` |
| FND-605 | low | `structural()` compares profile, root and ordered node kinds, deliberately excluding source spans, because spans legitimately differ between a source and its canonical rendering. This is the same comparison `tests/property.rs` uses. A formatter that permuted spans without changing kinds would not be caught here; the span tests in `tests/parser.rs` own that | `examples/*.rs` |

## Mutation probes

Nineteen. One was initially not detected, and it is listed with both results
because a probe table that shows only the final state is a table written after
the fix.

| Probe | Gate | First result | Now |
| --- | --- | --- | --- |
| Every producer output set to `fail` | chain | exit 2, detected | unchanged |
| Malformed rows rewritten to `pass` | chain | exit 1, detected | unchanged |
| Producer input removed | chain | exit 2, names the target | unchanged |
| Quire export replaced with `{}` | chain | exit 1, detected | unchanged |
| MSRV `build-finished.success` set false | chain | exit 1, detected | unchanged |
| Adapter protocol check removed | chain | exit 1, detected | unchanged |
| Adapter outcome map collapsed to all-pass | chain | exit 1, detected | unchanged |
| Empty-stream refusal removed | chain | exit 1, detected | unchanged |
| Dangling `pairs_with` introduced | chain | **NOT detected** | exit 2, detected |
| Corpus manifest malformed count changed | chain | exit 1, detected | unchanged |
| `npm.ix` in a requirement | pins | exit 1, detected | unchanged |
| Wrong consumed-artifact digest | pins | exit 1, detected | unchanged |
| Upstream pin disagreement (`lib.rs` vs `Cargo.toml`) | pins | exit 1, detected | unchanged |
| Retained evidence byte altered | compat-view | exit 1, detected | unchanged |
| Derived fixture hand-edited | compat-view | exit 2, detected | unchanged |
| Attribution digest doctored | attribution | exit 1, detected | unchanged |
| Authorship basis rewritten to the compiled digest | attribution | exit 1, detected | unchanged |
| Traced test marked `#[ignore]` | test-census | exit 1, detected | unchanged |
| Corpus fixture byte altered | check-corpus | exit 1, detected | unchanged |

Plus the five compatibility probes the view runs itself: collapse non-success
states, repair an unreadable outcome, accept a refused schema, unbind the tamper
digest, drop the source identity — 5/5 detected, with no exception handling that
could inflate the count.

**The probe that was initially not detected was a broken probe, not a broken
guard, and that distinction is the finding.** The first version renamed *both*
the control's `pairs_with` and the scenario's own name, so the pairing stayed
consistent and the guard correctly said nothing. Renaming only the `pairs_with`
gives exit 2 with `these controls name a scenario that does not exist:
['verify-accepts-an-unedited-receipt']`. A probe that mutates two things at once
can pass for a reason that has nothing to do with the gate.

## Gates at this head

| Gate | Result |
| --- | --- |
| `make ci` | exit 0 |
| `make spec` | exit 0 |
| Rust tests | 35 passed, 0 failed, 0 ignored |
| Quire coverage | 68/72 rows backed (94%); `spec/test-matrix.md` 28/28; rust 36/36/36 |
| Quire validate `--strict` | 42/42 docs grammar-clean, 0 findings |
| Shared pins | 4/4 compatible, 0 artifact mismatches, 0 mirror references, 0 upstream pin disagreements |
| Compatibility census | 11/11 cases, 12 envelopes, 709 files read, 0 bytes moved this run, 0 uncommitted, 2 positive controls accepted |
| Compatibility mutation probes | 5/5 detected |
| Assurance chain | 17 scenarios, 7 controls, 7 adapter probes, all matched |
| Attested results | all seven proofs `passed`, each read from the producer's own output |
| Audited receipt | `incomplete`, reasons `decision_missing` and `unresolved_unknown` |
| `git diff cf43f40 HEAD -- evidence/` | empty |
| Hosted CI | not dispatched |
