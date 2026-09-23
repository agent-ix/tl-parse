---
id: DIALECT-004
title: tl-parse clean ASCII infinite-trace dialect v4
type: Standard
code: tl-parse-clean-ascii-v4
status: active
---

# Clean ASCII Infinite-Trace Dialect v4

## Identity

`tl-parse.clean-ascii/v4` requires `mltl.infinite-trace/v1` and the
`event_position` clock. It produces `tl-syntax.formula-unbounded/v1` and,
when premises are present, a same-graph fairness document. It does not
evaluate formulas or decide fairness.

## Grammar

V4 retains the Boolean constants, canonical `p` plus unsigned decimal atoms,
`!`, `&`, `|`, `->`, `<->`, and parentheses. It admits the unary temporal
operators `F`, `G`, `O`, `H` with a required interval, `Y` without an
interval, and the binary temporal operators `U`, `R`, `W`, `M`, `S`, `T` with
a required interval. A temporal interval is `[a,b]` or `[a,)`, where bounds
are canonical `u32` decimals and `a <= b` for a closed interval. `[a,*]`
is outside this dialect. `W` and `M` are lowered by tl-syntax before owner
graph construction.

An optional envelope has the form `fair { premise; premise }: formula`.
Each premise is a complete v4 formula. A trailing semicolon is accepted.
`fair {}: formula` means no premises. Canonical formatting omits the empty
envelope and emits `fair {premise; premise}: formula` for nonempty premises.

Temporal prefix operators bind more tightly than temporal binary operators.
`U`, `R`, `W`, `M`, `S`, and `T` share one left-associative precedence level,
then `&`, `|`, right-associative `->`, and left-associative `<->` follow.
The formatter parenthesizes every binary expression to preserve its tree.

## Reports and resources

Every interval and premise has a half-open UTF-8 byte span. On refusal the
parser returns typed diagnostics and no graph. Source, token, node, nesting,
diagnostic, work, and output limits are clamped to process-safe maxima.
The v4 report records the selected profile, clock, dialect, compiled owner
revision, effective limits, graph, fairness binding, spans, and diagnostics.
Canonical report JSON is strictly read under artifact limits.
