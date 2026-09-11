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
- Signal inference, FRETish input, local contract-IR types, or a local evidence
  execution framework.

## System Overview

The lexer recognizes a closed ASCII token set. A deterministic precedence
parser appends nodes directly to a topologically ordered tl-syntax document.
Any diagnostic prevents a successful document. The formatter consumes only a
validated tl-syntax formula and emits fully unambiguous canonical text within
declared resource limits.

## Requirements Architecture

FR-001 owns the dialect and lexer, FR-002 parsing and graph construction,
FR-003 diagnostics and fail-closed limits, FR-004 canonical formatting and
round trips, FR-005 corpora, fuzzing, CLI, and evidence interchange, FR-006
the shared-assurance intake boundary, and FR-007 additive shared-catalog/context
binding reports for parsed formulas without changing the dialect or parser
graph. NFR-001 constrains determinism/resources, NFR-002 provenance and
authority, and NFR-003 explicit fail-closed qualification controls.

### Responsibility and dependency allocation

| Component | This specification guarantees | Assumption or external responsibility |
| --- | --- | --- |
| tl-parse | Bounded dialect parsing, graph construction, diagnostics, canonical formatting, corpus/fuzz behavior, context-bound reports, and producer-owned result bytes | It does not evaluate, rewrite, monitor, or infer application signal meaning. |
| tl-syntax | — | The exact pinned revision supplies validated graph, interval, span, proposition, and semantic-profile contracts. tl-parse does not redefine them. |
| Quire | tl-parse supplies its specification and requirement-tagged source tree as inputs | Quire reports static specification/coverage facts and does not execute a parser producer or grant release authority. |
| Quoin | tl-parse supplies producer-written structured results | Quoin seals, retains, audits, and reports the bytes it receives; it neither creates producer results nor decides release sufficiency. |
| Engineering Assurance | tl-parse reports the observed shared-tool versions to the released compatibility contract | The released compatibility matrix owns classification; this repository does not restate its rules. |
| ix-flow / release owner | tl-parse preserves an absent decision as an incomplete receipt | Only the human release owner may record the release decision; no automated pass implies it. |

NFR-003 records the remaining assumption that Make's own execution controls are
not qualified by a shared control. That known risk remains `tl-parse#11` and a
human release-decision input; the producer-byte boundary covers only results
that were actually produced and does not turn bypassed non-producer gates into
passes.

## References

- [tl-parse epic](https://github.com/agent-ix/tl-parse/issues/4).
- [tl-syntax](https://github.com/agent-ix/tl-syntax).
- [PGM-01](https://github.com/agent-ix/quire-contract-ir/blob/main/spec/program/PGM-01-governance.md).
