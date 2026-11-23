---
id: FR-061
title: "Refuse hostile v5 input with stable diagnostics and cover it with a corpus and a fuzz target"
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-044
    type: implements
  - target: ix://agent-ix/tl-parse/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-005
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-058
    type: depends_on
---

# FR-061: Refuse hostile v5 input with stable diagnostics and cover it with a corpus and a fuzz target

## Description

If v5 source is empty, malformed, over-limit or outside the dialect, then
tl-parse shall return a stable diagnostic with a half-open byte span and no
graph, and the repository shall retain a hostile corpus and a fuzz target for
v5.

## Behavior

Hostile corpus `tl-parse-corpus/v1` family `v5` has 29 cases, each with an
expected outcome per profile: empty (1), whitespace-only (1), leading-zero atom
(1), `u32` overflow atom (1), non-ASCII (1), interval spellings (4), bare `X`
under LTLf (1), `Xs` under infinite (1), `Xw` under infinite (1), unbalanced
parentheses (2), depth limit exact and one over (2), token limit exact and one
over (2), `W M O H Y S T` (7), unknown prefix `Xx` (1), NUL byte (1), source
limit exact and one over (2). Of the 29, 3 are exact-limit cases that parse and
26 are refused. The fuzz target `clean_ascii_v5` takes arbitrary bytes and a
profile bit, must not panic, and every accepted input must satisfy the FR-060
round trip; seeds are the 29 corpus cases.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-061-AC-1 | The 29 hostile cases produce exactly their manifest outcomes (3 parse, 26 refused) under each profile, with stable codes and spans. | Test (TC-120, TC-122) |
| FR-061-AC-2 | The `clean_ascii_v5` fuzz target builds, replays all 29 seeds without a panic, and asserts the round trip on accepted inputs. | Test (TC-121) |

## Dependencies

FR-003 diagnostics; FR-005 corpus and fuzz evidence pattern.
