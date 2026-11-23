---
id: PLAN-045
title: Plain LTL surface dialect v5 plan
type: Plan
status: not_started
relationships:
  - target: ix://agent-ix/tl-parse/FR-058
    type: references
  - target: ix://agent-ix/tl-parse/FR-059
    type: references
  - target: ix://agent-ix/tl-parse/FR-060
    type: references
  - target: ix://agent-ix/tl-parse/FR-061
    type: references
  - target: ix://agent-ix/tl-parse/FR-062
    type: references
  - target: ix://agent-ix/tl-parse/NFR-046
    type: references
---

# PLAN-045: Plain LTL surface dialect v5 plan

## Objective

Land TL-429: the bracket-free dialect `tl-parse.clean-ascii/v5` for the LTLf and
infinite-trace profiles, selected explicitly by the caller, with round-trip,
hostile-input and fuzz coverage, and v1 to v4 unchanged. Starts after ADR-041,
tl-syntax ADR-044 and ADR-045 are accepted and tl-syntax TL-428 has landed.

## Dependency DAG

Layer numbers match `ix-board project-chunk P-TL-151`.

```text
layer 1  TL-428 tl-syntax profile and corpus (external to this repo)
layer 2  TL-429
           Task-001 v5 grammar and explicit profile (FR-058, FR-059)
             -> Task-002 formatter and round trip (FR-060)
             -> Task-003 hostile corpus, fuzz, limits (FR-061, NFR-046)
           Task-004 v1 to v4 unchanged proof (FR-062)  (parallel; gates merge)
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | FR-058, FR-059 | TC-112 to TC-117 |
| Task-002 | FR-060 | TC-118, TC-119 |
| Task-003 | FR-061, NFR-046 | TC-120 to TC-122, TC-125, TC-126 |
| Task-004 | FR-062 | TC-123, TC-124 |

## Quality gates

- Gate A after Task-001: TC-123 digests of v1 to v4 identical before any further
  work. If it fails, stop.

## Deferred verification

| Item | Why deferred | Needed to run |
|---|---|---|
| Fuzz campaign beyond seed replay (TC-121) | no fuzz or build capacity during specification | `make fuzz-build fuzz-smoke`, then a budgeted campaign |
| Miri run (NFR-046) | no build capacity | the existing Miri target |
| `make ci` | no build capacity | the repository gate |

## Ungrounded items

None. Import front-ends are not in this repository (ADR-041 item 6).
