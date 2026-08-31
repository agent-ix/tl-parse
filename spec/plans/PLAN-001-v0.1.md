---
id: PLAN-001
title: tl-parse v0.1 implementation plan
type: Plan
status: active
relationships:
  - target: ix://agent-ix/tl-parse/FR-001
    type: references
  - target: ix://agent-ix/tl-parse/FR-005
    type: references
---

# tl-parse v0.1 Implementation Plan

## Dependency DAG

```text
PGM-01 + exact tl-syntax
  -> requirements, matrix, assurance packet, composite review
  -> versioned lexer/dialect
  -> direct bounded parser + stable diagnostics
  -> bounded canonical formatter + generated round trips
  -> malformed corpus + fuzz seeds + thin CLI
  -> retained evidence + independent human review
```

## Work Packages

1. Validate the specification, matrix, assurance packet, and composite review.
2. Pin tl-syntax and implement the bounded lexer and direct graph parser.
3. Implement versioned diagnostics, explicit counters, and fail-closed limits.
4. Implement iterative canonical formatting and generated round-trip tests.
5. Retain malformed/resource fixtures, fuzz seeds, and CLI integration tests.
6. Complete code/gap reviews and retain a PGM-01 envelope without publishing.

## Exit Criteria

Every matrix row is backed, all local and hosted gates pass, corpus and evidence
hashes verify, the crate remains unpublished, and the human source-release
decision remains open for independent review.
