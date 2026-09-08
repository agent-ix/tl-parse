---
id: FR-007
title: Bind parsed formulas to typed signal catalogs and caller context
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-003
    type: implements
  - target: ix://agent-ix/tl-syntax/MRS-001
    type: depends_on
---

# FR-007: Bind parsed formulas to typed signal catalogs and caller context

## Description

The crate shall provide an additive, opt-in binding surface that pairs a
successful `FormulaDocument` with a validated `tl_syntax::SignalCatalog` and
an optional `tl_syntax::RequirementContext`. Existing parse and format APIs
remain unchanged. The surface introduces no temporal AST, signal model,
FRETish parser, semantic-IR dependency, Quire dependency, or Quoin dependency.

## Behavior

- The surface parses through the existing parser before binding; a parse
  diagnostic returns no bound formula.
- Each proposition node resolves only through the catalog's direct binding.
  Every referenced proposition resolves to exactly one declared Boolean signal.
  Missing bindings report the proposition identifier and its original half-open
  MLTL byte span.
- Successful output exposes a deterministic, duplicate-free set of referenced
  proposition/signal pairs in increasing proposition identity order, with the
  validated formula and catalog. It does not infer names, coerce integer or
  fixed-decimal domains, or duplicate catalog values.
- Optional caller context is preserved as a separate value. Its clause-level
  source span is not a parser diagnostic span and does not replace formula-node
  source spans.
- Serialized contextual reports use strict tl-syntax catalog and
  requirement-context documents. Present nullable context is explicit; omitted
  required wire fields and unknown fields are refused.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-007-AC-1 | Binding a canonical MLTL formula returns the exact free proposition/signal pairs in deterministic proposition order, preserving signal ids, names, and bounded domains. | Test (TC-029) |
| FR-007-AC-2 | A formula proposition lacking a catalog binding is refused with that proposition's exact MLTL byte span; a non-Boolean binding is rejected by the upstream catalog rather than treated as Boolean. | Test (TC-030) |
| FR-007-AC-3 | Canonical format → parse → bind preserves formula structure, catalog identity, bounded domains, and supplied caller context without drift. | Test (TC-031) |
| FR-007-AC-4 | Contextual wire output retains supplied requirement context distinctly from parser spans, accepts explicit null where the API permits absence, and rejects omitted required fields, unknown fields, or invalid upstream document versions. | Test (TC-032) |
| FR-007-AC-5 | Existing context-free parser, formatter, CLI, corpus, fuzz, and round-trip behavior is unchanged, and the crate remains unpublished under `MIT OR Apache-2.0` with no runtime Quire, Quoin, or contract-IR dependency. | Test (TC-033) |

## Dependencies

Depends on FR-002, FR-004, and exact tl-syntax revision
`6ad7499f2ccc179bb33b2590666399c6632a7e3c`.
