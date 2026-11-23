---
id: FR-030
title: Report numeric diagnostics with stable codes and loci
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: implements
  - target: ix://agent-ix/tl-parse/FR-029
    type: depends_on
  - target: ix://agent-ix/tl-parse/FR-003
    type: depends_on
---

# FR-030: Report numeric diagnostics with stable codes and loci

## Description

When v5 refuses input, tl-parse shall return diagnostics from the closed
`tl-parse.numeric-diagnostics/v1` vocabulary with a half-open byte span, the
effective limits and the decisive refusal, even if the diagnostic count is
capped.

## Behavior

The closed code set is: `source_limit`, `token_limit`, `node_limit`,
`depth_limit`, `work_limit`, `unexpected_character`, `unexpected_token`,
`missing_token`, `trailing_input`, `non_canonical_number`, `integer_overflow`,
`unsupported_operator`, `profile_mismatch`, `unknown_signal`,
`non_numeric_signal`, `scale_mismatch`, `scale_overflow`, `rescale_narrowing`,
`range_overflow`, `reserved_identifier`, `non_associative_comparison`,
`unparenthesized_comparison`, `undefined_set`, `duplicate_set`,
`slot_outside_template`, `aggregation_too_large`, `duplicate_contract`,
`missing_contract_clause`, `proposition_overflow`. Resource limit codes take
priority over syntax codes in the same way as the v4 priority rule; at most 64
diagnostics are returned, and the decisive refusal is retained separately.
Messages are English prose for humans; only the code and span are contractual.
A lexical error never produces a cascade: recovery resumes at the next
delimiter in a bounded number of steps.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-030-AC-1 | Each code is produced by a minimal source at its declared span, and the closed set is exactly the list above. | unit-testing (TC-1546) |
| FR-030-AC-2 | When more than 64 diagnostics occur, the first 64 are returned in source order and the decisive refusal is still reported. | unit-testing (TC-1547) |
| FR-030-AC-3 | The report reader rejects unknown fields, unknown codes, trailing data and over-limit documents. | unit-testing (TC-1548) |

## Dependencies

FR-003 (existing diagnostics and limits).
