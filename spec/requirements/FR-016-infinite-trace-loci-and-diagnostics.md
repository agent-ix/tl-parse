---
id: FR-016
title: Preserve exact infinite-trace source loci and refusals
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-015
    type: depends_on
---

# FR-016: Preserve exact infinite-trace source loci and refusals

## Description

When v4 source is parsed or refused, tl-parse shall retain a byte-exact source
span for every unbounded interval and fairness premise, and shall report the
smallest offending locus with a stable `DiagnosticCode`.

## Inputs

- V4 source, dialect/profile/clock selection and resource limits.

## Outputs

- Operator, interval and premise spans on success; deterministic diagnostic
  code, span, expected set, found token and recovery action on failure.

## Behavior

An interval span covers the complete `[a,)` token sequence and remains
distinct from its operator and containing expression spans. A premise span
covers its expression, while envelope punctuation retains its own locus.
Whitespace, grouping and UTF-8 before a locus do not shift byte offsets to
character indexes. A malformed lower bound, missing comma or parenthesis,
finite-profile unbounded interval, misplaced fairness delimiter, duplicate
premise, unknown proposition and resource overrun have distinct stable codes
or typed refusal details. No parse error is relabeled by comparing message
text. A malformed input never yields a partial owner graph.

The v4 diagnostic-to-FR-341 mapping is total: syntax, profile and clock
refusals map to `unsupported` before evaluation; exceeded work limits map to
`failed` with execution disposition `resource-incomplete`; internal parser
failures map to `failed` with execution disposition `failed`. Successful
parsing supplies no temporal verdict. The mapping
retains original code and locus; it never claims `proved` or `refuted`.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-016-AC-1 | Every interval and premise retains exact byte spans across whitespace, nested syntax and multibyte UTF-8 prefixes. | Test (TC-061) |
| FR-016-AC-2 | Each malformed/profile/resource family has a stable code and smallest offending locus, no partial graph, and a total typed FR-341 mapping. | Test (TC-062) |

## Dependencies

FR-003 owns diagnostic fields and resource behavior; FR-015 owns v4 syntax.
