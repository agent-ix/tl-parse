---
id: FR-059
title: "Select the LTL profile explicitly and never infer it from spelling"
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-044
    type: implements
  - target: ix://agent-ix/tl-parse/ADR-041
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-058
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-045
    type: depends_on
---

# FR-059: Select the LTL profile explicitly and never infer it from spelling

## Description

When v5 text is parsed or formatted, tl-parse shall take the semantic profile
only from the caller's `LtlProfile` argument and shall provide no entry point,
default, environment lookup or source-text marker that selects a profile.

## Behavior

The same text `F p0` yields two different graphs, one per profile. `X p0` is
accepted only under `InfiniteTrace`; under `LtlfFiniteTrace` it is refused with a
diagnostic naming `Xs` and `Xw` and no graph. `Xs p0` and `Xw p0` are accepted
only under `LtlfFiniteTrace`. A profile identity string passed through the
generic dialect-selection API that is not one of the two v5 profiles is refused
before lexing.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-059-AC-1 | Source `F p0` parses under both profiles to graphs whose profile identities differ and match the argument. | Test (TC-114) |
| FR-059-AC-2 | Bare `X` under LTLf is refused naming `Xs` and `Xw`; `Xs` and `Xw` under the infinite profile are refused. | Test (TC-115, TC-116) |
| FR-059-AC-3 | No public v5 function lacks the profile parameter, and the profile type has no `Default` or `From<&str>` inference. | Test (TC-117) |

## Dependencies

FR-058.
