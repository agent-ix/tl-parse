---
id: FR-062
title: "Keep dialects v1 to v4 and their retained corpora byte-identical"
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-044
    type: implements
  - target: ix://agent-ix/tl-parse/FR-009
    type: depends_on
---

# FR-062: Keep dialects v1 to v4 and their retained corpora byte-identical

## Description

When the v5 dialect is added, tl-parse shall leave the records, digests,
documents, reports and corpus outputs of dialects v1 to v4 byte-identical, and
dialects v1 to v4 shall continue to refuse bracket-free temporal spellings.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-062-AC-1 | The four dialect digests and every retained corpus output of v1 to v4 equal their pre-change values. | Test (TC-123) |
| FR-062-AC-2 | Each of v1 to v4 refuses `G p0` and `p0 U p1` exactly as before. | Test (TC-124) |

## Dependencies

FR-009.
