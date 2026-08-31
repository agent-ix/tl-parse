---
id: StR-002
title: Provide canonical and bounded parsing tools
type: StR
---

# StR-002: Provide canonical and bounded parsing tools

## Stakeholder Need

Integrators need deterministic formatting and diagnostics that remain safe on
malformed or adversarial source and can be exercised through both a library
and a thin command-line interface.

## Rationale

Round trips are useful only when canonical output is stable and failure paths
cannot panic, recurse without limit, or report partial work as success.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-002-VC-1 | Canonical format is idempotent and valid generated formulas parse, format, and parse without structural drift. | Test |
| StR-002-VC-2 | Malformed corpora, explicit budget exhaustion, fuzz seeds, and CLI outcomes are retained and reproducible. | Test |

## Stakeholders

Library integrators, CLI users, fuzzing operators, and assurance reviewers.

## Context and Assumptions

Resource limits are logical counters rather than wall-clock performance claims.

## Traceability

This need is realized by FR-003, FR-004, and FR-005 and verified by TM-001.
