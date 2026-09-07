---
id: FR-007
title: Context-bound parsed formula reporting
type: FR
status: proposed
relationships:
  - target: ix://agent-ix/tl-parse/StR-003
    type: implements
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
---

# FR-007 Context-bound parsed formula reporting

## Description

The crate shall provide an additive native binding report for a successfully
parsed `FormulaDocument` and the shared `tl-syntax` `SignalCatalogDocument`,
with an optional shared `RequirementContextDocument`. The report shall preserve
the parser's exact free-proposition occurrence order and byte spans, and shall
either establish that every referenced proposition resolves exactly once or
return a typed non-success result naming the unresolved proposition and source
span. It shall not create local signal, domain, binding, or context schemas.

Existing context-free parsing and formatting APIs remain unchanged. Binding is
not parsing: catalog/context identities are caller-supplied shared documents;
parser diagnostic spans remain source byte spans and are distinct from any
shared requirement clause or anchor span. The surface shall not infer Boolean
or signal types, coerce names, introduce FRETish syntax, introduce a second
AST, or run tools/evidence collection.

## Behavior

- The parser stays authoritative only for source text, graph construction, and
  source spans; tl-syntax remains authoritative for catalog and requirement
  context documents.
- The binding result is additive and native. It does not alter existing parse
  reports, formatter input, the ASCII dialect, or error recovery.
- Every shared document is retained as provided; no field is synthesized from
  a proposition name or parser location.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-007-AC-1 | A valid formula and catalog produce a deterministic native report containing every free proposition once in first parser-node order, with its parser source span and the exact shared catalog/context values. | Test (TC-029) |
| FR-007-AC-2 | A catalog missing any referenced proposition produces typed non-success with that proposition and parser byte span; no successful bound report is returned. | Test (TC-030) |
| FR-007-AC-3 | A provided requirement context is retained verbatim, including its identity and clause/anchor/span fields, without confusing those fields with source diagnostic spans. | Test (TC-031) |
| FR-007-AC-4 | Repeated parse-and-bind operations and catalog/context/source mutations are deterministic, and each declared input identity is bound into the report. | Test (TC-032) |
| FR-007-AC-5 | Context-free APIs and their wire records remain compatible, while the new binding report has an explicit strict versioned wire boundary. | Test (TC-033) |
| FR-007-AC-6 | The public surface adds no runtime dependency on Quire, Quoin, Engineering Assurance, or any local evidence runner, and keeps license/publish boundary verification passing. | Test (TC-034) |

## Dependencies

Depends on the exact pinned tl-syntax catalog/context types and FR-002 for the
validated formula and source-span facts. It does not depend on Quire, Quoin,
Engineering Assurance, or a runtime contract-IR surface.
