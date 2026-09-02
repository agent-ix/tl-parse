---
id: Task-004
title: "Canonical formatter and round trips"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-001
    type: part_of
---
# Task-004: Canonical formatter and round trips

## Scope

Format validated graphs iteratively into one explicit representation within
hard-clamped output and logical-work limits.

## Completion Evidence

Exact operator fixtures, idempotence, generated both-profile round trips,
shared/deep graph traversal, and output/work exhaustion tests pass.
