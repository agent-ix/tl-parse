---
id: FR-003
title: Emit stable diagnostics and fail closed at resource limits
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-001
    type: implements
  - target: ix://agent-ix/tl-parse/StR-002
    type: implements
---

# FR-003: Emit stable diagnostics and fail closed at resource limits

## Description

Every rejected input shall produce deterministic versioned diagnostics and no
successful document. Logical limits shall stop work before unbounded growth.

## Behavior

- Each diagnostic carries a stable code, severity, byte span, found token,
  ordered expected-token set, recovery action, and human-readable message.
- Parse reports identify the dialect, diagnostic schema, tl-syntax revision,
  semantic profile, configured limits, and observed logical counts.
- Source bytes, tokens, nodes, nesting, diagnostics, and parser work are
  independently bounded with stable limit-specific codes.
- Reaching the diagnostic cap records truncation in report statistics; it does
  not fabricate a successful parse.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-003-AC-1 | Golden malformed inputs produce the exact expected codes, spans, expected tokens, found token, recovery action, and stable JSON bytes. | Test (TC-009, TC-010) |
| FR-003-AC-2 | Every declared resource limit is exercised at its boundary and produces no document, panic, or undeclared growth. | Test (TC-011, TC-012) |
| FR-003-AC-3 | Identical source, profile, and limits produce byte-identical reports and diagnostics. | Test (TC-013) |

## Dependencies

Depends on FR-001 and FR-002.
