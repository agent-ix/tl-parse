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
