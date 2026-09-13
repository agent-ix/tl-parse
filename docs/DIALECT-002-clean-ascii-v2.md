---
id: DIALECT-002
title: tl-parse clean-room MLTL text dialect v2
type: Standard
code: tl-parse-clean-ascii-v2
status: active
---

# tl-parse clean-room MLTL text dialect v2

## Identity and relation to v1

Stable dialect identity: `tl-parse.clean-ascii/v2`. It is an internal input
dialect selected explicitly by API. It is never inferred from text, and it
does not change `tl-parse.clean-ascii/v1`. The v1 record in
`DIALECT-001-clean-room-mltl-v1.md` stays closed: its parser still rejects
every v2-only spelling.

v2 is the v1 record plus two derived infix operators. Every v1 production,
token, interval rule, whitespace rule, and precedence rule is unchanged.

## Authorship and provenance

The v1 productions keep the clean-room basis that `DIALECT-001` and
`ATTRIBUTION.md` record. The two additions were authored from requirements
only:

- `agent-ix/tl-syntax` FR-009, which fixes the spellings, precedence,
  associativity, and interval syntax;
- the public `tl-syntax.future-lowering-request/v1` API, which fixes the
  lowering.

No third-party parser, grammar, or grammar prose was consulted.

## Normative lexical additions

```text
WEAK_UNTIL      = "W" INTERVAL
STRONG_RELEASE  = "M" INTERVAL
UNSUPPORTED     = "X" | "Y" | "O" | "H" | "S" | "T"
```

Keywords are case sensitive and must stand alone as identifiers. `w`, `m`,
`WU`, `WeakUntil`, and every other identifier are unknown, exactly as in v1.

`UNSUPPORTED` names operators that `tl-syntax.future-operators/v1` refuses:

- next: `X`;
- the past/history family: `Y`, `O`, `H`, `S`, `T`.

In v2 these are refused as unsupported operators, not as unknown identifiers,
with or without a following interval. In v1 they remain unknown identifiers.

## Normative syntax and precedence

```text
temporal := prefix (("U" | "R" | "W" | "M") INTERVAL prefix)*
```

All other productions are those of DIALECT-001. `W` and `M` share the `U`/`R`
binding power and associate left with them, so `p0 U[0,1] p1 W[2,3] p2` means
`(p0 U[0,1] p1) W[2,3] p2`.

An interval is mandatory and checked exactly as in v1:

- canonical unsigned 32-bit bounds;
- inclusive;
- rejected when start exceeds end.

## Spans

Each derived operator carries two half-open UTF-8 byte spans:

- **operator span:** from the first byte of the `W` or `M` keyword to the last
  byte of its closing interval bracket, including any whitespace inside the
  interval;
- **expression span:** from the extent start of the left operand to the extent
  end of the right operand, including grouping parentheses, as for `U` and
  `R`.

The operator span always lies within the expression span.

## Lowering

The parser never builds a derived node. After both operands are parsed, it
submits the operands, interval, and both spans to the pinned tl-syntax
`FutureLoweringRequest` with the selected semantic profile. It then appends
exactly the three returned primitive nodes:

- `p W[a,b] q` becomes `U[a,b](p,q)`, `G[a,b](p)`, `Or`;
- `p M[a,b] q` becomes `R[a,b](p,q)`, `F[a,b](p)`, `And`.

The three nodes carry the expression span.

## Canonical rendering

Canonical text is the DIALECT-001 rendering of the lowered primitive graph. It
never contains `W` or `M`. Old v1 parsers therefore accept it, and parsing it
under v2 yields the same text again.
