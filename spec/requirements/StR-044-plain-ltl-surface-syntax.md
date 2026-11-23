---
id: StR-044
title: LTL users need to write plain operators without intervals and without a guessed meaning
type: StR
relationships:
  - target: ix://agent-ix/tl-parse/FR-058
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-059
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-060
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-061
    type: satisfied_by
  - target: ix://agent-ix/tl-parse/FR-062
    type: satisfied_by
---

# StR-044: LTL users need to write plain operators without intervals and without a guessed meaning

## Stakeholder Need

Authors of LTL and LTLf requirements need to write `G p`, `F p`, `p U q` and
`X p` as in the literature and have the tool apply exactly the profile they
named, with existing MLTL text still parsing exactly as before.

## Rationale

Interval-free spelling is the community norm. The same spelling is valid in two
profiles with different meaning, so the profile must be a caller decision.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-044-VC-1 | The same source text parsed under two profiles yields graphs with the two profile identities, and no call exists without a profile. | Test (TC-114, TC-117) |
| StR-044-VC-2 | Dialects v1 to v4 produce byte-identical output on their retained corpora. | Test (TC-123) |

## Stakeholders

LTL/LTLf users, FRET users, tl-mltl and tl-solve maintainers.

## Context and Assumptions

Sources: De Giacomo and Vardi IJCAI 2013; Aurandt et al. NFM 2026.

## Traceability

Realized by FR-058 through FR-062; constrained by NFR-046.
