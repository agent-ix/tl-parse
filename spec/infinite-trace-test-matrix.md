---
id: TM-002
title: Infinite-trace parser test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-parse/MRS-001
    type: covers
---

# Infinite-trace parser test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-015 | FR-015-AC-1 through FR-015-AC-3 | TC-058 through TC-060 | ✅ implemented |
| FR-016 | FR-016-AC-1 through FR-016-AC-2 | TC-061, TC-062 | ✅ implemented |
| FR-017 | FR-017-AC-1 through FR-017-AC-3 | TC-063 through TC-065 | ✅ implemented |
| NFR-004 | NFR-004-AC-1 | TC-066 | ✅ implemented |

Executing assertions in `tests/infinite_v4.rs` and
`tests/infinite_trace_corpus.rs` cover the public API and checked seeds. The
`clean_ascii_v3` and `unbounded_parse_roundtrip` fuzz targets each completed
64 libFuzzer runs; their seeds are SHA-256 pinned. The original red stubs
remain as the V1 spec-cycle record and are outside the Cargo test census.

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-058 | Parse all admitted future/past forms into strict owner graph | Property | P0 | FR-015-AC-1 | ✅ implemented |
| TC-059 | Bind ordered fairness premises to the same graph without a verdict | Property | P0 | FR-015-AC-2 | ✅ implemented |
| TC-060 | Cross-refuse v1/v2/v3/v4 dialect, profile and clock mismatches | Integration | P0 | FR-015-AC-3 | ✅ implemented |
| TC-061 | Preserve byte-exact interval and premise loci through nesting and UTF-8 | Property | P0 | FR-016-AC-1 | ✅ implemented |
| TC-062 | Replay malformed inputs and map diagnostic codes without message parsing | Integration | P0 | FR-016-AC-2 | ✅ implemented |
| TC-063 | Prove parse-format-parse fixed point and exact fairness order | Property | P0 | FR-017-AC-1 | ✅ implemented |
| TC-064 | Digest-check malformed/locus fixtures and fail on one-axis mutation | Integration | P0 | FR-017-AC-2 | ✅ implemented |
| TC-065 | Execute both named fuzz targets with checked seeds | Fuzz | P1 | FR-017-AC-3 | ✅ implemented |
| TC-066 | Repeat deterministic reports and test every exact/one-over limit | Property | P0 | NFR-004-AC-1 | ✅ implemented |

## Integration Test Matrix

| Purpose | Target | Type | Test Cases |
|---|---|---|---|
| Parse against pinned owner graph and corpus | tl-syntax | service | TC-058 through TC-064 |
| Keep v3 past regression while fuzzing v4 | cargo-fuzz | workspace | TC-065, TC-066 |
