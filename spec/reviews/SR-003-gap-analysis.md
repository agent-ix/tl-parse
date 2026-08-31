---
id: SR-003
title: Gap analysis of tl-parse v0.1 candidate
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, docs/**/*, src/**/*.rs, tests/**/*.rs, corpus/**/*, fuzz/**/*, CI and repository settings
review_set: all
---

# Gap analysis of tl-parse v0.1 candidate

## Summary

Strict coverage maps all 55 rows to executable or retained inspection
evidence. Tests cover the complete dialect, exact graph/profile mapping,
stable malformed reports, every declared logical limit, canonical formatting,
generated round trips, compilation and bounded execution of the actual fuzz
target, checksum-protected hostile and fuzz populations, CLI behavior, evidence
contracts, and the manual-only hosted-CI boundary. Retained exact-candidate
evidence passes all collection and post-seal gates. The remaining gate is the
independent human source-release decision. Universal parser correctness,
temporal semantics, application name resolution, and consuming-system
qualification remain outside the candidate claim.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-301 | medium | The structural TestMatrix contract requires `Coverage Status` while the installed traceability module expects `Status`; the two module assertions cannot share one header. Status classification is skipped and disclosed, while every underlying requirement, stakeholder, suite, and test row is independently backed. | TM-001, SUITE-003 |
| FND-302 | low | Generated and fuzz populations are bounded evidence, not exhaustive proof over every bounded UTF-8 string. | FR-004, FR-005, AA-001 |
| FND-303 | medium | The exact amended tl-syntax revision is not yet merged; the time-boxed source exception blocks release and requires a repin plus regenerated evidence. | AA-001, NFR-002 |
| FND-304 | medium | Independent human code review and the exact source-release decision remain pending; automation cannot approve, tag, publish, qualify, accredit, or certify the candidate. | AP-001, AA-001 |
| FND-305 | low | Defensive invalid-graph/validation branches are not caller-reachable through the safe public constructors; they are retained as fail-closed invariant boundaries rather than claimed as ordinary input behavior. | FR-003, FR-004 |
| FND-306 | medium | Unparenthesized `U`/`R` chains are left-associative in this dialect and may differ from other MLTL grammars; explicit parentheses are required for portable interchange. | FR-001, DIALECT-001 |
