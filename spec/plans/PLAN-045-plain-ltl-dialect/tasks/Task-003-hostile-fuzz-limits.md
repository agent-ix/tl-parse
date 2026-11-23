---
id: Task-003
title: "Hostile corpus, fuzz target and limits"
type: Task
status: blocked
track: B
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-045
    type: part_of
  - target: ix://agent-ix/tl-parse/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-061
    type: references
  - target: ix://agent-ix/tl-parse/NFR-046
    type: references
  - target: ix://agent-ix/tl-parse/TC-120
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-121
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-122
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-125
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-126
    type: verifies
---

# Task-003: Hostile corpus, fuzz target and limits

## Scope

29-case corpus/v5, clean_ascii_v5 fuzz target, limit and determinism tests.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- corpus/v5 manifest, fuzz target, seeds

## Notes

- Seeds are the 29 cases.
