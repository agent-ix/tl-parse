---
id: StR-003
title: Bind parsed MLTL to declared typed signals and caller context
type: StR
relationships:
  - target: ix://agent-ix/tl-syntax/MRS-001
    type: depends_on
---

# StR-003: Bind parsed MLTL to declared typed signals and caller context

## Stakeholder Need

Contract-tool developers need an opt-in way to connect a parsed canonical MLTL
formula to one declared signal catalog and, when available, to the requirement
clause that supplied it. They must identify every free proposition
deterministically without changing the MLTL text dialect or asking this crate
to interpret FRETish or contract IR.

## Rationale

Numeric propositions alone cannot establish whether a formula is usable by a
consumer. Treating an absent binding, an undeclared name, or a non-Boolean
signal as Boolean would turn an incomplete contract into a plausible but false
formula. Collapsing an enclosing requirement span into a parser diagnostic span
would likewise lose two distinct source locations.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-003-VC-1 | A successful opt-in binding exposes a deterministic, duplicate-free proposition/signal set and preserves the declared signal identity and bounded domain. | Test (TC-029, TC-030) |
| StR-003-VC-2 | A missing or invalid proposition binding is refused at the exact MLTL proposition byte span; caller context, when supplied, remains a separate preserved value. | Test (TC-031, TC-032) |
| StR-003-VC-3 | Existing parse, format, CLI, corpus, fuzz, and round-trip paths remain source- and byte-compatible when the opt-in surface is unused. | Test (TC-033) |

## Stakeholders

Temporal-crate developers, contract-IR consumers, assurance reviewers, and
the human source-release owner.

## Context and Assumptions

`tl-syntax` owns `SignalCatalog`, typed bounded domains, direct proposition
bindings, `RequirementContext`, and their strict documents. This crate consumes
those types; it neither creates a second signal model nor assigns application
names or semantics to numeric propositions.

## Traceability

This need is realized by FR-007 and verified by TM-001.
