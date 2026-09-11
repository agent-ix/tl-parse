---
id: FR-007
title: Context-bound parsed formula reporting
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-003
    type: implements
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
---

# FR-007: Context-bound parsed formula reporting

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
- The report carries a catalog identity digest and a domain-separated binding
  request digest. The request digest binds the validated formula document,
  exact catalog document, explicit nullable requirement context, and exact
  pinned tl-syntax revision so altered or dropped inputs cannot retain the
  same bound result identity.
- The report is a local parser result envelope with its own strict versioned
  wire form. This does not redefine any shared tl-syntax signal, domain,
  binding, or context type.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-007-AC-1 | A valid formula and catalog produce a deterministic native report containing every free proposition once in first parser-node order, with its parser source span and the exact shared catalog/context values. | Test (TC-033) |
| FR-007-AC-2 | A catalog missing any referenced proposition produces typed non-success with that proposition and parser byte span; no successful bound report is returned. | Test (TC-034) |
| FR-007-AC-3 | A provided requirement context is retained verbatim, including its identity and clause/anchor/span fields, without confusing those fields with source diagnostic spans. | Test (TC-035) |
| FR-007-AC-4 | Repeated parse-and-bind operations and validated formula-document, catalog, or context mutations are deterministic, and each declared document identity is bound into the report. Source spellings that parse to the same `FormulaDocument` are intentionally the same binding input. | Test (TC-036) |
| FR-007-AC-5 | Context-free APIs and v1 wire schemas remain structurally compatible. Their bytes may move only when a declared dependency identity moves. The new binding report has an explicit strict versioned wire boundary. | Test (TC-037) |
| FR-007-AC-6 | The public surface adds no runtime dependency on Quire, Quoin, Engineering Assurance, or any local evidence runner, and keeps license/publish boundary verification passing. | Test (TC-038) |

## Dependencies

Depends on the exact pinned tl-syntax catalog/context types and FR-002 for the
validated formula and source-span facts. It does not depend on Quire, Quoin,
Engineering Assurance, or a runtime contract-IR surface.
