---
id: StR-001
title: Provide an independently reviewable MLTL text boundary
type: StR
---

# StR-001: Provide an independently reviewable MLTL text boundary

## Stakeholder Need

Temporal-tool developers need one unambiguous textual MLTL dialect whose
accepted inputs, rejected inputs, source locations, and dependency boundary can
be reviewed without trusting an undocumented or ambiguously licensed grammar.

## Rationale

Silent reinterpretation of precedence, intervals, proposition identities, or
profiles can change a formula before later semantic checks begin.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-001-VC-1 | The dialect revision, grammar, precedence, associativity, and clean-room provenance are explicit and stable. | Inspection (TC-020) |
| StR-001-VC-2 | Every accepted source produces a structurally valid graph from the exact pinned tl-syntax revision, while rejected source produces no graph. | Test (TC-007, TC-008) |

## Stakeholders

Temporal-crate developers, assurance reviewers, CLI users, and the human source-release owner.

## Context and Assumptions

The v0.1 textual proposition form is `p` followed by a canonical decimal u32;
application-specific name resolution is outside this crate.

## Traceability

This need is realized by FR-001, FR-002, and FR-003 and verified by TM-001.
