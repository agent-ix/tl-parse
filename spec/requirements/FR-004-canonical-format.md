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
- Prefix operators include their interval where applicable. Canonical output
  uses the authored precedence and associativity with parentheses only where
  removing them would change the syntax tree, so formatting does not consume a
  deeper parse budget than the accepted source.
- Decimal proposition and interval values contain no sign or leading zero
  except the value zero.
- Formatting ignores source spans and preserves node kinds, bounds,
  proposition identities, root meaning, and semantic profile on reparse.
- Canonical text is emitted into one iterative output buffer rather than
  retaining a complete string for every intermediate node. Output bytes are
  checked before emission. Formatter work charges one unit per expanded node
  plus one per emitted byte. The node, output-byte, and work ceilings are joint
  limits: depending on graph shape, output or cumulative work may reject a
  formula below the node ceiling even when another shape at that node count is
  accepted.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-004-AC-1 | Every operator has one exact canonical rendering and formatting canonical text twice is idempotent. | Test (TC-014, TC-015) |
| FR-004-AC-2 | A deterministic generated formula population parse-formats-parses without structural or profile drift. | Test (TC-016) |
| FR-004-AC-3 | Deep/shared graphs and output/work exhaustion are handled iteratively and return explicit errors without partial output; the tested 9,999-node shared-boundary shape formats under default limits while chain shapes may reach the joint work limit earlier. | Test (TC-017) |

## Dependencies

Depends on FR-002 and FR-003.
