---
id: FR-060
title: "Format v5 canonically and round-trip every formula in its image"
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-044
    type: implements
  - target: ix://agent-ix/tl-parse/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-058
    type: depends_on
---

# FR-060: Format v5 canonically and round-trip every formula in its image

## Description

When a validated graph is formatted in v5 under a profile, tl-parse shall emit
the unique canonical text if the graph is in the dialect's image and a typed
`NotExpressible` refusal otherwise.

## Behavior

The image under `LtlfFiniteTrace` is the whole LTLf edition. The image under
`InfiniteTrace` is graphs whose temporal nodes are `F/G/U/R` with interval
`[0,)` and `F[1,1]` (spelled `X`); `G[1,1]`, any other interval, and past nodes
are not in the image and are refused rather than rewritten. Canonical text uses
the minimum parentheses of the v1 precedence rules and single spaces.
`parse(format(g)) = g` and `format(parse(t))` is a fixed point.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-060-AC-1 | For all 236 depth-one formulas (120 LTLf, 116 infinite) and 10 hand-authored precedence cases, parse then format then parse reproduces the identical graph and the text is a fixed point. | Test (TC-118) |
| FR-060-AC-2 | Formatting a graph outside the image (a closed interval, `G[1,1]`, a past node, `Xs` under the infinite profile) returns `NotExpressible` and no text. | Test (TC-119) |

## Dependencies

FR-004 owns the v1 canonical rules reused here.
