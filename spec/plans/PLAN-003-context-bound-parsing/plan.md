---
id: PLAN-003
title: Context-bound parsing
type: Plan
status: complete
relationships:
  - target: ix://agent-ix/tl-parse/FR-007
    type: references
---

# PLAN-003: Context-bound parsing

Implement FR-007 as an additive parser result that consumes only the shared
`tl-syntax` catalog and optional requirement-context documents.

1. Pin the merged `tl-syntax` context revision and record it in parser output.
2. Parse with the existing API, retain the validated formula/catalog/context
   values, and resolve each free proposition once in stable parser-node order.
3. Return typed non-success for invalid catalogs, parse failure, absent parser
   spans, or unresolved proposition bindings.
4. Bind catalog and domain-separated request identities with deterministic
   SHA-256 records; do not add a local schema or assurance runtime dependency.
5. Close TC-029 through TC-034, including strict v2 wire decoding with an
   explicit nullable context field, and run normal local Rust gates.
