---
id: Task-002
title: "Canonical formatter and diagnostics"
type: Task
status: todo
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/FR-030
    type: references
  - target: ix://agent-ix/tl-parse/FR-031
    type: references
  - target: ix://agent-ix/tl-parse/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-parse/TC-1546
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-1549
    type: verifies
---
# Task-002: Canonical formatter and diagnostics

## Scope

FR-030 numeric diagnostics schema and FR-031 canonical formatter with fixed point.

## Subtasks

- [ ] Diagnostics vocabulary and reader.
- [ ] Formatter and unrepresentable_graph refusal.

## Deliverables

Formatter, schema, tests TC-1546..TC-1551.

## Notes

Specification-only session: no code was written or run.
