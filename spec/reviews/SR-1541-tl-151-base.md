---
id: SR-1541
title: "Base review of tl-parse plain LTL dialect v5 spec (TL-427)"
type: SpecReview
analysis: base
scope: "ADR-041, StR-044, FR-058..062, NFR-046, TM-043, PLAN-045"
review_set: all
---

## Summary

Counts recompute (120 LTLf and 116 infinite depth-one formulas; 29 hostile entries). Existing dialect identities `tl-parse.clean-ascii/v1..v4` exist in src/lib.rs:47-56; FR-001/003/004/005/009, NFR-002/004 exist. No sibling collisions (TL-150: FR-028-033, TC-154x). EARS: no warnings. Delta: validate unchanged, 28 new unbacked ACs, rows 116 to 146. ADR-041 item 6 and tl-mltl ADR-046 agree on importer placement. Cross-repo findings are in tl-mltl SR-1511..SR-1520.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15411 | high | FR-061-AC-1 asserts 3 parse and 26 refused under each profile, but bare `X`, `Xs`, `Xw` entries have profile-dependent outcomes, so per-profile counts differ; also TC-120 and TC-122 both trace FR-061-AC-1. | FR-061 |
| FND-15412 | medium | Lexing rule "maximal munch so Gp0 is G p0" does not define `Ftrue`, `Gtrue`, `Uq`; state the lexical grammar. | FR-058 |
| FND-15413 | low | New TC-121/TC-122 share numbers with existing tl-mltl TC-121/122 (different repos; harmless unless tags are resolved ecosystem-wide). | TM-043 |
