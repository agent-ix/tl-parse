---
id: NFR-001
title: Make parsing deterministic and resource bounded
type: NFR
quality_attribute: reliability
---

# NFR-001: Make parsing deterministic and resource bounded

## Statement

tl-parse shall produce deterministic lexical order, nodes, diagnostics,
formatting, JSON, and exit class for identical input bytes, semantic profile,
limits, dialect, and dependency revision, without wall-clock or randomized
behavior.

## Scope

The requirement covers lexer/parser reports, tl-syntax graph construction,
diagnostic serialization, canonical formatting, corpus execution, and CLI exit
classes.

## Rationale

Reproducible evidence and hostile-input safety require explicit logical
counters rather than timing assumptions or allocator-dependent behavior.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Repeat-output mismatch count | 0 | 0 | deterministic test |
| Unbounded public logical resources | 0 | 0 | inspection and boundary tests |

## Verification

Unit, integration, property, corpus, and CLI tests repeat requests and exercise
each limit independently. Retained reports record configured limits and
observed token, node, diagnostic, work, and output counts; timing is not a
conformance criterion.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Repeated parse, diagnostic, canonical-format, and CLI operations are byte-identical. | Test (TC-013, TC-015, TC-021) |
| NFR-001-AC-2 | Source, token, node, depth, diagnostic, parser-work, formatter-work, and output-byte limits are explicit, checked, and independently tested. | Test (TC-011, TC-012, TC-017) |
