---
id: SR-043
title: "Gap analysis — clean-ascii/v3 past parsing"
type: SpecReview
analysis: gap-analysis
scope: "Task-002, FR-013-AC-2, TC-055, parser portion of TC-057"
review_set: subset
---

# Gap analysis — clean-ascii/v3 past parsing

## Summary

Task-002 is complete in the candidate. The public API has closed v3 report,
dialect, profile, dependency-revision, and formatter identities; all five
operators parse to the exact formula-v2 nodes and spans; precedence and left
associativity are fixed; formatting reaches a parse/format/parse fixed point;
old dialects still reject past spellings; and bounded arbitrary UTF-8 cannot
unwind or cross into a future profile. There is no second temporal AST or
semantic evaluator in tl-parse.

## Verdict

**PASS for Task-002** — no implementation, behavior, or traceability gap
remains in the parser task. Tasks 003 through 007 remain owned by their planned
repositories and are not represented as complete here.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4301 | medium | The first wire-contract test did not prove that a failed report requires an observable diagnostic state or that document spans remain inside the submitted source. Fixed in the decoder and mutation matrix. | `src/past.rs`, `tests/clean_ascii_v3.rs`, TC-055, TC-057 |
| FND-4302 | medium | The first long-identifier handling for strong Previous did not preserve the exact token span promised by TC-055. Fixed with a closed lexical boundary and `Yesterday` control. | `src/lexer.rs`, `tests/clean_ascii_v3.rs`, TC-055 |
| FND-4303 | low | Quire reconciliation is repository-local: tl-parse reports the upstream FR-013/TC-055/TC-057 tags as untracked, while tl-syntax reports TC-055 as unbacked because it cannot see downstream symbols. The accepted Task-002 relationships and the exact tagged Rust symbols reconcile the two sides without inventing a duplicate local requirement. | `tl-syntax:Task-002`, `tl-syntax:FR-013-AC-2`, `tests/clean_ascii_v3.rs` |
| FND-4304 | low | The host tool versions differ from the repository's shared-assurance pins. The mismatch is external to Task-002 and remains a fail-closed non-result, not deferred implementation work. | `assurance/pins.json`, `tests/shared_assurance.rs` |

## Coverage

- Task slice: 1/1 Task-002 deliverable implemented; the four task subtasks are
  exercised by the candidate and its gates.
- TC-055: eight traced integration/property tests cover identity, all five
  nodes/spans, precedence, associativity, formatting, refusals, resources,
  strict wire behavior, and stable complete diagnostics.
- TC-057 parser half: five traced tests cover malformed/resource failures,
  strict report decoding, diagnostic stability, and 96-case bounded arbitrary
  UTF-8/property runs without unwind or profile misattribution.
- Compiled census: 66/66 requirement-tagged Rust tests matched compiled tests.
- Reverse inventory: two new public behavior entry points plus four closed wire
  identity types/constants; all are owned by accepted FR-013-AC-2/Task-002 and
  exercised through the public API. No source stub, test stub, unsafe path, or
  unowned temporal semantics was found.
- Semantic agreement was included in the requested code/gap self-review: the
  exact accepted grammar/profile clauses were compared with the public tests
  and parser/formatter branches. All three axes (intent, exercised code, and
  implementation behavior) agree for FR-013-AC-2.
