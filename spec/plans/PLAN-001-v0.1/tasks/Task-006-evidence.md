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

Retain the exact candidate revision's producer outcomes, tools,
dependency/dialect/corpus identities, limits, review results, and limitations
through the Quoin-owned change-assurance path.

## Completion Evidence

Every declared producer output is sealed and retained byte-identically as its
own proof input, every attested result is derived from those bytes, and the
receipt reports the intentionally absent human release decision without Quoin
or Quire executing a producer. The parser and clean-ascii-v2 campaign documents
remain separate proof results.

Reopened on 2026-09-13 to route the structured bounded-fuzz campaign results
through the existing Quoin-owned intake and verify retained-byte identity.

Completed on 2026-09-13 at `cc543ea`: both target-specific campaign documents
were retained byte-identically, all seven proof obligations attested from their
producer bytes, and every Quoin scenario, control, and mutation probe matched.
