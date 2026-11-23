---
id: ADR-041
title: "Add a plain LTL dialect v5 whose semantic profile is always selected by the caller"
type: ADR
status: proposed
owner: tl-parse-maintainer
relationships:
  - target: ix://agent-ix/tl-parse/StR-044
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-058
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-059
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-044
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-045
    type: depends_on
---

# ADR-041: Add a plain LTL dialect v5 whose semantic profile is always selected by the caller

## Status

Proposed (TL-427). Ready for human acceptance; not accepted.

## Context

Users write `G p`, `F p`, `p U q` and `X p` without interval brackets. The same
characters mean different things under LTLf (finite, truncating, two next
operators) and under the infinite-trace profile (no end, one next). The
dialects v1 to v4 are closed, digest-recorded and must not change
(FR-009). A parser that guessed the profile from the spelling would return
different graphs for one source text depending on which operators happened to
appear, and `F p0` is valid in both.

## Decision

1. Add a fifth dialect `tl-parse.clean-ascii/v5`, owned by a new module
   `dialect::v5`, with its own record, digest and document
   `docs/DIALECT-005-clean-ascii-v5.md`. v1 to v4 text, records, digests and
   reports do not change.
2. The semantic profile is a required typed argument
   `LtlProfile::{LtlfFiniteTrace, InfiniteTrace}` to parse and format. There is
   no default, no `Option`, no inference from spelling, and no profile token in
   the source text.
3. Spellings: `!`, `&`, `|`, `->`, `<->`, `true`, `false`, `p<canonical-u32>`,
   prefix `F`, `G`, infix `U`, `R`, and next as `X` (infinite profile only; it
   lowers to the tl-syntax `F[1,1]` node) or `Xs` and `Xw` (LTLf profile only).
   Interval brackets are never accepted in v5; a user needing intervals selects
   v1 to v4.
4. Under LTLf a bare `X` is refused with a diagnostic naming `Xs` and `Xw`.
   Under the infinite profile `Xs` and `Xw` are refused, because with no end of
   trace there is nothing for the strength to mean.
5. Precedence and associativity are those of v1 (prefix, then `U`/`R` left, `&`,
   `|`, `->` right, `<->` left).
6. Import front-ends for foreign formats (FRET MLTL, C2PO, WEST) are **not** in
   tl-parse: they live in tl-mltl (tl-mltl ADR-046), keeping tl-parse a
   clean-room parser for its own dialects (NFR-002 provenance boundary).

## Alternatives rejected

- Infer the profile from operators present: ambiguous for `F p0`.
- Encode the profile in a source header (`ltlf: G p`): makes the text carry a
  second authority that the caller's selection could contradict.
- Accept `X` as weak or strong under LTLf: a silent default (tl-syntax ADR-045).
- Extend v4 with bracket-free spellings: v4 is closed and digest-recorded.

## Consequences

New public entry points and new report schema `tl-parse.ltl-parse-report/v1`;
new fuzz target and hostile corpus; existing public API unchanged.
