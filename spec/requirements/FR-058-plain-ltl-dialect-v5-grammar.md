---
id: FR-058
title: "Parse bracket-free LTL and LTLf operators in a distinct v5 dialect"
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-044
    type: implements
  - target: ix://agent-ix/tl-parse/ADR-041
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-332
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-333
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-289
    type: depends_on
---

# FR-058: Parse bracket-free LTL and LTLf operators in a distinct v5 dialect

## Description

When `tl-parse.clean-ascii/v5` is selected with a caller-supplied `LtlProfile`,
tl-parse shall parse the bracket-free spellings of ADR-041 into a validated
tl-syntax graph of the edition and profile that the selection names, or return
typed diagnostics and no graph.

## Inputs

- UTF-8 source, `LtlProfile`, and the owner tl-syntax limits with an optional
  lower caller limit.

## Outputs

- Under `LtlfFiniteTrace`: a `tl-syntax.formula-ltlf/v1` graph under
  `mltl.ltlf-finite-trace/v1`.
- Under `InfiniteTrace`: a `tl-syntax.formula-unbounded/v1` graph under
  `mltl.infinite-trace/v1` in which `F p` is `F[0,) p`, and likewise `G`, `U`,
  `R`, and `X p` is `F[1,1] p`.
- A versioned report `tl-parse.ltl-parse-report/v1` recording dialect, profile
  and effective limits.

## Behavior

Keywords `F`, `G`, `U`, `R`, `X`, `Xs`, `Xw` are case-sensitive. Lexical
grammar: at each position the lexer takes the longest of the token spellings
`F G U R X Xs Xw true false`, `p` followed by one or more decimal digits (the
digits are consumed greedily and must be canonical, FR-001), the operators and
the parentheses; whitespace separates tokens but is not required between them.
So `Gp0` is `G p0`, `Xsp0` is `Xs p0`, `Ftrue` is `F true`, `Gtrue` is `G true`,
and `Xx` and `Uq` are an operator followed by the unknown character `x` or `q`,
which is refused at that character's span. There is no word-boundary rule. Any interval bracket
after a temporal keyword is refused with a diagnostic that names dialects v1
to v4 as the interval-bearing spellings. `W`, `M`, `O`, `H`, `Y`, `S`, `T` are
refused as unsupported in v5. Atoms, constants, Boolean operators, grouping and
precedence are those of v1.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-058-AC-1 | Every formula of the exhaustive depth-one population (120 LTLf, 116 infinite) parses under its profile to the expected graph, edition and profile identity. | Test (TC-112) |
| FR-058-AC-2 | Each interval spelling and each of `W M O H Y S T` is refused at a stable span with its own diagnostic code under both profiles. | Test (TC-113) |

## Dependencies

FR-009 owns dialect organization; tl-syntax FR-332/FR-333 own the graphs.
