---
id: FR-015
title: Parse a distinct infinite-trace text dialect
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-020
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-021
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-289
    type: depends_on
---

# FR-015: Parse a distinct infinite-trace text dialect

## Description

When `tl-parse.clean-ascii/v4` is selected with
`mltl.infinite-trace/v1`, tl-parse shall parse the bounded and unbounded
future/past syntax into a validated `tl-syntax.formula-unbounded/v1` graph and
an optional fairness-premises document in that same graph.

## Inputs

- UTF-8 source, explicit v4 dialect and exact TL profile identity.
- The owner tl-syntax syntax limits and a caller limit that may lower them.

## Outputs

- A validated owner graph, optional fairness document and versioned v4 parse
  report; otherwise typed diagnostics and no usable graph.

## Behavior

V4 accepts F/G/U/R/O/H/S/T with `[a,b]` or `[a,)`, Y as the one-position
past operator, W/M through the owner lowering, and the existing Boolean and
atomic forms. The explicit interval spelling `[a,)` is TL-native; QSL's
`[a,*]` is never silently accepted as TL text. The v4 envelope is
`fair { premise; ... } : formula` when premises are present, and `formula`
when none are present. Each premise is a full v4 formula joined into the one
canonical graph and referenced by its root identity. An empty `fair {}` is
accepted and has the same semantics as no premises, but formatting omits it.

V4 refuses an absent or different TL profile before graph construction. V1,
v2 and v3 reject the v4 envelope and every unbounded interval; their existing
documents, reports and canonical bytes do not change. V4 preserves the
explicit `event_position` clock from FR-020 and refuses other clocks. No
parser path evaluates fairness or liveness. The parse report records exact
compiled owner revision and effective limits.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-015-AC-1 | All eight bounded/unbounded future and past primitives, Y, and lowered W/M parse into the owner graph only under exact v4 and TL infinite-profile selection. | Test (TC-058) |
| FR-015-AC-2 | A fairness envelope yields ordered, unique validated roots in the same graph; absent and empty envelopes mean no assumptions and never decide fairness. | Test (TC-059) |
| FR-015-AC-3 | V1/v2/v3 bytes and admissions remain unchanged; each refuses v4-only syntax, while v4 refuses finite/QSL profile or non-event-position clocks. | Test (TC-060) |

## Dependencies

FR-009 owns dialect separation. tl-syntax FR-020/021/289 own identities,
fairness and the admitted graph; this parser builds no mirror AST.
