---
id: FR-005
title: Retain hostile-input, fuzz, CLI, and conformance evidence
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-002
    type: implements
---

# FR-005: Retain hostile-input, fuzz, CLI, and conformance evidence

## Description

The repository shall retain versioned malformed and resource fixtures, a fuzz
target seeded by those fixtures, thin CLI surfaces, and PGM-01 evidence for one
exact source candidate.

## Behavior

- Corpus files and manifest are checksum-protected and identify expected
  diagnostic codes or success results.
- The complete local gate compiles the checked-in fuzz target and executes a
  bounded libFuzzer smoke run over every checksum-protected seed. The target
  exercises parsing, diagnostic serialization, and successful canonical round
  trips under small fixed budgets.
- `tl-parse validate` and `tl-parse format` accept a profile and file/stdin,
  use the library report, and have stable success, invalid-input, and usage
  exit classes.
- Retained candidate evidence is immutable. It is read through the shared
  compatibility mapping rather than by a local verifier, and no automated check
  grants review or release authority.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | The malformed/resource corpus is checksum-valid and every fixture produces its declared bounded outcome. | Test (TC-018) |
| FR-005-AC-2 | The checked-in fuzz target compiles, a bounded libFuzzer smoke run consumes every seed, and successful seeds round-trip under declared limits. | Test (TC-019) |
| FR-005-AC-3 | CLI validation/formatting outputs and exit classes match the library for valid, invalid, profile, stdin, source-limit, and usage cases; an oversized seekable file reports its metadata byte count, while a non-closing stream is read only through the first byte beyond the limit, without parsing fabricated text. | Test (TC-020, TC-021) |
| FR-005-AC-4 | Every byte of retained candidate evidence is preserved unmodified and remains readable through the Engineering Assurance compatibility mapping, and release authority remains a human's. | Test (TC-025) |

## Dependencies

Depends on FR-003, FR-004, and PGM-01.
