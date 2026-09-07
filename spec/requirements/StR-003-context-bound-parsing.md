---
id: StR-003
title: Bound parsed formula traceability
type: StR
---

# StR-003: Bound parsed formula traceability

## Stakeholder Need

Consumers need a parsed formula to be explicitly related to the authoritative
signal catalog and optional requirement context without turning the parser into
an application resolver or a second semantic model.

## Rationale

Binding must be inspectable at the textual boundary, but proposition resolution
cannot silently change the parsed formula or duplicate the shared semantic
types that downstream rewrite and evaluation consume.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-003-VC-1 | The native result preserves the shared documents and independently locates each free proposition in parser source bytes. | Test (TC-029, TC-031) |
| StR-003-VC-2 | Missing or changed bindings cannot silently succeed, while existing context-free parsing retains its compatibility boundary. | Test (TC-030, TC-032, TC-033) |

## Stakeholders

Parser integrators, contract authors, rewrite/evaluation integrators, and
assurance reviewers.

## Context and Assumptions

The caller owns the authoritative catalog and optional context; this parser
only reports their relationship to already parsed source.

## Traceability

This need is realized by FR-007 and verified by TM-001.
