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
| All executable tests and `make ci` | No build capacity during authoring | Local gate at implementation head |
| Longer libFuzzer campaign | Supplementary evidence, needs capacity | Retained campaign record under `fuzz/evidence/` |
| Miri run of the new corpus | No capacity | `make miri` at head |
