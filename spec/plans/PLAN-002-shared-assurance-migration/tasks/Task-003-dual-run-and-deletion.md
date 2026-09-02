---
id: Task-003
title: "Dual run, deletion, and residual"
type: Task
status: in_progress
track: Deletion
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-parse/NFR-003
    type: references
---
# Task-003: Dual run, deletion, and residual

## Scope

Run the old and new paths against the same candidate revision, record the result
as observed rather than as parity, delete the local evidence framework in a
separate commit afterwards, and record what the deletion costs.

## Completion Evidence

The old path was already failing before this change: at the base revision
`cf43f40`, `make ci` exits 2 and `make test` exits 101 on the staleness deadlock
that reviewers filed as FND-916, and `make evidence-tool` exits 2 on a
`tools.lock` digest that no longer matches the host binary. The dual run records
those exact observations rather than manufacturing a green baseline to claim
parity against.

Deletion removes the local runner, collector, finalizer, verifiers, tool-identity
framework, anchor verifier and Make execution-control guard. The two evidence
schemas are frozen rather than deleted, because retained envelopes name them by
SHA-256. The loss of the Make execution-control guard is a real reduction in
local detection and is recorded as an open unknown in the change-assurance
declaration and as `agent-ix/tl-parse#11`.
