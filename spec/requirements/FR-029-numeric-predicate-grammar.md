---
id: FR-029
title: Parse numeric predicates into the predicate table
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: implements
  - target: ix://agent-ix/tl-parse/FR-028
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-302
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-304
    type: depends_on
---

# FR-029: Parse numeric predicates into the predicate table

## Description

When v5 text contains a parenthesized comparison, tl-parse shall type it against
the supplied catalog through tl-syntax, allocate a proposition for it, and emit
the formula leaf and the predicate-table entry, or refuse with typed diagnostics.

## Behavior

Grammar of the numeric part (ASCII):

```
atom        = "(" arith cmp arith ")"      (first token: identifier, number, "-", "abs", "scale")
cmp         = "<" | "<=" | ">" | ">=" | "==" | "!="
arith       = term { ("+" | "-") term }      left-associative
term        = unary { "*" unary }            left-associative
unary       = "-" unary | primary
primary     = identifier | integer | decimal | "abs" "(" arith ")"
            | "scale" "(" arith "," integer ")" | "(" arith ")"
integer     = [ "-" ] digit+              canonical: no leading zero, no "-0"
decimal     = [ "-" ] digit+ "." digit{1,18}   scale is the digit count, trailing zeros kept
identifier  = [A-Za-z_][A-Za-z0-9_]{0,254}     exact catalog name bytes
```

- A comparison is always parenthesized and non-associative; `a < b < c` is
  `non_associative_comparison`; a bare comparison outside parentheses is
  `unparenthesized_comparison`.
- Boolean atoms keep their `p<u32>` spelling. An identifier of the shape
  `p<digits>`, or a reserved word (`true false each some atleast in set contract
  assume guarantee abs scale`), used as a signal name is
  `reserved_identifier`; reserved words cannot be catalog names in v5.
- A literal is exactly its own type: `5` is scale 0, `5.00` is scale 2.
  There is no implicit rescale; `scale(e, k)` is the explicit widening.
- **Minus is lexed by position.** `-` is part of a numeric literal only where an
  operand is expected (start of an arithmetic expression, after `(`, after a binary
  operator or after `,`) and a digit follows immediately; after an operand (an
  identifier, a number or `)`) it is always the binary subtraction operator, and before
  anything else at an operand position it is `neg`. So `x -5` is `x - 5` (one
  subtraction), `x - -5` is `x` minus the literal `-5`, and `-x` is `neg x`.
- **Parenthesis disambiguation.** A `(` opens a comparison exactly when a comparison
  operator occurs at that parenthesis's own depth before its matching `)`; the scan is
  one pass bounded by the token limit and never backtracks. A parenthesised item
  without a top-level comparison is a parenthesised formula (`(p1 & p2)`), and a
  parenthesised arithmetic expression with no comparison at formula position (`(x)`)
  is `missing_token` for the absent comparison.
- **Reserved names.** A signal whose catalog name is a reserved word or has the shape
  `p<digits>` is not usable in v5: the catalog stays valid in tl-syntax, and v5 refuses
  only a source that references the name (`reserved_identifier`). There is no quoting
  escape; the remedy is to rename the signal in the catalog (ADR-020).
- Division, modulo, power, floats with exponent, hexadecimal, digit separators and
  non-ASCII digits or minus signs are `unsupported_operator` or
  `unexpected_character` at their byte.
- Typing is performed by tl-syntax FR-302 against the catalog; its errors map
  one-to-one to `unknown_signal`, `non_numeric_signal`, `scale_mismatch`,
  `scale_overflow`, `rescale_narrowing`, `range_overflow`, with the byte locus of
  the offending sub-expression.
- Proposition allocation: predicate propositions outside quantified templates are
  allocated in order of first occurrence in a left-to-right scan from `1 + max(explicit
  pN ids in the whole source, catalog binding ids)` (checked against `u32`); two
  textually distinct but structurally identical comparisons share one proposition.
  Member-dependent predicates inside a quantified template are not allocated here:
  they take consecutive ranges from the tl-syntax expander, which tl-parse starts
  strictly above every identity allocated by this rule, in source order (tl-syntax
  FR-307 "Identity precedence").
- Resource limits (source bytes, tokens, nodes, depth, work) apply to the
  arithmetic nodes as well and clamp to the existing hard maxima; expression
  nesting counts toward the 256 depth limit.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-029-AC-1 | Every operator, node kind, literal scale 0, 1, 2, 9, 18 and the `scale` widening parse to the exact predicate-table content declared by the corpus, with deterministic proposition allocation and deduplication. | integration-testing (TC-1543) |
| FR-029-AC-2 | Each typing and syntax refusal listed produces its own code at the declared byte locus, and none yields a graph; `x -5` parses as one subtraction, `x - -5` and `-x` as specified, `(p1)` as a parenthesised formula and `(x)` at formula position as `missing_token`, and a source that references a reserved-word catalog name is refused while the same catalog serves a source that does not reference it. | unit-testing (TC-1544) |
| FR-029-AC-3 | Parsing is a total function of (source, catalog, profile, limits): generated sources of bounded size never panic and always return a graph or typed diagnostics. | property-based-testing (TC-1545) |

## Dependencies

tl-syntax FR-302 (typing), FR-304 (binding).
