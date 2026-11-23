---
id: FR-028
title: Select a distinct numeric dialect and leave existing dialects unchanged
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: implements
  - target: ix://agent-ix/tl-parse/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-303
    type: depends_on
  - target: ix://agent-ix/tl-parse/ADR-020
    type: depends_on
---

# FR-028: Select a distinct numeric dialect and leave existing dialects unchanged

## Description

When `tl-parse.clean-ascii/v5` is explicitly selected together with a signal
catalog, tl-parse shall parse numeric predicates, set aggregation and contracts
into the tl-syntax predicate table, formula graph and contract set, and when any
other dialect is selected tl-parse shall behave exactly as it does today.

## Inputs

- UTF-8 source, explicit dialect `tl-parse.clean-ascii/v5`, a validated
  `tl-syntax.signal-catalog/v1` document (required: v5 is always
  context-bound), an `mltl.closed-trace/v1` or `mltl.online-prefix/v1` profile
  identity, and caller limits.

## Outputs

- A `tl-parse.numeric-parse-report/v1` report containing the validated formula
  document, the predicate table, optionally the contract-set document and the
  aggregation expansion report, the effective limits, spans, and a structural echo
  of the catalog (its schema identity, signal count and binding count, no digest);
  otherwise typed diagnostics and no graph.

## Behavior

- Operators admitted are those of v2 (F, G, U, R and W, M through the owner
  lowering). Past operators, unbounded intervals and the fairness envelope are
  refused in v5 as `unsupported_operator` with their locus; extending v5 to
  those profiles is a later dialect revision.
- A profile other than `mltl.closed-trace/v1` and `mltl.online-prefix/v1` is refused
  before parsing (`profile_mismatch`). Numeric past requirements therefore have no
  text form in v5 (see ADR-020).
- V1 through v4 reject every v5-only form (identifier atoms, comparisons,
  `each`, `some`, `atleast`, `contract`, `set`) with their existing diagnostics;
  their reports, canonical output and corpora are byte-identical to the
  pre-change versions.
- The v5 report carries its own schema identity and diagnostics identity
  (`tl-parse.numeric-diagnostics/v1`); the existing `DiagnosticCode` wire
  vocabulary is not extended.
- `DIALECT_V5_REVISION` (an integer) and a normative
  `docs/DIALECT-005-clean-ascii-v5.md` follow the pattern of v4; unlike v4, there is
  no dialect digest record. The dialect identity string collides with the plain LTL
  dialect v5 of P-TL-151 and is provisional until that human decision (ADR-020).

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-028-AC-1 | Dialect and profile selection is explicit; v5 without a catalog or with a non-future profile is refused before lexing. | unit-testing (TC-1540) |
| FR-028-AC-2 | The existing v1..v4 corpora, reports and canonical bytes are unchanged, and each rejects every v5-only form with its existing diagnostics. | integration-testing (TC-1541) |
| FR-028-AC-3 | Past, unbounded and fairness syntax in v5 are refused as `unsupported_operator` with a byte locus and no graph. | unit-testing (TC-1542) |

## Dependencies

FR-009 owns dialect separation.
