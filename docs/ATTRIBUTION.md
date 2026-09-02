---
id: ATTRIBUTION-001
title: tl-parse clean-room source attribution
type: Standard
code: tl-parse-clean-room-attribution-v1
status: active
---

# Clean-room attribution boundary

The tl-parse dialect was independently authored from only the checked syntax
vocabulary and value model in `agent-ix/tl-syntax` revision
`740182f13b84858008d6f176f75136737d405c1b`, licensed MIT OR Apache-2.0. The
files consulted at that revision were `src/syntax.rs`, `src/document.rs`,
`LICENSE-MIT` and `LICENSE-APACHE`.

No third-party parser implementation, parser grammar, grammar production, or
grammar prose was consulted or copied. Conventional ASCII operator spellings
are expected convergence for this operator vocabulary and are not claimed as
novel. This declaration records the authorship boundary; automation cannot
prove a negative provenance claim and human review remains required.

## Compiled revision, which is a different fact

The authorship basis above is historical and does not move: those are the bytes
that were read when the dialect was authored. The revision this crate *compiles
against* is separate, and it has advanced.

`tl-syntax` merged its own shared-assurance migration as
`953ee825e5060335b4c79682f5f41a78c5a1bfae`, and `Cargo.toml`, `Cargo.lock`,
`fuzz/Cargo.lock` and [`TL_SYNTAX_REVISION`] now name that revision. Those files
are where the pin is enforced: cargo resolves the dependency by exact revision
and refuses a graph that disagrees. This document records the boundary and does
not restate a checksum of it.

The two source files did change between the two revisions, and the clean-room
claim survives it because of *what* changed rather than because the change was
small. The delta over `src/` is a bounded wire decoder
(`MAX_FORMULA_DOCUMENT_NODES`), a `DocumentNodeLimitExceeded` variant,
`FormulaDocument::from_formula` returning `Result`, `#[non_exhaustive]` on four
error types, removal of the `Node` `Deserialize` derive, and a duplicate-name
check moved to a `BTreeMap`. No `NodeKind` variant, interval semantic, span
semantic, or operator spelling is added, removed, or renamed. The vocabulary
this dialect was authored from is therefore the same vocabulary, and the
authored grammar in `DIALECT-001-clean-room-mltl-v1.md` is unaffected.

Both licence files are byte-identical across the two revisions, so the licence
boundary is unchanged.

`tl-parse` uses none of the changed API surface: it never calls
`from_formula`, never deserializes a `Node`, and matches on none of the four
error types made `non_exhaustive`.

## Why there is no digest table

Earlier revisions of this document carried per-file SHA-256 tables for both
revisions, re-derived by a repository-local script. They are gone as of
`agent-ix/tl-parse#15`.

A digest is evidence only if a reader can fetch the same bytes independently and
arrive at the same number. The authorship basis is not fetchable: `740182f1` was
reachable only from tl-syntax's `feat/tl-syntax-v0.1` branch, which has been
deleted, and no ref in that repository reaches it. Its digests could be checked
for shape but never re-derived, which made an unverifiable claim look like a
verified one — worse than recording no number at all.

The compiled revision needs no local table. `Cargo.lock` pins it and cargo
enforces it, which is the ordinary mechanism and a stronger one than a markdown
table maintained by hand. Machine-checkable per-file provenance, if it is ever
wanted here, belongs in a generated SBOM verified against a live source.
