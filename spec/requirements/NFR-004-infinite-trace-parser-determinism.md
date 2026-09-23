---
id: NFR-004
title: Keep infinite-trace parsing bounded and deterministic
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-parse/FR-015
    type: constrains
  - target: ix://agent-ix/tl-parse/FR-016
    type: constrains
  - target: ix://agent-ix/tl-parse/FR-017
    type: constrains
---

# NFR-004: Keep infinite-trace parsing bounded and deterministic

## Statement

When a v4 parse or format request is processed, tl-parse shall apply the
existing source, token, node, depth, diagnostic, work and output ceilings
before excess retention and return deterministic reports at equal inputs.

## Scope

Applies to v4 source, fairness premises, unbounded intervals and formatter
output. Owner tl-syntax ceilings cannot be raised by a local caller limit.

## Rationale

An unbounded interval denotes time, not an unbounded parser workload. A
fairness list must not bypass the limits already enforced for formulas.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Same-input report and canonical-text byte differences | 0 | 0 | Test (TC-066) |
| One-over-limit requests yielding a usable graph or output | 0 | 0 | Test (TC-066) |

## Verification

Run each exact limit and one-over case, including a fairness list that spends
the whole node and work budget. Repeat at fixed input and configuration and
compare full report bytes. Exercise arbitrary bounded UTF-8 under fuzzing.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-004-AC-1 | Equal v4 inputs and limits yield byte-identical reports and text; each exact limit succeeds and one-over refuses before a partial graph or excess output. | Test (TC-066) |

## Dependencies

FR-003 and NFR-001 supply existing resource accounting; tl-syntax owns the
immutable graph ceiling.
