---
id: DIALECT-005
title: tl-parse clean ASCII numeric predicate dialect v5
type: Standard
code: tl-parse-clean-ascii-v5
status: draft
---

# Clean ASCII Numeric Predicate Dialect v5

Status: draft, UNBUILT (TL-421). Specified by tl-parse FR-028 to FR-033 and
ADR-020. The P-TL-150/P-TL-151 identity collision is resolved by the TL
program decision of 2026-11-23: the numeric dialect keeps
`tl-parse.clean-ascii/v5`; the plain LTL dialect is `tl-parse.clean-ascii/v6`.

## Identity

`tl-parse.clean-ascii/v5`, revision number 1 (`DIALECT_V5_REVISION_NUMBER`).
No dialect digest record exists for v5 (tl-syntax ADR-019 register).
The dialect is selected explicitly through `parse_clean_ascii_v5` and is always
context-bound: a validated `tl-syntax.signal-catalog/v1` document is a required
argument. It admits only the profiles `mltl.closed-trace/v1` and
`mltl.online-prefix/v1`; any other profile is `profile_mismatch` before lexing.

Text in this dialect is derived interchange. FSL is the only editable
formal-clause authority, and text authored by hand is unattributed: a result
derived from it makes no claim about an FSL requirement.

## Grammar

The temporal skeleton is the v2 vocabulary: `false`, `true`, `p<canonical-u32>`,
`!`, `&`, `|`, `->`, `<->`, `F[a,b]`, `G[a,b]`, `U[a,b]`, `R[a,b]`, `W[a,b]`,
`M[a,b]` (the last two lowered through tl-syntax), with v1 precedence.
Past operators, `X`, unbounded intervals `[a,)` and the fairness envelope are
`unsupported_operator`.

A numeric atom is a parenthesized, non-associative comparison:

```
atom        = "(" arith cmp arith ")"
cmp         = "<" | "<=" | ">" | ">=" | "==" | "!="
arith       = term { ("+" | "-") term }
term        = unary { "*" unary }
unary       = "-" unary | primary
primary     = identifier | integer | decimal | "abs" "(" arith ")"
            | "scale" "(" arith "," integer ")" | "(" arith ")"
integer     = [ "-" ] digit+                 no leading zero, no "-0"
decimal     = [ "-" ] digit+ "." digit{1,18}   scale = fractional digit count
identifier  = [A-Za-z_][A-Za-z0-9_]{0,254}    exact catalog name bytes
```

- Minus is lexed by position: it begins a literal only where an operand is
  expected and a digit follows immediately. `x -5` is one subtraction,
  `x - -5` subtracts the literal `-5`, `-x` is negation.
- A `(` opens a comparison exactly when a comparison operator occurs at its own
  depth before its matching `)` (one stack pass over the tokens).
- Reserved words (`true false each some atleast in set contract assume
  guarantee abs scale`, the operator letters `F G U R W M X O H Y S T`, `fair`)
  and `p<digits>` cannot name a signal: a source referencing such a catalog
  name is `reserved_identifier`. There is no quoting escape.
- No implicit rescale: `scale(e, k)` is the explicit widening. Typing follows
  tl-syntax FR-302 against the catalog.

Set aggregation and contract declarations (FR-032) are reserved for TL-424 and
TL-425; their keywords are currently `unsupported_operator`.

## Proposition identities

Predicates are allocated in order of first occurrence from
`1 + max(explicit pN ids, catalog binding ids)` (from 0 when both sets are
empty); structurally identical comparisons share one identity.

## Canonical text

The skeleton prints as v1 (no spaces, minimum parentheses). A predicate prints
as `(left op right)` with one space around the comparison and binary
arithmetic operators, nested binary arithmetic parenthesized, `abs(e)`,
`scale(e, k)`, decimals with exactly `scale` fractional digits, and a negated
operand that is not a name or call parenthesized (`-(5)`).

## Diagnostics

`tl-parse.numeric-diagnostics/v1` (closed): `source_limit`, `token_limit`,
`node_limit`, `depth_limit`, `work_limit`, `unexpected_character`,
`unexpected_token`, `missing_token`, `trailing_input`, `non_canonical_number`,
`integer_overflow`, `unsupported_operator`, `profile_mismatch`,
`unknown_signal`, `non_numeric_signal`, `scale_mismatch`, `scale_overflow`,
`rescale_narrowing`, `range_overflow`, `reserved_identifier`,
`non_associative_comparison`, `unparenthesized_comparison`, `undefined_set`,
`duplicate_set`, `slot_outside_template`, `aggregation_too_large`,
`duplicate_contract`, `missing_contract_clause`, `proposition_overflow`.
