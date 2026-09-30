---
id: TM-001
title: tl-parse v0.1 test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-parse/MRS-001
    type: covers
---

# tl-parse v0.1 Test Matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1 through FR-001-AC-3 | TC-001 through TC-004, TC-020 | ✅ covered |
| FR-002 | FR-002-AC-1 through FR-002-AC-4 | TC-005 through TC-008, TC-030 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-009 through TC-013 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-014 through TC-017 | ✅ covered |
| FR-005 | FR-005-AC-1 through FR-005-AC-3 | TC-018 through TC-021, TC-047 | ✅ covered |
| FR-007 | FR-007-AC-1 through FR-007-AC-6 | TC-033 through TC-038 | ✅ covered |
| FR-008 | FR-008-AC-1 through FR-008-AC-6 | TC-039 through TC-045 | ✅ covered |
| FR-009 | FR-009-AC-1 through FR-009-AC-5 | TC-046 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-002, FR-003, FR-008 | TC-001, TC-005, TC-008, TC-010, TC-020, TC-039 | ✅ covered |
| StR-002 | FR-003, FR-004, FR-005 | TC-011, TC-016, TC-018, TC-019, TC-021, TC-047 | ✅ covered |
| StR-003 | FR-007 | TC-033 through TC-037 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-bound tests | TC-011 through TC-017, TC-021 | ✅ covered |
| NFR-002 | dialect and provenance inspection | TC-020 | ✅ covered |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Recognize the complete dialect token vocabulary | Unit | P0 | FR-001-AC-1 | ✅ implemented |
| TC-002 | Apply specified precedence and associativity | Unit | P0 | FR-001-AC-1 | ✅ implemented |
| TC-003 | Reject unknown/non-ASCII tokens at byte spans | Unit | P0 | FR-001-AC-2 | ✅ implemented |
| TC-004 | Reject non-canonical and overflowing numbers | Unit | P0 | FR-001-AC-2 | ✅ implemented |
| TC-005 | Parse primitive and nested complete vocabulary | Unit | P0 | FR-002-AC-1 | ✅ implemented |
| TC-006 | Retain topological nodes and source spans | Unit | P0 | FR-002-AC-1 | ✅ implemented |
| TC-007 | Validate exact tl-syntax graph and profile | Integration | P0 | FR-002-AC-2 | ✅ implemented |
| TC-008 | Recover from missing/trailing syntax without output | Unit | P0 | FR-002-AC-3 | ✅ implemented |
| TC-009 | Match diagnostic golden records | Unit | P0 | FR-003-AC-1 | ✅ implemented |
| TC-010 | Serialize versioned diagnostics and expectations | Unit | P0 | FR-003-AC-1 | ✅ implemented |
| TC-011 | Exhaust source, token, node, and depth limits | Unit | P0 | FR-003-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-012 | Exhaust diagnostic and parser-work limits | Unit | P0 | FR-003-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-013 | Repeat parse reports byte-identically | Unit | P0 | FR-003-AC-3, NFR-001-AC-1 | ✅ implemented |
| TC-014 | Render exact canonical forms for every operator | Unit | P0 | FR-004-AC-1 | ✅ implemented |
| TC-015 | Prove canonical formatting idempotent | Unit | P0 | FR-004-AC-1, NFR-001-AC-1 | ✅ implemented |
| TC-016 | Round-trip a generated bounded formula population | Property | P0 | FR-004-AC-2, StR-002-VC-1 | ✅ implemented |
| TC-017 | Bound iterative formatting and shared/deep graphs | Unit | P0 | FR-004-AC-3, NFR-001-AC-2 | ✅ implemented |
| TC-018 | Validate malformed/resource corpus | Integration | P0 | FR-005-AC-1, StR-002-VC-2 | ✅ implemented |
| TC-019 | Consume checked-in fuzz target seeds | Fuzz | P0 | FR-005-AC-2, StR-002-VC-2 | ✅ implemented |
| TC-020 | Validate dialect provenance and CLI valid paths | Integration | P0 | FR-001-AC-3, FR-005-AC-3, NFR-002-AC-1 | ✅ implemented |
| TC-021 | Validate CLI dispatch, read/write/error paths, malformed intervals, diagnostic limits, and determinism | Integration | P0 | FR-005-AC-3, NFR-001-AC-1 | ✅ implemented |
| TC-030 | Preserve grouped child lexical spans and enclosing ancestor extents | Unit | P0 | FR-002-AC-4 | ✅ implemented |
| TC-033 | Report ordered free propositions with parser spans and shared documents | Unit | P0 | FR-007-AC-1, StR-003-VC-1 | ✅ implemented |
| TC-034 | Refuse an unresolved proposition with its parser byte span | Unit | P0 | FR-007-AC-2, StR-003-VC-2 | ✅ implemented |
| TC-035 | Preserve shared requirement context distinct from parser spans | Unit | P0 | FR-007-AC-3, StR-003-VC-1 | ✅ implemented |
| TC-036 | Bind every declared document identity deterministically, detect document mutations, and equate source spellings that parse to the same document | Integration | P0 | FR-007-AC-4, StR-003-VC-2 | ✅ implemented |
| TC-037 | Preserve context-free compatibility and strictly version binding wires | Integration | P0 | FR-007-AC-5, StR-003-VC-2 | ✅ implemented |
| TC-038 | Retain public dependency and evidence-tool boundaries | Integration | P1 | FR-007-AC-6 | ✅ implemented |
| TC-039 | Bind the explicit v2 dialect identity and digest, and keep v1 parsing, reports, and derived-spelling rejection byte-unchanged | Integration | P0 | FR-008-AC-1, StR-001-VC-1 | ✅ implemented |
| TC-040 | Parse generated W/M/U/R/Boolean chains into the graph built by direct construction with tl-syntax lowering | Property | P0 | FR-008-AC-2 | ✅ implemented |
| TC-041 | Attribute exact operator, expression, and lowered-node spans across whitespace, grouping, and nesting | Unit | P0 | FR-008-AC-2 | ✅ implemented |
| TC-042 | Refuse interval-less, aliased, unsupported, malformed-bound, node-charge, and lowering-work inputs with stable codes and no document | Unit | P0 | FR-008-AC-3 | ✅ implemented |
| TC-043 | Format lowered documents as primitive-only text that, within limits, v1 accepts and v2 re-parses to the same text, byte-identical to direct construction; beyond limits the reparse is refused only for resources | Property | P0 | FR-008-AC-4 | ✅ implemented |
| TC-044 | Build and seed the clean-ascii v2 fuzz target and bound arbitrary-input outcomes | Fuzz | P1 | FR-008-AC-5 | ✅ implemented |
| TC-045 | Serialize the derived report deterministically under a strict versioned identity that mutations change | Unit | P0 | FR-008-AC-6 | ✅ implemented |
| TC-046 | Preserve all three dialects while separating their policies | Integration | P0 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5 | ✅ implemented |
| TC-047 | Classify fuzz smoke outcomes from process status and artifacts, refuse unknown targets and ambient sanitizer overrides, and terminate a timed-out process group | Unit | P0 | FR-005-AC-2 | ✅ implemented |
