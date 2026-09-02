---
id: Task-003
title: "Stable diagnostics and limits"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-001
    type: part_of
---
# Task-003: Stable diagnostics and limits

## Scope

Emit stable versioned diagnostics and enforce hard-clamped source, token, node,
depth, diagnostic, and parser-work limits without partial success.

## Completion Evidence

Golden and deterministic report tests exercise each declared parser limit,
stable codes/spans/expected/found/recovery fields, truncation, and JSON bytes.
