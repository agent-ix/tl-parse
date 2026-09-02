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
`740182f13b84858008d6f176f75136737d405c1b`, licensed MIT OR Apache-2.0.

Files consulted at that revision:

| File | SHA-256 |
|---|---|
| `src/syntax.rs` | `04e6a46e697444df8e6764dd0e5e5227b1271199ffc0e9d24f77720c979eb14e` |
| `src/document.rs` | `f97005479f1f12511f1fceb2f9a85b94b482170e606c5735758e11aa2e4580f2` |
| `LICENSE-MIT` | `97ead12ddb151fc37ffb1c623ab42b9814e21629dee252ff23dc7205f1df9f05` |
| `LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |

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
`fuzz/Cargo.lock` and [`TL_SYNTAX_REVISION`] now name that revision.

| File | SHA-256 at `953ee825` |
|---|---|
| `src/syntax.rs` | `76a5f72f6ee2791b9665ffacab057b1d62a262fa6a56a80bfbe443235d368647` |
| `src/document.rs` | `8ba023faf819b1934b06e47b536e7aa1739143df3a82f7fc426ecf91a6aa1268` |
| `LICENSE-MIT` | `97ead12ddb151fc37ffb1c623ab42b9814e21629dee252ff23dc7205f1df9f05` |
| `LICENSE-APACHE` | `62c7a1e35f56406896d7aa7ca52d0cc0d272ac022b5d2796e7d6905db8a3636a` |

Both licence files are byte-identical across the two revisions, so the licence
boundary is unchanged.

The two source files did change, and the clean-room claim survives it because
of *what* changed rather than because the change was small. The delta over
`src/` is a bounded wire decoder (`MAX_FORMULA_DOCUMENT_NODES`), a
`DocumentNodeLimitExceeded` variant, `FormulaDocument::from_formula` returning
`Result`, `#[non_exhaustive]` on four error types, removal of the `Node`
`Deserialize` derive, and a duplicate-name check moved to a `BTreeMap`. No
`NodeKind` variant, interval semantic, span semantic, or operator spelling is
added, removed, or renamed. The vocabulary this dialect was authored from is
therefore the same vocabulary, and the authored grammar in
`DIALECT-001-clean-room-mltl-v1.md` is unaffected.

`tl-parse` uses none of the changed API surface: it never calls
`from_formula`, never deserializes a `Node`, and matches on none of the four
error types made `non_exhaustive`.
