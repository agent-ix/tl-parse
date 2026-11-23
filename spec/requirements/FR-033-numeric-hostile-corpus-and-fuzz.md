---
id: FR-033
title: Extend the hostile-input corpus and fuzz targets for the numeric dialect
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: implements
  - target: ix://agent-ix/tl-parse/FR-005
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-029
    type: depends_on
---

# FR-033: Extend the hostile-input corpus and fuzz targets for the numeric dialect

## Description

Where v5 exists, tl-parse shall carry a hostile-input corpus `corpus/numeric-v5/`
and a cargo-fuzz target `numeric_v5_roundtrip` with checked seeds, and the smoke
runner shall execute them.

## Behavior

- Corpus manifest `tl-parse.numeric-corpus/v1` lists, per case: id, path, catalog
  reference, expected canonical text or expected code with span, and reason. The
  required classes (a missing class fails the corpus check) are: valid per
  operator and node kind; each decimal scale 0, 1, 2, 9, 18; each diagnostic
  code; literals `-0`, `007`, `1.`, `.5`, `1e5`, `0x10`, 19-digit overflow,
  scale 19, 4,096-digit literal; U+2212 minus, fullwidth digits, NUL, an invalid
  UTF-8 file (byte-level CLI case); identifier of 255 and 256 bytes; reserved
  word and `p17` as signal names; chained comparison; `=` for `==`;
  division and modulo; nesting of 255, 256 and 257 parentheses and arithmetic
  nodes; token flood at the token limit plus one; expansion `k = 1,024` and
  `1,025`; at-least expansion crossing 100,000 nodes; empty and duplicate sets;
  duplicate and half contracts.
- Generated fuzz output is not corpus; a minimized disagreement is promoted only
  with provenance and an independently confirmed expectation.
- Fuzz target `numeric_v5_roundtrip` splits input into a catalog selector byte and
  source bytes, parses, formats on success, and asserts the fixed point and
  absence of panic, with resource caps equal to the hard maxima. Checked seeds
  are independently authored under the crate's licence.
- `make fuzz-smoke`, `make conformance` and the Miri suite list include the new
  corpus and seeds; the runner fails on a crash artifact or nonzero exit.
  Longer libFuzzer campaigns are supplementary.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-033-AC-1 | The corpus manifest covers every required class, each case's recorded expectation matches the parser, and removing a class fails the check. | integration-testing (TC-1555) |
| FR-033-AC-2 | The fuzz target builds, its seeds replay under the smoke runner within the 64-execution/300-second bounds without a crash artifact. | fuzzing (TC-1556) |
| FR-033-AC-3 | The hostile corpus runs under the resource caps with peak work bounded by the hard maxima. | unit-testing (TC-1557) |

## Dependencies

FR-005 owns the existing corpus and CLI pattern.
