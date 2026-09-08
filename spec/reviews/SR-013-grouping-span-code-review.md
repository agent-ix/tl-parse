---
id: SR-013
title: "Code review — grouping span provenance"
type: SpecReview
analysis: code-review
scope: "src/parser.rs, tests/parser.rs, spec/requirements/FR-002-parser-graph.md, spec/test-matrix.md"
review_set: subset
---

# Code review — grouping span provenance

## Summary

This review examined the parser's parenthesis path and its diagnostic-span
contract. The change removes the mutation that replaced a grouped child node's
lexical span with its enclosing delimiters, and TC-029 observes the public
parsed document for the concrete `(p1)&p2` case. No defect was found in the
changed Rust surface.

## Verdict

**CONDITIONAL** — the changed parser path, test, and specification are sound;
the repository's pre-existing local Python assurance tooling remains a
separate shared-assurance migration concern outside this change.

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
| FND-1301 | low | No defect found in the changed parser surface. Parenthesized parsing now returns the existing child node unchanged, and TC-029 fails if that child span is overwritten by the delimiter extent. | src/parser.rs:250, tests/parser.rs:156, FR-002-AC-4 |
| FND-1302 | medium | The pre-existing Python assurance-chain surface remains outside this patch and must be addressed by the shared-assurance migration, not reimplemented locally. | scripts/assurance_chain.py:1, scripts/check_shared_pins.py:1, CLAUDE.md:27 |
