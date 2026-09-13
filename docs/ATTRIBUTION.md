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

The compiled revision is
`e3651cde5524c61cb9623ce39fcc9f0b90b99317`, a commit that was reachable from
the reviewed `tl-syntax` `main` history when this pin was admitted. It is not a
moving branch head. `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock` and
[`TL_SYNTAX_REVISION`] name that exact revision. Those files are where the pin
is enforced: cargo resolves the dependency by exact revision and refuses a
graph that disagrees. This document records the boundary and does not restate a
checksum of it.

## Source-inspected compiled-pin delta

The immediately preceding compiled pin was
`26b801d4a68ebfe720062cfdb3c66b070ab60e92`. Source inspection of the exact
range
`26b801d4a68ebfe720062cfdb3c66b070ab60e92..e3651cde5524c61cb9623ce39fcc9f0b90b99317`
found these changes:

- the future-operator lowering family in `src/future.rs`
  (`FutureLoweringRequest`, `FutureLowering`, `FutureLoweringReport`,
  `FutureLoweringRefusal`, `FutureKind`, `UnsupportedFutureKind`, `RawBounds`,
  and their identity constants), which lowers bounded `W` and `M` into primitive
  nodes;
- `MAX_FORMULA_DOCUMENT_NODES` moved from `src/document.rs` to `src/syntax.rs`
  and is still re-exported at the crate root with the same value; and
- a crate-private `SemanticProfile::ALL` constant, plus specification and
  assurance records.

tl-parse consumes the future-operator lowering family for the explicitly
selected `tl-parse.clean-ascii/v2` dialect in
`DIALECT-002-clean-ascii-v2.md`. That dialect was authored from the tl-syntax
FR-009 requirement and the public lowering API, not from any third-party
grammar. The v1 grammar is unaffected. `src/syntax.rs` introduced no operator or
node-kind change in this range.

Both licence files are byte-identical across `26b801d4` and `e3651cde`.

### Earlier compiled-pin delta

The compiled pin before that was
`953ee825e5060335b4c79682f5f41a78c5a1bfae`. Source inspection of the exact
range
`953ee825e5060335b4c79682f5f41a78c5a1bfae..26b801d4a68ebfe720062cfdb3c66b070ab60e92`
found these changes:

- caller-context APIs and their owned wire document (`RequirementContext` and
  `RequirementContextDocument` families);
- signal declarations, scalar domains, proposition bindings, validated signal
  catalogs, bound-formula views, and their owned wire document;
- span-free semantic formula identity through `SemanticFormulaDocument`,
  `FormulaDocument::semantic_view`, and the new `Hash` implementation on
  `FormulaDocument`; and
- assurance-only changes: removal of the legacy evidence archive, tracked
  source-census and qualification controls, property/fuzz targets, and their
  specification and review records.

tl-parse consumes the shared `RequirementContextDocument`,
`SignalCatalogDocument`, and `SignalId` families for its additive context-bound
parse report. It continues to use the pre-existing `Formula`,
`FormulaDocument`, `Interval`, `Node`, `NodeId`, `NodeKind`, `PropositionId`,
`SemanticProfile`, and `SourceSpan` graph, interval, span, proposition, and
semantic-profile contracts; it does not consume the span-free semantic-formula
identity family. No later grammar source was consulted: `src/syntax.rs`
introduced no operator or grammar change in this range, and the independently authored grammar in
`DIALECT-001-clean-room-mltl-v1.md` is unaffected.

The clean-room claim therefore survives these implementation and assurance
changes because the authorship basis remains the exact earlier source corpus.

Both licence files are byte-identical across the two revisions, so the licence
boundary is unchanged.

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
