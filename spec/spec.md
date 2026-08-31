---
id: MRS-001
title: tl-parse v0.1 master requirements
type: MasterRequirements
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
  - target: ix://agent-ix/tl-syntax/MRS-001
    type: depends_on
---

# Master Requirements Specification

## Purpose

This specification defines the clean-room textual boundary for bounded MLTL.
tl-parse converts independently specified ASCII source into the exact pinned
tl-syntax graph model, emits stable source diagnostics, and formats validated
graphs into one canonical representation.

PGM-01 governs provenance, compatibility, evidence, human authority, and
qualification boundaries. tl-syntax owns formula nodes, intervals, source
spans, proposition identities, and semantic-profile identities; this crate
does not introduce a second AST or temporal semantics.

## Scope

### In Scope

- A versioned ASCII dialect for Boolean and bounded MLTL operators.
- Deterministic precedence parsing into validated tl-syntax documents.
- Versioned, byte-located diagnostics with expected tokens and recovery action.
- Explicit source, token, node, nesting, diagnostic, work, and output limits.
- Canonical formatting, round-trip properties, malformed fixtures, fuzz seeds,
  and thin validation/formatting CLI surfaces.

### Out of Scope

- A second formula AST, rewrite rules, temporal evaluation, or monitoring.
- Copying grammar text from third-party implementations or publications.
- Unicode proposition names, application name resolution, or unbounded input.
- Automatic qualification, certification, publication, or release approval.

## System Overview

The lexer recognizes a closed ASCII token set. A deterministic precedence
parser appends nodes directly to a topologically ordered tl-syntax document.
Any diagnostic prevents a successful document. The formatter consumes only a
validated tl-syntax formula and emits fully unambiguous canonical text within
declared resource limits.

## Requirements Architecture

FR-001 owns the dialect and lexer, FR-002 parsing and graph construction,
FR-003 diagnostics and fail-closed limits, FR-004 canonical formatting and
round trips, and FR-005 corpora, fuzzing, CLI, and evidence interchange.
NFR-001 constrains determinism/resources and NFR-002 provenance and authority.

## References

- [tl-parse epic](https://github.com/agent-ix/tl-parse/issues/4).
- [tl-syntax](https://github.com/agent-ix/tl-syntax).
- [PGM-01](https://github.com/agent-ix/quire-contract-ir/blob/main/spec/program/PGM-01-governance.md).
