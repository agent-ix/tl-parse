---
id: Task-006
title: "Exact-candidate evidence"
type: Task
status: in_progress
track: Evidence
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-001
    type: part_of
  - target: ix://agent-ix/tl-parse/MP-001
    type: references
---
# Task-006: Exact-candidate evidence

## Scope

Retain the exact clean revision's local outcomes, tools, dependency/dialect/
corpus identities, limits, PGM-01 checks, and limitations in a checksummed
record; then confirm that finalized PR revision with one manual hosted run.

## Completion Evidence

Pending. Completion requires passing post-seal PGM-01 validations, a checksum
manifest covering every artifact, a clean feedback audit, and the single
deliberate hosted `workflow_dispatch` result for the finalized revision.
