---
id: FR-002
title: Parse directly into validated tl-syntax graphs
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-001
    type: implements
---

# FR-002: Parse directly into validated tl-syntax graphs

## Description

The parser shall append source-spanned nodes directly to one topologically
ordered tl-syntax formula document and shall expose no second public AST.

## Behavior

- Both tl-syntax semantic profiles are selectable and preserved unchanged.
- Inclusive interval bounds are constructed through `tl_syntax::Interval`.
- Proposition text maps exactly to its numeric `PropositionId`.
- Constants and every Boolean and temporal operator map one-for-one to the
  corresponding `NodeKind`.
- A document is returned only after exact tl-syntax structural validation and
  only when no lexical, syntactic, or resource diagnostic occurred.
- Parentheses shall group an expression without replacing the grouped node's
  own lexical source span, while every enclosing operator span shall include
  the complete grouped operand extent, including both delimiters.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-002-AC-1 | Primitive, nested, precedence-sensitive, and complete-vocabulary inputs produce the specified topologically ordered node kinds and spans. | Test (TC-005, TC-006) |
| FR-002-AC-2 | Every successful result validates through the exact pinned tl-syntax revision and preserves the requested semantic profile. | Test (TC-007) |
| FR-002-AC-3 | Missing operands, delimiters, or trailing tokens retain recovery diagnostics and never expose a partial document. | Test (TC-008) |
| FR-002-AC-4 | Grouping parentheses preserve the grouped node's delimiter-free lexical source span, and every enclosing unary or binary operator span includes the complete balanced grouping extent. | Test (TC-030) |

## Dependencies

Depends on FR-001 and exact tl-syntax revision `26b801d4a68ebfe720062cfdb3c66b070ab60e92`.
