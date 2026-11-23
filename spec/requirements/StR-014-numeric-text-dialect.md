---
id: StR-014
title: Authors need readable numeric, group and contract text that parses to the shared model
type: StR
relationships:
  - target: ix://agent-ix/tl-parse/FR-028
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-029
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-031
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-032
    type: satisfied_by
---

# StR-014: Authors need readable numeric, group and contract text that parses to the shared model

## Stakeholder Need

Tool maintainers and requirement reviewers require that numeric predicates,
group requirements and assume-guarantee contracts shall have one readable,
canonical text form that parses into the tl-syntax models with located,
typed diagnostics, without changing how any existing dialect parses.

## Rationale

Without a surface form the numeric model of tl-syntax (FR-302 onward) is
reachable only by hand-built documents, and reviewers cannot diff requirements.
Existing users of dialects v1 through v4 must not see their bytes or reports
move. TL text remains derived interchange; FSL is the editable authority. Text authored by
hand is unattributed (tl-syntax/ADR-014 decision 6) and a result derived from it makes
no claim about an FSL requirement (tl-parse ADR-020).

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-014-VC-1 | A reviewer can read a numeric requirement, a group requirement and a contract in canonical text and obtain the same graph by parsing it again. | Demonstration |
| StR-014-VC-2 | Hostile input to the new dialect yields typed diagnostics within bounds and never a panic or an unbounded allocation. | Demonstration |

## Dependencies

tl-syntax FR-302..FR-311 (the tl-syntax repository; this repository's StR-014 is not tl-mltl's StR-014).
