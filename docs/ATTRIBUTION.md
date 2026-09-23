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
`9de638dc4d14d0ae62a6825473a3a9bb6a9e57ac`, an exact reviewed commit.
It is not a
moving branch head. `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock` and
[`TL_SYNTAX_REVISION`] name that exact revision. Those files are where the pin
is enforced: cargo resolves the dependency by exact revision and refuses a
graph that disagrees. This document records the boundary and does not restate a
checksum of it.

The previous `4a561419` pin is the squash merge of tl-syntax#88 and was the commit tagged
`v0.3.0`. The current pin retains the version requirement
`=0.3.0` while Stage 1 remains prerelease. V4 consumes the additive
`formula-unbounded/v1`, fairness, and infinite trace identities. V1 through
V3 continue to use the bounded owner vocabulary; the authorship basis above
does not move.
The prior pin advance from `8bcbce98` to `fed48a2f` added V4 fuzz evidence.
The advance to `6d182fa9` made tl-syntax MIT-only. The `9a4316e`
advance fixes alloc-only test compilation and adds V8 coverage tests. The
current `9de638d` advance adds an alloc-lane wrong-profile test only. These
advances do not change the parser-consumed production API or historical
authorship basis.

## Source-inspected 0.3.0 release compiled-pin delta

The immediately preceding compiled pin was
`d52d89549b0a6c0c429261bab912cd5396c4a19e`. Source inspection of the exact
range
`d52d89549b0a6c0c429261bab912cd5396c4a19e..4a5614193d21e5ae99950ae683b04ba0ec931358`
found no change under `src/`: the public API tl-parse compiles against is
unchanged. The package metadata moves `version` from `0.1.0` to `0.3.0` and
`rust-version` from `1.75` to `1.98.1`, which is why this crate's MSRV moves to
1.98.1 in the same release. The only corpus change is a correction of stale
vendoring language in `corpus/past-history/README.md`, which changes that
file's recorded digest in `corpus/past-history/manifest.json` and
`SHA256SUMS`; `cases.json` and `schema.json` are byte-identical across the
range. The remainder of the range is tl-syntax's own tooling and records: its
MSRV and toolchain pin, the engineering-assurance v0.2.1 / ix-flow 0.2.3
repin, a cross-document spec-id uniqueness check, a test-matrix column rename,
review renumbering, the PLAN-007 source-qualification-readiness specification
(prose and plan records only), and the 0.3.0 CHANGELOG.

tl-parse consumes nothing new from this range. It reads the corrected
past-history manifest through `tl_syntax::CORPUS_DIR` exactly as before, and
the manifest digest it asserts moves with the README correction. It introduces
no new dependency on any other file this range touches.

Both licence files are byte-identical across `d52d8954` and `4a561419`.

## Earlier source-inspected CORPUS_DIR compiled-pin delta

`d52d8954` is the squash merge of tl-syntax#82 and exports `tl_syntax::CORPUS_DIR`
so dependents can read the shared `corpus/` tree through the compiled
dependency instead of vendoring a copy of it.

The immediately preceding compiled pin was
`842d82553f045eb69a7f38745756d968254fc25e`. Source inspection of the exact
range
`842d82553f045eb69a7f38745756d968254fc25e..d52d89549b0a6c0c429261bab912cd5396c4a19e`
found exactly one source-code change: a 7-line addition to `src/lib.rs`
introducing `pub const CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"),
"/corpus")`, a path resolved at tl-syntax's own compile time so a git
dependent can join a corpus-relative path onto it and read the checked-in
corpus directly. The only other code-adjacent change in the range is an
internal hardening of tl-syntax's own `tests/past_formula_v2.rs` proptest
(`assert!` replaced with `prop_assert!` so a failure shrinks instead of
unwinding); that file is a test of tl-syntax's own crate, not part of its
public API, and tl-parse does not consume it. The range also adds
specification-only content — `FR-289` (infinite-trace interval grammar),
`FR-290` (liveness capability registration), `FR-291` (infinite-trace
downstream evidence), and a cross-reference from `FR-008` to `FR-289` — that
describes a future `UnboundedInterval` extension to the future-operator
lowering family. No corresponding implementation exists anywhere in this
range: the addition is prose only. The remainder of the range is CLA/CI
enablement, `.gitignore` housekeeping, a README badge, and PLAN-010 closure
specification and review records.

tl-parse consumes only the new `CORPUS_DIR` constant, and only to read
tl-syntax's own `corpus/past-history/` in place instead of vendoring a
byte-identical copy of it (see `tests/past_history_corpus.rs`). It does not
consume the FR-289/290/291 infinite-trace specification additions — there is
no implementation of them in this range to consume — and it introduces no new
dependency on any other file this range touches.

Both licence files are byte-identical across `842d8255` and `d52d8954`.

## Earlier source-inspected strict-owner compiled-pin delta

The immediately preceding compiled pin was
`e70f2379a752117c79603bc399a86c26feed7716`. Source inspection of the exact
range
`e70f2379a752117c79603bc399a86c26feed7716..842d82553f045eb69a7f38745756d968254fc25e`
found the cycle-free `formula`, `signal`, and `contracts` topology; canonical
formula-v1/v2 JSON bytes and identities; caller-lowered immutable owner limits;
the bounded strict `FormulaDocument::from_json_bytes` reader; strict signal
catalog and proposition-map readers; and the unified ecosystem architecture and
tracking artifacts.

tl-parse consumes only the reorganized compatibility exports, canonical formula
bytes, `SyntaxArtifactLimits`, and the strict `FormulaDocument` reader. It does
not consume or mirror the signal/proposition contracts, and it introduces no
evaluator, rewrite rule, monitoring state, or Contract-IR vocabulary.

Both licence files are byte-identical across `e70f2379` and `842d8255`.

## Earlier source-inspected formula-v2 compiled-pin delta

The immediately preceding compiled pin was
`8dc18eec5af227f484170362c9e8894b8531a27d`. Source inspection of the exact
range
`8dc18eec5af227f484170362c9e8894b8531a27d..e70f2379a752117c79603bc399a86c26feed7716`
found the additive formula-v2 schema, origin-complete history semantic profile,
O/H/Y/S/T node variants, closed past operator catalog, profile validation,
conversion API, and owned v2 resource bounds. The existing formula-v1 graph,
future-lowering request, and clean-ascii/v1/v2 inputs remain compatibility
boundaries.

tl-parse consumes those new contracts only through the explicitly selected
`tl-parse.clean-ascii/v3` entry point. The past spelling and precedence rules
were authored from accepted tl-syntax MRS-003/FR-013 and are recorded in
`DIALECT-003-clean-ascii-v3.md`; no third-party parser or grammar was consulted.

Both licence files are byte-identical across `8dc18eec` and `e70f2379`.

### Earlier future-operator compiled-pin delta

## Source-inspected compiled-pin delta

The immediately preceding compiled pin was
`26b801d4a68ebfe720062cfdb3c66b070ab60e92`. Source inspection of the exact
range
`26b801d4a68ebfe720062cfdb3c66b070ab60e92..8dc18eec5af227f484170362c9e8894b8531a27d`
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

Both licence files are byte-identical across `26b801d4` and `8dc18eec`.

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
