---
id: SR-002
title: Code review of tl-parse v0.1 implementation
type: SpecReview
analysis: code-review
scope: src/**/*.rs, tests/**/*.rs, corpus/**/*, fuzz/**/*, schemas/**/*.json, scripts/**/*, Cargo.toml, Makefile, .github/workflows/ci.yml
review_set: all
---

# Code review of tl-parse v0.1 implementation

## Summary

The code review examined exact dependency/profile identities, lexical token
boundaries, UTF-8 byte spans, canonical-number handling, precedence and
associativity, direct topological graph construction, fail-closed recovery,
all parser/formatter limits, iterative graph traversal, diagnostic and JSON
stability, CLI exit classes, corpus/fuzz integrity, evidence classification,
and CI triggers. Review corrected canonical adjacency tokenization, exact
amended tl-syntax pins, limit-test construction, corpus integrity gates, and
evidence false-success handling. No unresolved code defect or blocking
requirement was found. Independent human review remains mandatory.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-201 | high | Resolved: all package, lockfile, API, dialect, assurance, and evidence identities now pin amended tl-syntax revision `740182f13b84858008d6f176f75136737d405c1b`. | FR-002, NFR-002 |
| FND-202 | medium | Resolved: atoms followed immediately by canonical `U[...]`/`R[...]` operators tokenize consistently with whitespace-free formatter output. | FR-001, FR-004 |
| FND-203 | medium | Resolved: source, token, node, depth, diagnostic, parse-work, output, and format-work failures suppress partial success and have direct tests. | FR-003, FR-004 |
| FND-204 | medium | Resolved: retained corpus and fuzz seeds are checksum-gated and successful seeds must parse-format-parse without drift. | FR-005 |
| FND-205 | medium | Resolved: evidence tooling distinguishes failure, skipped validation, provisional/final non-self-attestation, and sealed validation; immutable checksums are in the complete local gate. | FR-005, NFR-002 |
| FND-206 | medium | Resolved: hosted CI retains only `workflow_dispatch`; local iteration uses `make ci`, and a machine-checkable regression rejects push/PR triggers. | FR-005, MP-001 |
| FND-207 | high | The temporary exact git-source exception expires on 2026-09-07 or upstream merge and blocks source release until repin and evidence regeneration. | AA-001 |
