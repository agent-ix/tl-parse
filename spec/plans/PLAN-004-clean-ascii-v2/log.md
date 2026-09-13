---
type: log
title: "PLAN-004 - Update log"
description: "Chronological changes to the clean-ascii v2 derived future-operator plan."
---

# PLAN-004 - Update log

## History

- **2026-09-12** - Specified DIALECT-002, FR-008, and TC-039 through TC-045
  for issue #31 ahead of implementation. The matrix adopts the single `Status`
  column (spec-artifacts-process#87). Development compiles against tl-syntax
  `e3651cd` on the unmerged issue/40 branch. The pin moves to merged `main`
  before the PR leaves draft.
- **2026-09-12** - Implemented the v2 lexer and parser lowering, the strict
  `tl-parse.derived-parse-report/v1` report, `tests/clean_ascii_v2.rs`, and the
  `clean_ascii_v2` fuzz target with four checked seeds. `make check-corpus`,
  `fuzz-build`, and `fuzz-smoke` now cover both fuzz targets.
- **2026-09-12** - Applied the PR #32 code review and gap analysis:
  - Canonical text of left-nested chains grows exponentially. FR-008 AC-4 and
    DIALECT-002 now bound v1 acceptance by the effective limits, and the fuzz
    target accepts a resource refusal on reparse. A fifth `growth` seed pins it.
  - The lowering work refusal sits at the expression span.
  - Unsupported names split after `true`/`false` before `[`.
  - Decoding refuses a document with diagnostics and lowering records outside
    the document.
  - Added tests for the online profile, `->`/`<->` precedence, and dense,
    timestamped, and unbounded intervals.
