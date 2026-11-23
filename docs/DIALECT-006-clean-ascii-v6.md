---
id: DIALECT-006
title: tl-parse clean ASCII plain LTL and LTLf dialect v6
type: Standard
code: tl-parse-clean-ascii-v6
status: draft
---

# Clean ASCII Plain LTL and LTLf Dialect v6

Status: draft, UNBUILT (TL-429). Specified by tl-parse FR-058 to FR-062 and
ADR-041, which name this dialect `v5`; the TL program decision of 2026-11-23
renumbered it to `tl-parse.clean-ascii/v6` because the numeric dialect keeps
`v5`. Following that decision's register rule, v6 has no dialect digest record.

## Identity and profile

`tl-parse.clean-ascii/v6`, selected through `parse_clean_ascii_v6`. The semantic
profile is a required caller argument `LtlProfile::{LtlfFiniteTrace,
InfiniteTrace}`. It has no default, is never inferred from spelling, and the
source text carries no profile marker.

| Profile | Graph edition | Reading |
|---|---|---|
| `mltl.ltlf-finite-trace/v1` | `tl-syntax.formula-ltlf/v1` | `F G U R` to the last position; `Xs`, `Xw` |
| `mltl.infinite-trace/v1` | `tl-syntax.formula-unbounded/v1` | `F p` is `F[0,) p` (likewise `G U R`); `X p` is `F[1,1] p` |

## Grammar

`false`, `true`, `p<canonical-u32>`, `!`, `&`, `|`, `->`, `<->`, parentheses,
prefix `F`, `G`, infix `U`, `R`, and next: `X` (infinite profile only) or `Xs`,
`Xw` (LTLf only). Keywords are case-sensitive and lexed by maximal munch:
`Gp0` is `G p0`, `Xsp0` is `Xs p0`. Precedence is that of v1: prefix, then
`U`/`R` (left), `&`, `|`, `->` (right), `<->` (left).

Refusals: interval brackets (`interval_not_in_dialect`, naming dialects v1 to
v4), `W M O H Y S T` (`unsupported_operator`), bare `X` under LTLf
(`next_strength_required`, naming `Xs` and `Xw`), `Xs`/`Xw` under the infinite
profile (`next_strength_undefined`), unknown words such as `Xx`
(`unknown_identifier`).

## Canonical text

Minimum parentheses under the v1 rules and single spaces: `!p0`, `G p0`,
`Xs p0`, `p0 U p1`, `G (p0 & p1)`. Under the infinite profile the image is
`F/G/U/R` over `[0,)` and `F[1,1]` (spelled `X`); `G[1,1]`, any other interval
and past nodes are refused as `not_expressible`, never rewritten.

## Diagnostics

`source_limit`, `token_limit`, `node_limit`, `depth_limit`, `work_limit`,
`unexpected_character`, `unknown_identifier`, `non_canonical_number`,
`integer_overflow`, `unexpected_token`, `missing_token`, `trailing_input`,
`unsupported_operator`, `interval_not_in_dialect`, `next_strength_required`,
`next_strength_undefined`, `profile_mismatch`, `validation_failure`.
