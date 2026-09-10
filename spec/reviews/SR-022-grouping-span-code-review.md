---
id: SR-022
title: "Code review — grouping span provenance"
type: SpecReview
analysis: code-review
scope: "src/parser.rs, tests/parser.rs, spec/requirements/FR-002-parser-graph.md, spec/test-matrix.md"
review_set: subset
---

# Code review — grouping span provenance

## Summary

This authorial review examined the parser's parenthesis path and its
diagnostic-span contract. The first change removed the mutation that replaced a
grouped child node's lexical span with its enclosing delimiters, but an
independent exact-head review established that doing so discarded the grouping
extent needed by enclosing operators. The remediated parser carries node
identity and syntactic extent separately, and TC-030 observes both facts through
the public parsed document.

## Verdict

**AUTHORIAL / CONDITIONAL** — FND-2201 is addressed in the current candidate,
but this record grants no independent review clearance. The repository's
pre-existing local Python assurance tooling remains a separate shared-assurance
migration concern outside this change.

## Assurance Context

AP-001 (`spec/assurance/AP-001.md`) applies through
`impact-silent-reinterpretation`: misleading source locations can misdirect a
consumer's diagnostic handling of an otherwise valid temporal graph. The
evaluated baseline is `958f018`, with the four changed paths named in this
review scope. FR-002, TM-001, AP-001, the parser's public integration test,
and the repository Rust conventions were available. No candidate-specific
Quoin record or architectural description was available, and the full shared
assurance lane was not run from this fresh worktree because its pinned
environment and producer outputs are absent. No AP-001 exception applies.
Existing Python assurance scripts were observed but neither changed nor
extended; their ownership remains the shared Quoin/Quire/Engineering-Assurance
migration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2201 | high | Resolved in the current candidate: preserving only the grouped child's lexical span made enclosing spans unbalanced. `Parsed` now carries the balanced grouping extent separately, and TC-030 checks grouped children plus unary and binary ancestors. | src/parser.rs, tests/parser.rs, FR-002-AC-4, TC-030, tl-parse#26 |
| FND-2202 | medium | The pre-existing Python assurance-chain surface remains outside this patch and must be addressed by the shared-assurance migration, not reimplemented locally. | scripts/assurance_chain.py:1, scripts/check_shared_pins.py:1, CLAUDE.md:27 |
