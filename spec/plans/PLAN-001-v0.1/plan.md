---
id: PLAN-001
title: tl-parse v0.1 implementation plan
type: Plan
relationships:
  - target: ix://agent-ix/tl-parse/FR-001
    type: references
  - target: ix://agent-ix/tl-parse/FR-005
    type: references
---

# tl-parse v0.1 implementation plan

## Dependency DAG

```text
PGM-01 + exact tl-syntax revision
  -> specification and assurance foundation
  -> versioned lexer and direct bounded parser
  -> stable diagnostics and fail-closed limits
  -> bounded canonical formatter and generated round trips
  -> corpus, fuzz seeds, CLI, verification, and review remediation
  -> exact-candidate retained evidence
  -> human v0.1 source-release decision
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | Specification and assurance foundation | Validated requirements, matrix, reviews, and assurance packet |
| Task-002 | Versioned lexer and direct parser | Complete dialect, exact graph/profile, precedence, and source-span tests |
| Task-003 | Stable diagnostics and limits | Golden reports and independent parser resource-boundary tests |
| Task-004 | Canonical formatter and round trips | Exact, idempotent, generated, shared/deep, and exhaustion tests |
| Task-005 | Corpus, fuzz, CLI, and verification | Checksummed populations, CLI tests, complete local gate, and resolved review findings |
| Task-006 | Exact-candidate evidence | Sealed PGM-01 validations and checksummed retained record |
| Task-007 | Human source-release decision | Maintainer review and explicit release decision |

## Exit Criteria

All matrix rows are backed, the complete local and finalized manual hosted
gates pass, no blocking gap remains, and the Assurance Argument stays open
until a human release owner records the source-release decision.
