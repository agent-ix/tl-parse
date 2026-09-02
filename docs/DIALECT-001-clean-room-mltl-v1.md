---
id: DIALECT-001
title: tl-parse clean-room MLTL text dialect v1
type: Standard
code: tl-parse-clean-ascii-v1
status: active
---

# tl-parse clean-room MLTL text dialect v1

## Authorship and provenance

This grammar was independently authored for tl-parse from the operator and
checked-value vocabulary exposed by MIT OR Apache-2.0 dual-licensed
`agent-ix/tl-syntax` at exact
revision `740182f13b84858008d6f176f75136737d405c1b`. No parser implementation,
grammar production, or grammar prose from any third-party MLTL tool was used.
The separately retained `ATTRIBUTION.md` binds the exact tl-syntax files and
license texts consulted by SHA-256 and records the negative clean-room
declaration as a reviewable artifact.

The revision above is the authorship basis and is historical. The revision this
crate compiles against is `953ee825e5060335b4c79682f5f41a78c5a1bfae`, the head
of tl-syntax `main` after that repository merged its own shared-assurance
migration. The two differ only by a bounded wire decoder, a new document
node-limit error variant, `#[non_exhaustive]` markers on four error types, and
the removal of the `Node` `Deserialize` derive. No operator, interval, span, or
`NodeKind` in the vocabulary this grammar was authored from is added, removed,
or renamed, so the grammar below is unchanged by the repin. `ATTRIBUTION.md`
carries a SHA-256 table for both revisions.

Stable dialect identity: `tl-parse.clean-ascii/v1`. The implementation exposes
a SHA-256 digest over the normative production and precedence record so drift
is detectable by tests and evidence.

## Normative lexical record

```text
FALSE       = "false"
TRUE        = "true"
PROPOSITION = "p" CANONICAL_U32
NOT         = "!"
AND         = "&"
OR          = "|"
IMPLIES     = "->"
EQUIVALENT  = "<->"
FUTURE      = "F" INTERVAL
GLOBALLY    = "G" INTERVAL
UNTIL       = "U" INTERVAL
RELEASE     = "R" INTERVAL
INTERVAL    = "[" CANONICAL_U32 "," CANONICAL_U32 "]"
WHITESPACE  = ASCII space, tab, carriage return, or line feed
```

`CANONICAL_U32` is `0` or a decimal digit from 1 through 9 followed by zero or
more decimal digits, with numeric value at most 4294967295. Keywords are case
sensitive. Every other identifier or character is outside the dialect.

## Normative syntax and precedence

Parentheses may group any expression. Prefix `!`, `F`, and `G` bind tightest.
Infix `U` and `R` bind next, then `&`, then `|`, then right-associative `->`,
then `<->`; infix operators other than `->` associate left.

The left association of `U` and `R` is specific to this dialect. Consumers
targeting grammars that associate those operators to the right must preserve
the intended tree with explicit parentheses when exchanging text; unparenthesized
chains are not assumed to be portable between dialects.

The accepted syntax is equivalent to the following independently authored
precedence description:

```text
equivalent  := implication ("<->" implication)*
implication := disjunction ("->" implication)?
disjunction := conjunction ("|" conjunction)*
conjunction := temporal ("&" temporal)*
temporal    := prefix (("U" | "R") INTERVAL prefix)*
prefix      := "!" prefix
             | ("F" | "G") INTERVAL prefix
             | FALSE | TRUE | PROPOSITION | "(" equivalent ")"
```

Every accepted interval is inclusive and is rejected when start exceeds end.
The selected semantic profile is an API/CLI parameter, never inferred from
text. Application proposition names and Unicode syntax are deliberately out of
profile rather than silently normalized.

## Canonical rendering

Canonical text is whitespace-free and uses the precedence and associativity
above. Parentheses appear only when their removal would change the syntax tree;
prefix operands are parenthesized only when they are infix expressions. This
keeps accepted prefix and same-associativity chains within the same parser depth
budget after formatting.
