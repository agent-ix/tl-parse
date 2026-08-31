---
id: FR-004
title: Format one canonical representation and preserve round trips
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-002
    type: implements
---

# FR-004: Format one canonical representation and preserve round trips

## Description

The formatter shall render any supported validated formula into one explicit,
whitespace-free representation within declared output and work limits.

## Behavior

- Constants and propositions are canonical atoms.
- Prefix operators include their interval where applicable and parenthesize
  their operand; every binary operator is fully parenthesized.
- Decimal proposition and interval values contain no sign or leading zero
  except the value zero.
- Formatting ignores source spans and preserves node kinds, bounds,
  proposition identities, root meaning, and semantic profile on reparse.
- Output bytes and formatting work are checked before emission.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-004-AC-1 | Every operator has one exact canonical rendering and formatting canonical text twice is idempotent. | Test (TC-014, TC-015) |
| FR-004-AC-2 | A deterministic generated formula population parse-formats-parses without structural or profile drift. | Test (TC-016) |
| FR-004-AC-3 | Deep/shared graphs and output/work exhaustion are handled iteratively and return explicit errors without partial output. | Test (TC-017) |

## Dependencies

Depends on FR-002 and FR-003.
