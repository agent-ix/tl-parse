---
id: FR-001
title: Define and tokenize the clean-room MLTL dialect
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-001
    type: implements
---

# FR-001: Define and tokenize the clean-room MLTL dialect

## Description

The library shall expose a versioned, independently authored ASCII dialect and
tokenize it deterministically with half-open UTF-8 byte spans.

## Behavior

- Constants are `false` and `true`; propositions are `p` plus canonical
  unsigned decimal u32 digits.
- Prefix operators are `!`, `F[start,end]`, and `G[start,end]`.
- Infix operators are `&`, `|`, `U[start,end]`, `R[start,end]`, `->`, and `<->`.
- Parentheses group expressions. Whitespace is ASCII space, tab, CR, or LF.
- From tightest to loosest, precedence is prefix, temporal binary, `&`, `|`,
  right-associative `->`, then `<->`; all other infix classes associate left.
- Unknown identifiers, non-ASCII tokens, non-canonical numbers, and constructs
  outside the closed grammar are rejected rather than reinterpreted.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-001-AC-1 | The full token vocabulary, precedence, associativity, and whitespace behavior match the checked-in dialect revision. | Test (TC-001, TC-002) |
| FR-001-AC-2 | Invalid characters, identifiers, and numbers are rejected at stable half-open UTF-8 byte spans. | Test (TC-003, TC-004) |
| FR-001-AC-3 | The dialect record states its independent authorship, exact tl-syntax vocabulary pin, and digest-bearing revision. | Inspection (TC-020) |

## Dependencies

Depends only on PGM-01 and the exact pinned tl-syntax operator vocabulary.
