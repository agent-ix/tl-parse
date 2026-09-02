---
id: Task-006
title: "Exact-candidate evidence"
type: Task
status: done
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
record.

## Completion Evidence

The retained `fc8e742cfb51` record has a passing post-seal collection summary,
two passing sealed PGM-01 validations, and a checksum manifest covering every
artifact. The envelope remains non-self-attesting; its separate post-seal
summary records exact finalized-envelope validation.
