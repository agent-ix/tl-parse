---
id: Task-001
title: "Inventory, pins, and the upstream repin"
type: Task
status: done
track: Foundation
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-parse/FR-006
    type: references
---
# Task-001: Inventory, pins, and the upstream repin

## Scope

Produce the keep/replace/delete/defer inventory, declare the adopted release in
`assurance/pins.json`, delegate every version verdict to the packaged
compatibility matrix, and move the tl-syntax pin onto a revision reachable from
that repository's `main`.

## Completion Evidence

`scripts/check_shared_pins.py` classifies four components through
`engineering_assurance.compatibility` and restates no version rule locally. The
hosted workflow's `@agent-ix/quoin@0.22.5` pin — a version the matrix names
explicitly incompatible — is repinned to 0.23.1 and `ix-flow@0.0.4` is added.
The compiled tl-syntax revision moves from `740182f1`, reachable only from an
open pull request's branch, to `953ee825` on `main`; `docs/ATTRIBUTION.md` now
records the authorship basis and the compiled revision as two separate facts and
`scripts/check_attribution.py` re-derives the second from the resolved checkout.
