# Dialects

tl-parse reads four ASCII dialects. Each is closed: it accepts exactly its own spellings, and a spelling from another dialect is a diagnostic. The normative text of each is in the dialect documents in this directory; this page summarizes them.

The clean-ascii names are identities, available as constants `DIALECT_REVISION` and `DIALECT_V2_REVISION` to `DIALECT_V4_REVISION`. The library also exports each normative record and its SHA-256 digest (`dialect_digest`, `dialect_v2_digest`, `dialect_v3_digest`).

## Lexical forms shared by all dialects

| Form | Spelling |
| --- | --- |
| Constants | `false`, `true` |
| Proposition | `p` followed by a canonical unsigned 32-bit decimal: `0`, or a digit 1-9 followed by digits |
| Boolean operators | `!`, `&`, `\|`, `->`, `<->` |
| Grouping | `(` and `)` |
| Interval | `[a,b]` with canonical `u32` bounds, inclusive, and `a <= b` |
| Whitespace | space, tab, carriage return, line feed, between tokens only |

Keywords are case sensitive. Propositions are numeric. Application names and Unicode syntax are outside every dialect.

## v1: bounded future

Identity `tl-parse.clean-ascii/v1`. Prefix operators `!`, `F[a,b]` and `G[a,b]`; infix `U[a,b]` and `R[a,b]`.

Precedence from tightest to loosest: prefix operators, then `U` and `R`, then `&`, `|`, `->`, `<->`. `U`, `R`, `&`, `|` and `<->` associate left. `->` associates right. Other grammars may associate `U` and `R` to the right, so parenthesize chains when exchanging text with them.

The semantic profile is `closed` (`ClosedTraceV1`) or `online` (`OnlinePrefixV1`). The grammar is the same; the profile is preserved in the report and the document. The report type is `ParseReport`, with schema `tl-parse.diagnostics/v1`.

## v2: derived future operators

Identity `tl-parse.clean-ascii/v2`. v1 plus `W[a,b]` (weak until) and `M[a,b]` (strong release), which share the precedence and left associativity of `U` and `R`. `X`, `Y`, `O`, `H`, `S` and `T` are refused as `unsupported_operator`.

The parser does not build derived nodes. It passes each use to the tl-syntax lowering request and appends the three primitive nodes that come back: `p W[a,b] q` becomes `(p U[a,b] q) | G[a,b] p`, and `p M[a,b] q` becomes `(p R[a,b] q) & F[a,b] p`. `DerivedParseReport.lowerings` has one `LoweringRecord` per use, with the operator span, the expression span, the operand nodes and the generated nodes. Canonical text of the result never contains `W` or `M`, and a v1 parser reads it.

Schema `tl-parse.derived-parse-report/v1`, operator profile `tl-syntax.future-operators/v1`.

## v3: past time

Identity `tl-parse.clean-ascii/v3`. Boolean operators plus `O[a,b]` (once), `H[a,b]` (historically), `Y` (strong previous, no interval), `S[a,b]` (since) and `T[a,b]` (triggered). The future spellings `X F G U R W M` are refused as `unsupported_operator`. `S` and `T` share one left-associative level. The entry point fixes the profile at `OriginCompleteHistoryV1`, and a result is a `tl-syntax.formula/v2` document.

Schema `tl-parse.past-parse-report/v1`, operator profile `tl-syntax.past-operators/v1`.

## v4: infinite trace

Identity `tl-parse.clean-ascii/v4`. It requires the profile `mltl.infinite-trace/v1` and the clock `event_position`; any other value is a refusal. It reads future and past operators together: `F G O H` with an interval, `Y` without one, and `U R W M S T` with an interval. An interval is `[a,b]` or `[a,)`, where `[a,)` has no upper bound. `[a,*]` is outside the dialect. All binary temporal operators share one left-associative level. `W` and `M` are lowered by tl-syntax.

An optional envelope states fairness premises before the formula:

```text
fair { G[0,)p0; O[1,)p1 }: F[0,)p2
```

Each premise is a full v4 formula, a trailing semicolon is accepted, and `fair {}: formula` has no premises. Canonical text omits an empty envelope, parenthesizes every binary expression, and writes `fair {premise; premise}: formula` otherwise. The parser records fairness as syntax and never evaluates it.

The result is an `InfiniteParseReport` with an `InfiniteFormulaDocument`, an optional `FairnessPremisesDocument`, the byte span of every interval and premise, and the decisive refusal with its locus, kept even when diagnostics are capped. `disposition()` maps the report to `NoTemporalVerdict` (a graph was built), `Unsupported`, `ResourceIncomplete` or `Failed` from the typed refusal. Parsing never produces a temporal verdict.

Schema `tl-parse.infinite-parse-report/v1`.

## Choosing a dialect

| Need | Entry point |
| --- | --- |
| Future operators `F G U R` | `parse` |
| `W` and `M` | `parse_clean_ascii_v2` |
| Past operators | `parse_clean_ascii_v3` |
| Unbounded intervals, mixed directions, fairness | `parse_clean_ascii_v4` |

The `tl-parse` binary reads v1 only. See [cli.md](cli.md).
