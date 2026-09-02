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
| FR-002 | FR-002-AC-1 through FR-002-AC-3 | TC-005 through TC-008 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-009 through TC-013 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-014 through TC-017 | ✅ covered |
| FR-005 | FR-005-AC-1 through FR-005-AC-3 | TC-018 through TC-021 | ✅ covered |
| FR-006 | FR-006-AC-1 through FR-006-AC-3, FR-006-AC-5 through FR-006-AC-7 | TC-022 through TC-024, TC-026 through TC-028 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-002, FR-003 | TC-001, TC-005, TC-008, TC-010, TC-020 | ✅ covered |
| StR-002 | FR-003, FR-004, FR-005, FR-006 | TC-011, TC-016, TC-018, TC-019, TC-021, TC-027 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-bound tests | TC-011 through TC-017, TC-021 | ✅ covered |
| NFR-002 | dialect and provenance inspection | TC-020 | ✅ covered |
| NFR-003 | qualification integrity and fail-closed shared intake | TC-023, TC-026 | ✅ covered |

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
| TC-018 | Validate malformed/resource corpus and checksums | Integration | P0 | FR-005-AC-1, StR-002-VC-2 | ✅ implemented |
| TC-019 | Consume checked-in fuzz target seeds | Fuzz | P0 | FR-005-AC-2, StR-002-VC-2 | ✅ implemented |
| TC-020 | Validate dialect provenance and CLI valid paths | Integration | P0 | FR-001-AC-3, FR-005-AC-3, NFR-002-AC-1 | ✅ implemented |
| TC-021 | Validate CLI dispatch, read/write/error paths, malformed intervals, diagnostic limits, and determinism | Integration | P0 | FR-005-AC-3, NFR-001-AC-1 | ✅ implemented |
| TC-022 | Classify every shared component through the packaged compatibility matrix and refuse a mirror reference | Integration | P0 | FR-006-AC-1 | ✅ implemented |
| TC-023 | Reach Quoin intake from producer bytes, and prove neither Quoin nor Quire executes a producer | Integration | P0 | FR-006-AC-2, NFR-003-AC-1, NFR-003-AC-2 | ✅ implemented |
| TC-024 | Bind the sealed record's impact snapshot to a populated Quire export naming every requirement | Integration | P0 | FR-006-AC-3 | ✅ implemented |
| TC-026 | Demonstrate all twelve verification outcomes and pair every negative with a positive control | Integration | P0 | FR-006-AC-5, NFR-003-AC-3 | ✅ implemented |
| TC-027 | Report malformed input as malformed, agreeing with the corpus manifest and surviving into the retained bytes | Integration | P0 | FR-006-AC-6, StR-002-VC-2 | ✅ implemented |
| TC-028 | Prove no local evidence framework remains and none of its files came back | Integration | P0 | FR-006-AC-7 | ✅ implemented |
