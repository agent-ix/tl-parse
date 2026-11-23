---
id: Task-004
title: "Hostile corpus and fuzz target"
type: Task
status: todo
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/FR-033
    type: references
  - target: ix://agent-ix/tl-parse/NFR-016
    type: references
  - target: ix://agent-ix/tl-parse/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-parse/TC-1555
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-1556
    type: verifies
---
# Task-004: Hostile corpus and fuzz target

## Scope

FR-033 corpus numeric-v5, numeric_v5_roundtrip target, smoke runner and Makefile entries; NFR-016.

## Subtasks

- [ ] Corpus and manifest with required classes.
- [ ] Fuzz target and seeds; fuzz-smoke and Miri lists.

## Deliverables

Corpus, fuzz target, tests TC-1555..TC-1558.

## Notes

Specification-only session: no code was written or run.
