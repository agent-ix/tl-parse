---
id: Task-002
title: "v5 formatter and round trip"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-045
    type: part_of
  - target: ix://agent-ix/tl-parse/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-060
    type: references
  - target: ix://agent-ix/tl-parse/TC-118
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-119
    type: verifies
---

# Task-002: v5 formatter and round trip

## Scope

format_clean_ascii_v5, NotExpressible refusal, 236 + 10 round-trip population.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- formatter and property tests

## Notes

- Image rules in FR-060.
