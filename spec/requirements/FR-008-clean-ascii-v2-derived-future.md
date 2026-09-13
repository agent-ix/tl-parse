---
id: FR-008
title: Parse derived future operators in clean-ascii v2
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-001
    type: implements
  - target: ix://agent-ix/tl-parse/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-008
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-009
    type: depends_on
---

# FR-008: Parse derived future operators in clean-ascii v2

## Description

When a caller explicitly selects `tl-parse.clean-ascii/v2`, the crate shall
parse the bounded derived operators `W[a,b]` and `M[a,b]`. It shall lower each
through the pinned tl-syntax `FutureLoweringRequest` into primitive nodes and
report the exact operator and expression spans. If the source holds a
malformed, unknown, or unsupported form, then the crate shall return a typed
diagnostic and no document. The crate shall not add a derived node, a second
AST, an evaluator branch, a user-authored Quire language, or a FRETish parser.

## Behavior

- **Explicit selection:** `parse_clean_ascii_v2` is the only entry point that
  accepts v2 text.
  - The existing `parse`, `parse_with_context`, formatter, CLI, and
    `tl-parse.clean-ascii/v1` identity are unchanged.
  - v1 parsing continues to reject `W` and `M` as unknown identifiers.
- **Grammar:** `DIALECT-002-clean-ascii-v2.md` is normative.
  - `W` and `M` bind at the `U`/`R` power and associate left with them.
  - Intervals are mandatory and checked exactly as in v1.
- **Lowering:** each derived expression lowers once, after both operands are
  parsed.
  - The request carries the selected semantic profile, the parser's current
    node table as the borrowed formula, both operand identities, the interval,
    and both spans.
  - The three returned nodes are appended unchanged.
  - A lowering refusal is a `validation_failure` diagnostic naming the refusal
    code. The parser always submits admissible fields, so reaching this path
    indicates a defect.
- **Resources:**
  - A derived expression charges three nodes against the effective node limit,
    checked before any node is appended.
  - Validating the borrowed formula charges one parser work unit per existing
    node, so repeated lowerings stay within the effective work limit.
- **Unsupported operators:** in v2, the identifiers `X`, `Y`, `O`, `H`, `S`,
  and `T` are refused with `unsupported_operator` and no document. Weak-next
  and mixed-time forms have no v2 spelling, so they are refused through these
  codes or as unknown identifiers.
  - v1 never emits `unsupported_operator`.
  - Dense, timestamped, unit-bearing, open, or unbounded interval text is
    refused through the existing interval diagnostics.
- **Report:** a `tl-parse.derived-parse-report/v1` value carries:
  - the dialect and operator-profile identities, the tl-syntax revision, the
    semantic profile, limits, and stats;
  - the document, present only on a diagnostic-free parse;
  - one lowering record per derived expression, in lowering order: kind,
    operator span, expression span, left, right, first generated node, and
    root;
  - the diagnostics.

  Lowering records are present only with the document. The wire form rejects
  unknown fields. The v1 `ParseReport` wire form is unchanged.
- **Formatting:** canonical output of a v2 document is the existing
  primitive-only rendering. It contains no `W` or `M`.
  - Within the effective parse limits, v1 accepts it and parsing it under v2
    returns identical canonical text.
  - Lowering shares the left operand, so the text repeats it and left-nested
    chains double it per level. A reparse beyond the limits is refused only
    through a resource diagnostic.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-008-AC-1 | The v2 identity and normative record are explicit and digest-bound. v2 is selected only through its own entry point. v1 parsing, reports, and diagnostics are byte-unchanged and still reject derived spellings. | Test (TC-039) |
| FR-008-AC-2 | Every accepted v2 source produces the same graph as direct construction with tl-syntax lowering. That includes W/M precedence and left associativity mixed with U/R and Boolean operators, parenthesized operands, and exact operator, expression, and node spans. | Test (TC-040, TC-041) |
| FR-008-AC-3 | Each of these produces its stable diagnostic code at the offending span, with no document and no lowering record: interval-less W/M, lowercase or long aliases, `X` and past spellings, non-canonical, overflowing, inverted, open, or unit-bearing bounds, the three-node charge at the node limit, and the lowering work charge at the work limit. | Test (TC-042) |
| FR-008-AC-4 | Canonical text of a lowered document contains only primitive operators. Within the effective parse limits v1 accepts it and v2 re-parsing reaches the same canonical text; beyond them the reparse is refused only through a resource diagnostic. The lowered document is byte-identical on the formula-v1 wire to the directly constructed one. | Test (TC-043) |
| FR-008-AC-5 | A checked-in `clean_ascii_v2` fuzz target builds and consumes its seeds. For arbitrary input it returns either a document with lowering records or bounded diagnostics, never panics, and never reports the v1 dialect identity. | Test (TC-044) |
| FR-008-AC-6 | The derived report is deterministic across repeated parses and serializes under its own strict versioned identity. Unknown fields and a v1 schema identity are rejected, and mutations of the identity, record, or span fields change the report. | Test (TC-045) |

## Dependencies

Depends on FR-002 graph construction and FR-004 canonical formatting. Also
depends on the tl-syntax FR-008 lowering API and the FR-009 dialect rules at a
pinned tl-syntax revision that contains `src/future.rs` (tl-syntax#40).
