---
id: FR-005
title: Retain hostile-input, fuzz, CLI, and conformance evidence
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-002
    type: implements
---

# FR-005: Retain hostile-input, fuzz, CLI, and conformance evidence

## Description

The repository shall retain versioned malformed and resource fixtures, a fuzz
target seeded by those fixtures, and thin CLI surfaces.

## Inputs

- One of the closed fuzz-target identities `parser` or `clean_ascii_v2`.
- That target's repository-owned seed files.
- The fixed smoke bounds of 64 executions and 300 seconds and LeakSanitizer
  availability.

## Outputs

- A zero exit status for a passing smoke run and a non-zero status otherwise.

## Behavior

- Corpus files and manifest identify expected diagnostic codes or success
  results.
- The complete local gate compiles each checked-in fuzz target and executes a
  bounded libFuzzer smoke run over every seed. The target exercises parsing,
  diagnostic serialization, and successful canonical round trips under small
  fixed budgets.
- The smoke runner shall refuse an unknown target, ambient sanitizer override,
  or unavailable LeakSanitizer without reporting a passing run.
- The smoke runner shall return a non-zero process status for every non-passing
  outcome.
- The smoke runner shall report `pass` only when the fuzz process exits
  successfully and produces no crash artifact; a launched non-zero process is
  `fail`, as is a successful process that nevertheless leaves a crash
  artifact.
- If the fuzz process exceeds 300 seconds, then the smoke runner shall
  terminate it and report `unavailable`.
- `tl-parse validate` and `tl-parse format` accept a profile and file/stdin,
  use the library report, and have stable success, invalid-input, and usage
  exit classes.

- The CLI SHALL delegate generic format-report JSON encoding and result-line writing to ix-cli-kit. Domain report serialization, bounded input, profile selection and diagnostic rendering remain owned by tl-parse.
- If stdout closes with BrokenPipe, then the CLI SHALL retain the command outcome; other write failures retain usage-or-I/O exit 2 with the existing diagnostic context. Result lines retain exactly one appended newline.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-005-AC-1 | Every malformed/resource corpus fixture produces its declared bounded outcome. | Test (TC-018) |
| FR-005-AC-2 | Each checked-in fuzz target compiles, a 64-execution/300-second libFuzzer smoke run consumes every seed, every unavailable or failed run is non-zero, and successful seeds round-trip under declared limits. | Test (TC-019, TC-047) |
| FR-005-AC-3 | CLI validation/formatting outputs and exit classes match the library for valid, invalid, profile, stdin, source-limit, and usage cases; an oversized seekable file reports its metadata byte count, while a non-closing stream is read only through the first byte beyond the limit, without parsing fabricated text. | Test (TC-020, TC-021) |
| FR-005-AC-4 | Shared compact format-report encoding and result-line writing retain JSON fields, one appended newline and domain exit classes; an injected BrokenPipe succeeds and another write error retains its context. | Test (TC-020, TC-021) |

## Dependencies

Depends on FR-003 and FR-004.
