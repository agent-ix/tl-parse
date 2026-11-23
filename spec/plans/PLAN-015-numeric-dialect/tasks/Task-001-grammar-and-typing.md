---
id: Task-001
title: "Lexer, grammar and typing for numeric predicates"
type: Task
status: todo
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/FR-028
    type: references
  - target: ix://agent-ix/tl-parse/FR-029
    type: references
  - target: ix://agent-ix/tl-parse/TC-1540
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-1543
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-1541
    type: verifies
---
# Task-001: Lexer, grammar and typing for numeric predicates

## Scope

FR-028 dialect selection and FR-029 grammar, typing through tl-syntax, proposition allocation.

## Subtasks

- [ ] dialect/v5 module; identifier and literal lexing.
- [ ] Typing diagnostics mapping.
- [ ] v1..v4 non-interference.

## Deliverables

Parser path and tests TC-1540..TC-1545.

## Notes

Specification-only session: no code was written or run.
