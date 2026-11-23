---
id: TM-014
title: "Numeric dialect test matrix (tl-parse)"
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: covers
---

# Numeric dialect test matrix (tl-parse)

Project P-TL-150, tl-parse portion: TL-421 and the parsing half of TL-424 and
TL-425. Test ids TC-1540 through TC-1599 are reserved for this project in
tl-parse. Every row is planned; none claims an implementation.

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-1540 | Explicit dialect, catalog and profile selection; refusals before lexing | Unit | P0 | FR-028-AC-1 | 🚧 planned (TL-421) |
| TC-1541 | Existing v1..v4 corpora, reports and bytes unchanged; v5-only forms rejected there | Integration | P0 | FR-028-AC-2 | 🚧 planned (TL-421) |
| TC-1542 | Past, unbounded and fairness syntax refused in v5 with locus | Unit | P1 | FR-028-AC-3 | 🚧 planned (TL-421) |
| TC-1543 | Every operator, node kind and scale parses to the declared table, deterministic allocation | Integration | P0 | FR-029-AC-1 | 🚧 planned (TL-421) |
| TC-1544 | Each typing and syntax refusal at its locus, no graph | Unit | P0 | FR-029-AC-2 | 🚧 planned (TL-421) |
| TC-1545 | Generated sources never panic, return graph or typed diagnostics | Property | P0 | FR-029-AC-3 | 🚧 planned (TL-421) |
| TC-1546 | Each diagnostic code from a minimal source; closed set | Unit | P0 | FR-030-AC-1 | 🚧 planned (TL-421) |
| TC-1547 | More than 64 diagnostics: first 64 and decisive refusal kept | Unit | P1 | FR-030-AC-2 | 🚧 planned (TL-421) |
| TC-1548 | Report reader strictness | Unit | P1 | FR-030-AC-3 | 🚧 planned (TL-421) |
| TC-1549 | Parse then format round-trips on the numeric corpus | Integration | P0 | FR-031-AC-1 | 🚧 planned (TL-421) |
| TC-1550 | Fixed point on generated valid sources | Property | P0 | FR-031-AC-2 | 🚧 planned (TL-421) |
| TC-1551 | Unrepresentable graphs refused | Unit | P1 | FR-031-AC-3 | 🚧 planned (TL-421) |
| TC-1552 | Quantifiers equal hand-written expansion; counts in report | Integration | P0 | FR-032-AC-1 | 🚧 planned (TL-424) |
| TC-1553 | Expansion bound plus one refused with counts | Unit | P0 | FR-032-AC-2 | 🚧 planned (TL-424) |
| TC-1554 | Contract declarations parse; duplicate and half contracts refused | Unit | P0 | FR-032-AC-3 | 🚧 planned (TL-425) |
| TC-1555 | Hostile corpus manifest covers every class and matches the parser | Integration | P0 | FR-033-AC-1 | 🚧 planned (TL-421) |
| TC-1556 | numeric_v5_roundtrip fuzz target builds and smoke-replays clean | Fuzz | P0 | FR-033-AC-2 | 🚧 planned (TL-421) |
| TC-1557 | Hostile corpus within hard maxima | Unit | P0 | FR-033-AC-3 | 🚧 planned (TL-421) |
| TC-1558 | Repeat runs byte-identical and bounded | Unit | P0 | NFR-016-AC-1 | 🚧 planned (TL-421) |
| TC-1559 | Reviewer reads numeric, group and contract text and re-parses to the same graph | Demonstration | P2 | StR-014-VC-1 | 🚧 planned (TL-421) |
| TC-1560 | Hostile input yields typed diagnostics, no panic | Demonstration | P2 | StR-014-VC-2 | 🚧 planned (TL-421) |

## Ticket acceptance-criteria map (tl-parse scope)

| Ticket | Acceptance criterion | Requirements | Test cases |
|---|---|---|---|
| TL-421 | Parse then format round-trips on the corpus | FR-031 | TC-1549, TC-1550 |
| TL-421 | Existing dialects unchanged | FR-028 | TC-1541 |
| TL-421 | Fuzz target and hostile-input corpus extended | FR-033 | TC-1555, TC-1556, TC-1557 |
| TL-424 | Oversized expansions refused; expanded form equals hand-written form | FR-032 | TC-1552, TC-1553 |
| TL-425 | Contract form parsed by tl-parse | FR-032 | TC-1554 |

The master map for all tickets is in tl-syntax `spec/numeric-predicates-test-matrix.md`.
