---
id: DIALECT-003
title: tl-parse clean ASCII past-time dialect v3
type: Standard
code: tl-parse-clean-ascii-v3
status: active
---

# Clean ASCII Past-Time Dialect v3

## Identity and scope

This document defines `tl-parse.clean-ascii/v3`. The dialect is an internal,
independently authored ASCII representation for Boolean formulas plus the
closed past-time operator set `O`, `H`, `Y`, `S`, and `T`. It is not a native
Quire source language and makes no independent claim about temporal semantics.

Successful parsing produces `tl-syntax.formula/v2` under semantic profile
`mltl.origin-complete-history/v1` and operator catalog
`tl-syntax.past-operators/v1`. A successful document cannot contain a
future-time operator.

## Lexical forms

The accepted spellings are case-sensitive:

- constants: `false`, `true`
- propositions: `p` followed by a canonical unsigned 32-bit decimal integer
- Boolean operators: `!`, `&`, `|`, `->`, `<->`
- bounded unary past operators: `O[lower,upper]`, `H[lower,upper]`
- strong Previous: `Y`
- bounded binary past operators: `S[lower,upper]`, `T[lower,upper]`
- grouping: `(` and `)`

An interval bound is canonical decimal: `0`, or a non-zero digit followed by
zero or more digits. Both bounds must fit in `u32`, the bounds are inclusive,
and `lower` must not exceed `upper`. Space, tab, carriage return, and line feed
may occur between tokens.
Whitespace is not permitted inside a keyword or numeric literal.

The future spellings `X`, `F`, `G`, `U`, `R`, `W`, and `M` are rejected as
unsupported operators. Weak Previous, long operator names, aliases, and
lowercase operator spellings are not part of this dialect and are rejected.

Boolean keywords have lexical boundaries. A proposition literal ends after its
decimal digits; any following bytes are tokenized independently. Operator
letters that require an interval are recognized only in their operator position
and must be followed by an interval. `Y` is the only interval-free temporal
prefix.

## Grammar and precedence

The grammar below is descriptive. Bracketed bounds are mandatory where shown.

```text
formula      = equivalent
equivalent   = implies ("<->" implies)*
implies      = disjunction ("->" implies)?
disjunction  = conjunction ("|" conjunction)*
conjunction  = temporal ("&" temporal)*
temporal     = prefix (("S" | "T") interval prefix)*
prefix       = "!" prefix
             | ("O" | "H") interval prefix
             | "Y" prefix
             | primary
primary      = "false" | "true" | proposition | "(" formula ")"
interval     = "[" canonical-u32 "," canonical-u32 "]"
proposition  = "p" canonical-u32
```

Prefix operators bind most tightly. `S` and `T` share one precedence level and
associate left. `&`, `|`, and `<->` associate left. `->` associates right.

## Source spans

Leaf spans cover their complete lexical token. A prefix-node span begins at
its operator token and ends at the complete operand extent. A binary-node span
begins at the complete left operand extent and ends at the complete right
operand extent. Parentheses create no graph node, but their complete extent is
used when determining an enclosing operator span.

All offsets are UTF-8 byte offsets. Invalid input produces diagnostics and no
document.

## Canonical formatting

Canonical output uses exactly the spellings above, canonical decimal integers,
and no whitespace. Parentheses are emitted only when required to preserve the
parsed tree under the declared precedence and associativity rules.

The clean-ascii/v3 formatter accepts only validated `tl-syntax.formula/v2`
documents under `mltl.origin-complete-history/v1`. A profile, schema, graph,
work, or output-limit mismatch fails closed and produces no text.

## Resource and report behavior

Parsing and formatting use the crate's clamped source, token, node, depth,
diagnostic, work, and output limits. Limit exhaustion and malformed input are
reported through stable diagnostic/error codes without unwinding. A parse
report carries the exact dialect, report-schema, operator-profile, semantic
profile, and compiled `tl-syntax` revision identities. A document is present
only when the parse is diagnostic-free and validates under the pinned syntax
model.
