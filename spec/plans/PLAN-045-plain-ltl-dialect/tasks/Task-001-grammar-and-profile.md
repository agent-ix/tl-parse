---
id: Task-001
title: "v5 grammar and explicit profile selection"
type: Task
status: not_started
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-045
    type: part_of
  - target: ix://agent-ix/tl-parse/FR-058
    type: references
  - target: ix://agent-ix/tl-parse/FR-059
    type: references
  - target: ix://agent-ix/tl-parse/TC-112
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-113
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-114
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-115
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-116
    type: verifies
  - target: ix://agent-ix/tl-parse/TC-117
    type: verifies
---

# Task-001: v5 grammar and explicit profile selection

## Scope

Add dialect::v5, LtlProfile, parse_clean_ascii_v5 and the 120/116 depth-one acceptance tests.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- dialect module, record, digest, DIALECT-005 document

## Notes

- Depends on tl-syntax TL-428 landing.
