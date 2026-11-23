---
id: ADR-020
title: "Add a numeric text dialect v5 that is derived interchange and never an editing authority"
type: ADR
status: proposed
owner: tl-parse-maintainer
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-028
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-029
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-032
    type: realizes_requirement
  - target: ix://agent-ix/tl-parse/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-014
    type: depends_on
---

# ADR-020: Add a numeric text dialect v5 that is derived interchange and never an editing authority

## Status

Proposed (TL-419, project P-TL-150). Ready for human acceptance; not accepted. This
is the tl-parse ADR-020. The ADR-014 and ADR-015 it depends on are tl-syntax records
(`tl-syntax/ADR-014`), and the StR-014 of this repository is not the StR-014 of tl-mltl.

## Context

The numeric model of tl-syntax (FR-302 onward) is reachable only through hand-built
documents unless a text form exists. Dialects v1 to v4 are closed and must not change
(FR-009). A text form also invites the reading that the text is the requirement; it is
not, because FSL is the only editable formal-clause authority (tl-syntax/ADR-014
decision 6, a proposed amendment of tl-syntax StR-003).

## Decision

1. Add `tl-parse.clean-ascii/v5`, explicitly selected, always context-bound (a
   catalog is required), admitting the `mltl.closed-trace/v1` and
   `mltl.online-prefix/v1` profiles only. Past operators, unbounded intervals and
   fairness are refused as `unsupported_operator`; numeric past requirements therefore
   have no text form in v5 (tl-mltl FR-055 and tl-solve ADR-200 evaluate and encode
   them through documents only). Extending v5 to those profiles is a later revision.
2. The dialect is identified by the string `tl-parse.clean-ascii/v5` and the integer
   `DIALECT_V5_REVISION`; it has no dialect digest record, unlike v4 (see the
   tl-syntax/ADR-019 register, item 5), and its report echoes the catalog structurally
   (schema identity, signal count, binding count) rather than by digest (item 6).
3. A minus sign is lexed by position, not by adjacency (FR-029): it is part of a
   numeric literal only where an operand is expected and a digit follows immediately;
   after an operand it is subtraction. A parenthesis opens a comparison exactly when a
   comparison operator occurs at its own nesting depth before the matching close
   (bounded by the token limit, one pass, no backtracking).
4. Reserved words cannot be used as signal names and there is no quoting escape.
   A catalog that contains such a name is still a valid tl-syntax catalog; v5 refuses
   only a source that references that name (`reserved_identifier`). The remedy is to
   rename the signal in the catalog.
5. Proposition identities: predicates outside quantified templates are allocated by
   first occurrence from `1 + max` of the explicit atoms and catalog bindings;
   quantified templates take consecutive ranges above that, in source order, through
   the tl-syntax expander (tl-syntax FR-307 "Identity precedence"). Contracts parse
   to a root that is the left-balanced conjunction of the obligations, with the
   structure carried by the contract-set document.
6. Source text accepted by v5 is derived interchange. v5 source authored by hand is
   unattributed (tl-syntax/ADR-014 decision 6): a result derived from it makes no
   claim about an FSL requirement. The text has no requirement-context fields.

## Open decision (not made here)

P-TL-151 (`TL-151-spec-tl-parse`, ADR-041, FR-058) also claims the identity
`tl-parse.clean-ascii/v5` and `docs/DIALECT-005-clean-ascii-v5.md` for a plain LTL
surface dialect. Two dialects cannot share one identity. One of the two projects must
take `v6`, and which one is a human decision between P-TL-150 and P-TL-151; until it
is made, every `v5` in this repository's numeric specification means the numeric
dialect and is provisional in name. Recommendation: renumber whichever specification
is accepted second, and do not edit either until that is recorded.

## Alternatives rejected

- Extending v2 or v4 with numeric syntax: rejected, FR-009 forbids moving any
  existing dialect.
- Guessing a quoted-identifier escape for reserved names: rejected for v1 of the
  dialect; a catalog rename is cheaper than a second lexical mode.

## Consequences

The numeric text form is small and closed, covers future profiles only, and cannot be
mistaken for the requirement of record.
