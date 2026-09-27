---
id: FR-017
title: Canonicalize infinite-trace text and publish parser corpora
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-015
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-016
    type: depends_on
---

# FR-017: Canonicalize infinite-trace text and publish parser corpora

## Description

When a validated v4 graph and fairness document are formatted, tl-parse shall
emit one canonical text form that reaches a parse-format-parse fixed point and
preserves graph, profile, clock and premise-root identity.

## Inputs

- Strict-read `tl-syntax.formula-unbounded/v1` graph and optional fairness
  document with the same profile and clock.
- Effective format limits.

## Outputs

- Canonical v4 text or a typed formatting refusal; digest-pinned malformed
  and locus fixture families for replay.

## Behavior

The formatter emits `[a,)` without spaces inside intervals, one stable
parenthesization and spacing policy, and premise expressions in their declared
order as `fair { premise; ... } : formula`. It omits the envelope for an empty
premise set. It refuses foreign/mismatched graph, profile or clock identity,
and never reconstructs an unbounded interval as a large closed bound.
It renders owner W/M lowerings with their original text operators. A valid
graph whose shared nodes or node order cannot be represented in v4 text
receives `unrepresentable_graph` rather than text with a changed owner
identity. Diagnostic source spans remain available but do not affect that
semantic identity.

The parser corpus holds separately digest-pinned malformed-input and
locus-preservation cases. Each case carries its exact source, selected
profile, expected code or canonical text, byte spans and a human rationale.
The `unbounded_parse_roundtrip` fuzz target and a `clean_ascii_v3` past-time
target are declared now as non-vacuous test-first stubs; TL-14 makes them
executing targets. Corpus expectations are independent of parser output.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-017-AC-1 | Every admitted v4 graph and fairness set formats to text that re-parses to the same owner identities and is byte-stable at the next formatting step. | Test (TC-063) |
| FR-017-AC-2 | Malformed and locus corpus fixtures have unique identities, human expectations and verified digests; changing a source, locus, expected code or digest fails replay. | Test (TC-064) |
| FR-017-AC-3 | Both named fuzz targets exercise real parser paths with checked seeds, bounded outcomes and no panic, including past-time v3 regression. | Test (TC-065) |

## Dependencies

FR-004 owns bounded canonical formatting; FR-015/016 own v4 admission and
source loci.
