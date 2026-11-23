---
id: PLAN-015
title: Numeric dialect v5 plan (tl-parse)
type: Plan
status: draft
relationships:
  - target: ix://agent-ix/tl-parse/FR-028
    type: references
  - target: ix://agent-ix/tl-parse/FR-029
    type: references
  - target: ix://agent-ix/tl-parse/FR-030
    type: references
  - target: ix://agent-ix/tl-parse/FR-031
    type: references
  - target: ix://agent-ix/tl-parse/FR-032
    type: references
  - target: ix://agent-ix/tl-parse/FR-033
    type: references
  - target: ix://agent-ix/tl-parse/NFR-016
    type: references
---

# Implementation Plan: numeric dialect v5 (tl-parse)

## Requirements Summary

- [ ] **FR-028..FR-031, FR-033, NFR-016** (TL-421): dialect, grammar, diagnostics, format, corpus and fuzz.
- [ ] **FR-032** (TL-424 and TL-425 parse half): aggregation and contract forms.

## Dependency Graph

Layers follow `ix-board project-chunk P-TL-150`:

| Layer | Tickets | tl-parse tasks |
|---|---|---|
| 0 | TL-419 | this specification |
| 1 | TL-420, TL-425 | upstream tl-syntax model (must exist before Task-001) |
| 2 | TL-421 | Task-001, Task-002, Task-004 |
| 3 | TL-424 | Task-003 (aggregation parse needs tl-syntax expansion) |

- `Task-001 -> Task-002` (format needs the parsed model).
- `Task-001 -> Task-003`, `Task-001 -> Task-004`.
- Contract text parsing in Task-003 depends only on the tl-syntax contract set (layer 1) and may start with layer 2.

## Quality Gates

**Gate 1**: v1..v4 bytes unchanged (TC-1541) before any other task merges.
**Gate 2**: fixed point (TC-1550) and hostile corpus (TC-1555) before release.
**Gate 3**: fuzz smoke (TC-1556) once at the final head, with the full gate.

## Execution Tracks

- **Track A**: Task-001 -> Task-002 -> Task-004.
- **Track B**: Task-003 (after Task-001 and the tl-syntax aggregation task).

## Test Plan

`spec/numeric-test-matrix.md` is authoritative; no ids are added here.

## Deferred verification

| Item | Why deferred | Evidence needed to close |
|---|---|---|
| Dialect identity `v5` versus the plain LTL dialect v5 of P-TL-151 | Two projects claim one identity; a human decision (ADR-020) | One project renumbered to v6 and recorded |
| All executable tests and `make ci` | No build capacity during authoring | Local gate at implementation head |
| Longer libFuzzer campaign | Supplementary evidence, needs capacity | Retained campaign record under `fuzz/evidence/` |
| Miri run of the new corpus | No capacity | `make miri` at head |
| Dialect identity (decided) | TL program decision 2026-11-23 item 2: numeric keeps `v5`, plain LTL takes `v6` | ADR-020 "Open decision" text updated to record it |
| TL-421 UNBUILT: no `cargo`/`rustc`/`make` was run on any TL-421 code (branch `TL-code-unbuilt`) | Program rule for unbuilt code | `make ci` at the TL-421 head after the tl-syntax pin below lands |
| UNBUILT-PIN tl-syntax rev | `Cargo.toml` pins tl-syntax origin/main `a80d09e` and a `[patch]` (root and `fuzz/`) points at the sibling `../TL-code-tl-syntax` worktree; Cargo.lock and fuzz/Cargo.lock not regenerated | Merge TL-420, pin its rev, delete both `[patch]` sections, regenerate locks |
| tl-syntax API (all in `src/numeric/model.rs`) | Written against tl-syntax `TL-code-unbuilt` (`ExprNode`, `ExprId`, `Predicate` fields, `PredicateTableDocument::new(SignalCatalog, ..)`, `ExpressionError`/`ExpressionLimit`), which is still changing | Re-read those signatures at the merged rev |
| TC-1540 FR-028-AC-1 | unbuilt | `cargo test --test numeric_v5 dialect_profile_and_catalog_are_explicit_and_refused_before_lexing` |
| TC-1541 FR-028-AC-2 | unbuilt; byte identity checked against pre-change copies in `tests/fixtures/pre-v5` (no digest literals, decision 7). v1..v4 code was touched only to add the six tl-syntax LTLf `NodeKind` arms in `src/formatter.rs::operands` and `tests/parser.rs` (behaviour-neutral) | `tests/existing_dialects_unchanged.rs` plus the v1..v4 suites green |
| TC-1542 FR-028-AC-3 | unbuilt | `past_unbounded_and_fairness_are_unsupported_with_a_locus` |
| TC-1543 FR-029-AC-1 | unbuilt; W/M corpus cases assert only the text fixed point (lowered graphs share operand nodes, so re-parse is not node-identical) | `allocation_is_first_occurrence_above_explicit_and_bound_identities` and corpus replay |
| TC-1544 FR-029-AC-2 | unbuilt | three `numeric_v5` unit tests and `src/numeric/lexer.rs` unit tests |
| TC-1545 FR-029-AC-3 | unbuilt | proptest `arbitrary_bytes_never_panic_and_yield_graph_xor_refusal` |
| TC-1546 FR-030-AC-1 | PARTIAL: closed-set test only; `undefined_set`, `duplicate_set`, `slot_outside_template`, `aggregation_too_large`, `duplicate_contract`, `missing_contract_clause` have no v5 producer until TL-424/TL-425 | minimal producers with TL-424/TL-425 |
| TC-1547 FR-030-AC-2 | unbuilt | `more_than_sixty_four_diagnostics_keep_the_first_and_the_decisive_refusal` |
| TC-1548 FR-030-AC-3 | unbuilt; the reader now takes the catalog and re-types the table through `PredicateTableDocument::new` from a report-local mirror | `the_report_reader_is_strict` |
| TC-1549 FR-031-AC-1, TC-1559 StR-014-VC-1 | unbuilt | corpus round trip test |
| TC-1550 FR-031-AC-2 | unbuilt | proptest fixed point |
| TC-1551 FR-031-AC-3 | unbuilt | `unrepresentable_graphs_are_refused_with_no_text` |
| TC-1552..TC-1554 FR-032 | not TL-421 scope (TL-424, TL-425) | parse half of TL-424/TL-425 |
| TC-1555 FR-033-AC-1 | unbuilt; 83 hand-maintained cases (the Python generator was removed: Rust-only repository); every manifest span was independently byte-checked by the review, expectations traced but not run | `hostile_corpus_covers_every_class_and_matches_the_parser`; aggregation classes (k = 1,024/1,025, at-least > 100,000 nodes, empty/duplicate sets, duplicate/half contracts) join with TL-424/TL-425 |
| TC-1556 FR-033-AC-2 | unbuilt; `fuzz/Cargo.lock` not regenerated | `make fuzz-build fuzz-smoke` |
| TC-1557 FR-033-AC-3, TC-1560 StR-014-VC-2 | unbuilt | `hostile_corpus_stays_within_the_hard_maxima` |
| TC-1558 NFR-016-AC-1 | unbuilt; Miri list extended with `--test numeric_v5` but not run | `repeated_runs_are_byte_identical`, `make miri` |
| FR-031 fixed point vs `W`/`M` (SPEC-GAP) | Lowering shares operands; the text is a tree, so about 13 chained `W[0,1]` give text whose re-parse exceeds `HARD_MAX_NODES`. The formatter refuses such graphs as `unrepresentable_graph`, and the fuzz target accepts only that class and resource refusals | Spec decision: amend FR-031 or define DAG-preserving text |
| Expression depth (review H-3) | Chained `+`/`*` add owner expression depth without parser nesting; the local typing mirror now refuses at depth 257 with the sub-expression locus | `chained_arithmetic_counts_toward_the_expression_depth_limit`, corpus `arith-chain-255/256` |
