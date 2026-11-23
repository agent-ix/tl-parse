---
id: Task-004
title: "v1 to v4 unchanged proof"
type: Task
status: not_started
track: B
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-045
    type: part_of
  - target: ix://agent-ix/tl-parse/FR-062
    type: references
  - target: ix://agent-ix/tl-parse/TC-123
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-124
    type: verifies
---

# Task-004: v1 to v4 unchanged proof

## Scope

Pin pre-change digests and retained outputs of v1 to v4 and assert refusal of bracket-free text.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- digest pins and tests

## Notes

- Capture the digests before any v5 code lands.
