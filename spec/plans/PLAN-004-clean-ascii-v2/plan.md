---
id: PLAN-004
title: Clean-ascii v2 derived future operators
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-parse/FR-008
    type: references
---

# PLAN-004: Clean-ascii v2 derived future operators

Implement FR-008 for issue #31 as an explicitly selected input dialect that
lowers `W`/`M` through tl-syntax and never adds a derived node.

1. Pin a tl-syntax revision that carries the FR-008 lowering API (tl-syntax#40).
   Record the source-inspected compiled-pin delta, and move the pin to the
   merged `main` commit before the PR leaves draft.
2. Specify `DIALECT-002`, FR-008, and TC-039 through TC-045 before the code.
3. Share the lexer and parser between dialects. v2 adds the `W`/`M` and
   unsupported-operator tokens, lowers each derived expression through
   `FutureLoweringRequest`, and returns the strict
   `tl-parse.derived-parse-report/v1`.
4. Leave v1 parsing, reports, formatting, and the CLI unchanged. Prove that
   with byte-level tests and the existing golden tests.
5. Add the `clean_ascii_v2` fuzz target and checksummed seeds. Run the fuzz
   build and smoke gates, then the exact-head local `make ci`. Hosted CI stays
   manual-only.
