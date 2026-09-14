---
id: FR-009
title: "Organize and preserve every versioned text dialect"
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/MRS-001
    type: implements
  - target: ix://agent-ix/tl-parse/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-008
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-014
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-004
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-006
    type: implements
---
# FR-009: Organize and preserve every versioned text dialect

## Description

When clean-ascii text is parsed or a validated formula is formatted, tl-parse SHALL select exactly one closed dialect policy and share only policy-independent traversal, diagnostics and resource accounting across dialects.

## Architecture

The implementation SHALL be organized as
`dialect::{v1,v2,v3}`, `lexer`, `parser`, `formatter`, and `diagnostic`.
Existing public functions and report types remain compatibility re-exports.
The lexer supplies bounded tokens; each dialect module owns its exact spelling,
precedence, associativity, profile and permitted lowering policy; the parser
builds the owner tl-syntax graph; the formatter consults the same selected
dialect policy; diagnostics own stable codes/loci but no recovery semantics.

V1 remains the primitive future/Boolean dialect. V2 adds only W/M and lowers
them through the pinned tl-syntax owner API. V3 is exactly
`tl-parse.clean-ascii/v3` and admits case-sensitive O/H/Y/S/T plus Boolean
syntax under `tl-syntax.past-operators/v1`, producing only
`tl-syntax.formula/v2` / `mltl.origin-complete-history/v1`. V3 refuses every
future, mixed, weak-previous, long-form/aliased and unknown operator. V1/V2
refuse every v3 spelling.

Every successful parse validates its document through the selected tl-syntax
strict document boundary; every formatter accepts only a validated matching
owner document. V3 parse reports use exactly
`tl-parse.past-parse-report/v1`, bind source/dialect/operator/formula contracts,
retain exact spans and limits, and strict-read without unknown/duplicate fields,
trailing data or noncanonical variants.

The reorganization SHALL preserve all existing v1/v2/v3 canonical text,
documents, spans, diagnostics, limit charging and report bytes. It introduces
no second AST, temporal evaluation, rewrite, proposition inference or source
language.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | V1, V2 and V3 accept exactly their closed spellings/profiles and cross-refuse every other dialect without fallback or partial graph. | Test (TC-046) |
| FR-009-AC-2 | Every O/H/Y/S/T precedence, associativity, interval and span case parses to the exact formula-v2 graph and round-trips to one canonical v3 text. | Test (TC-046) |
| FR-009-AC-3 | Existing v1/v2/v3 documents, diagnostics, report bytes, public paths and resource charges remain byte-identical across the module reorganization. | Test (TC-046) |
| FR-009-AC-4 | The real tl-syntax strict reader admits every successful graph and rejects every profile/operator/topology mutation; tl-parse owns no mirror node or semantic evaluator. | Test (TC-046) |
| FR-009-AC-5 | Exact limits succeed and one-over source/token/node/depth/diagnostic/work/output inputs refuse before excess retention; bounded arbitrary UTF-8 never unwinds. | Test (TC-046) |

## Dependencies

FR-001 through FR-004 supply established lexer/parser/formatter behavior,
FR-008 supplies v2 lowering, and tl-syntax FR-011/014 supplies the selected
past profile and strict graph owner reader.

## Status

Proposed architecture reconciliation of delivered Task-002 behavior under
`tl-syntax#52/#64`.
