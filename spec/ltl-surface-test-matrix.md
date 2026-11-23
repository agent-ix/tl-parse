---
id: TM-043
title: "Plain LTL surface dialect v5 test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-parse/MRS-001
    type: covers
---

# TM-043: Plain LTL surface dialect v5 test matrix

Specification stage (TL-427). Every row is `🚧` until TL-429 lands it.

Identifier scope: ids in this repository are local. `TC-nnn`, `FR-nnn` and
`ADR-nnn` numbers may coincide with ids of the same shape in sibling repositories
(for example tl-mltl); when a document cites an id of another repository it
qualifies it with the repository name (`tl-mltl FR-092`, `tl-mltl TC-300`).
Unqualified ids always mean this repository.

Ticket acceptance mapping (TL-429): "new dialect; existing dialects unchanged"
is FR-058 and FR-062; "profile is always selected explicitly, never inferred
from spelling" is FR-059; "round-trip, fuzz and hostile-input coverage" is
FR-060 and FR-061. Import front-ends (TL-431, TL-432, TL-433) are deliberately
not in tl-parse (ADR-041 item 6); see tl-mltl TM-047.

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---------|-------|------|----------|-----------|--------|
| TC-112 | Parse the 120 LTLf and 116 infinite depth-one formulas to the expected graphs | Unit | P0 | FR-058-AC-1 | 🚧 planned |
| TC-113 | Refuse interval spellings and W M O H Y S T with distinct codes under both profiles | Unit | P0 | FR-058-AC-2 | 🚧 planned |
| TC-114 | Parse F p0 under both profiles to differing profile identities | Unit | P0 | FR-059-AC-1, StR-044-VC-1 | 🚧 planned |
| TC-115 | Refuse bare X under LTLf naming Xs and Xw | Unit | P0 | FR-059-AC-2 | 🚧 planned |
| TC-116 | Refuse Xs and Xw under the infinite profile | Unit | P0 | FR-059-AC-2 | 🚧 planned |
| TC-117 | Assert no profile-less v5 entry point and no profile default or inference | Static | P0 | FR-059-AC-3, StR-044-VC-1 | 🚧 planned |
| TC-118 | Round-trip 236 depth-one formulas and 10 precedence cases to a fixed point | Property | P0 | FR-060-AC-1 | 🚧 planned |
| TC-119 | Refuse formatting of graphs outside the dialect image | Unit | P0 | FR-060-AC-2 | 🚧 planned |
| TC-120 | Replay the 29 hostile cases per profile against the manifest | Integration | P0 | FR-061-AC-1 | 🚧 planned |
| TC-121 | Build the clean_ascii_v5 fuzz target and replay the 29 seeds | Fuzz | P1 | FR-061-AC-2 | 🚧 planned |
| TC-122 | Check stable codes and half-open spans of every refusal | Unit | P0 | FR-061-AC-1 | 🚧 planned |
| TC-123 | Compare four dialect digests and retained v1 to v4 outputs to pre-change values | Integration | P0 | FR-062-AC-1, StR-044-VC-2 | 🚧 planned |
| TC-124 | Confirm v1 to v4 still refuse bracket-free temporal text | Unit | P0 | FR-062-AC-2 | 🚧 planned |
| TC-125 | Accept each ceiling at its value and refuse one past it under both profiles | Unit | P1 | NFR-046-AC-1 | 🚧 planned |
| TC-126 | Compare reports of two runs byte for byte | Unit | P1 | NFR-046-AC-2 | 🚧 planned |
