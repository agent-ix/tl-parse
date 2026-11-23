---
id: FR-031
title: Format numeric predicates canonically with a parse-format fixed point
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: implements
  - target: ix://agent-ix/tl-parse/FR-029
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-004
    type: depends_on
---

# FR-031: Format numeric predicates canonically with a parse-format fixed point

## Description

When a validated graph, predicate table and catalog are formatted, tl-parse shall
emit canonical v5 text such that parsing it reproduces the same graph, table and
proposition identities, or refuse with a typed `unrepresentable_graph`.

## Behavior

- Predicates print as `(left op right)` with one space around the comparison
  operator and around binary arithmetic operators, nested binary arithmetic fully
  parenthesized, `abs(e)` and `scale(e, k)` with one space after the comma,
  signal names exactly as in the catalog, integer literals as decimal digits,
  decimal literals with exactly `scale` fractional digits.
- A proposition defined by the table prints as its predicate; every other
  proposition prints as `p<id>`. The temporal skeleton prints as in v2.
- Canonical text of an aggregation or contract source is the expansion
  (ordinary formula) respectively one `contract name: assume phi; guarantee psi;`
  declaration per contract in id order; aggregation is not recovered from a
  graph.
- Fixed point: for every successfully parsed v5 source `s`,
  `format(parse(s))` parses to the same graph, and
  `format(parse(format(parse(s)))) == format(parse(s))` byte for byte.
- Refused: a table or graph the dialect cannot express (past node, unbounded
  interval, a predicate whose signal is absent from the supplied catalog,
  a proposition both bound and defined) returns `unrepresentable_graph`.
- Output is bounded by the existing output limit and work limit.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-031-AC-1 | Parse then format round-trips on every valid case in the numeric corpus to its recorded canonical text, and re-parse yields an identical graph and table. | integration-testing (TC-1549) |
| FR-031-AC-2 | The fixed point holds for generated valid sources of bounded size, including non-canonical spellings of the same graph. | property-based-testing (TC-1550) |
| FR-031-AC-3 | Unrepresentable graphs are refused with the typed code and no output. | unit-testing (TC-1551) |

## Dependencies

FR-004 owns v1 canonical formatting rules reused for the temporal skeleton.
