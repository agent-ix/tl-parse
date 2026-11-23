---
id: NFR-046
title: "Keep v5 parsing bounded and deterministic"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-parse/FR-058
    type: constrains
  - target: ix://agent-ix/tl-parse/FR-060
    type: constrains
  - target: ix://agent-ix/tl-parse/NFR-004
    type: depends_on
---

# NFR-046: Keep v5 parsing bounded and deterministic

## Statement

When a v5 parse or format request is processed, tl-parse shall apply the
existing source, token, node, depth, diagnostic and output ceilings before
excess retention and return equal reports at equal inputs.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Each ceiling accepted at its value and refused one past it | 100% | 100% | Test |
| Reports byte-equal across 2 runs of the same input | 100% | 100% | Test |

## Verification

The NFR-004 limit and determinism tests are repeated for v5.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-046-AC-1 | Each ceiling accepts at its value and refuses at value plus one with the typed limit error, under both profiles. | Test (TC-125) |
| NFR-046-AC-2 | Two runs over every hostile and round-trip input produce byte-equal reports. | Test (TC-126) |
