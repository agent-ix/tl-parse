# Changelog

All notable user-visible changes to `tl-parse` are recorded here. The crate is
distributed as a git source release (`publish = false`); versions are git tags.

## 0.4.0

Stage 1 candidate. The clean-ascii v4 parser adds infinite-trace syntax and
fairness reports. This section records the public API migrations measured
against the immutable 0.3.0 tag; the coordinated release tag is still pending.

### API migration inventory

- `enum_variant_added` `ExpectedToken:LeftBrace`: Migration: handle `{` explicitly in exhaustive expected-token matches, including diagnostic renderers.
- `enum_variant_added` `ExpectedToken:RightBrace`: Migration: handle `}` explicitly in exhaustive expected-token matches, including diagnostic renderers.
- `enum_variant_added` `ExpectedToken:Semicolon`: Migration: handle `;` explicitly in exhaustive expected-token matches, including diagnostic renderers.
- `enum_variant_added` `ExpectedToken:Colon`: Migration: handle `:` explicitly in exhaustive expected-token matches, including diagnostic renderers.
- `enum_variant_added` `DiagnosticCode:InfiniteProfileMismatch`: Migration: handle an unsupported infinite semantic profile as a typed parse refusal.
- `enum_variant_added` `DiagnosticCode:InfiniteClockMismatch`: Migration: handle an unsupported infinite clock as a typed parse refusal.
- `enum_variant_added` `DiagnosticCode:DuplicateFairnessPremise`: Migration: handle duplicate fairness premises as a typed parse refusal.
- `enum_variant_added` `FormatErrorCode:UnrepresentableGraph`: Migration: handle a valid graph that cannot be represented in the selected text dialect without emitting partial text.
- `enum_no_repr_variant_discriminant_changed` `ExpectedToken::Comma`: Migration: do not persist or compare numeric casts of `ExpectedToken`; match variants or use an application-owned stable code. Its implicit discriminant shifts from 5 to 9.
- `enum_no_repr_variant_discriminant_changed` `ExpectedToken::EndOfInput`: Migration: do not persist or compare numeric casts of `ExpectedToken`; match variants or use an application-owned stable code. Its implicit discriminant shifts from 6 to 10.
- `enum_no_repr_variant_discriminant_changed` `FormatErrorCode::OutputLimit`: Migration: match `FormatErrorCode` variants directly or assign an application-owned code; its implicit discriminant shifts from 1 to 2.
- `enum_no_repr_variant_discriminant_changed` `FormatErrorCode::WorkLimit`: Migration: match `FormatErrorCode` variants directly or assign an application-owned code; its implicit discriminant shifts from 2 to 3.
- `partial_ord_enum_variants_reordered` `ExpectedToken::Comma`: Migration: replace derived `ExpectedToken` ordering in persisted or protocol decisions with an explicit key. Its position shifts from 6 to 10.
- `partial_ord_enum_variants_reordered` `ExpectedToken::EndOfInput`: Migration: replace derived `ExpectedToken` ordering in persisted or protocol decisions with an explicit key. Its position shifts from 7 to 11.

## 0.3.0

Part of the first coordinated release of the MLTL crates (`tl-syntax`,
`tl-parse`, `tl-mltl`, `tl-rewrite`). The version skips 0.2.0 so all four
crates share one number past `tl-mltl`'s existing v0.2.0. Changes are relative
to v0.1.0.

Release gate: the full local gate (`make ci`) passes at this revision against
`tl-syntax` v0.3.0. This crate has no separate `make spec-release` target:
`make spec`, which `make ci` runs, already applies strict Quire coverage
(`quire coverage --strict`), so every active specification row is backed.

### Added

- **clean-ascii v2: bounded `W` and `M`.** `parse_clean_ascii_v2` accepts the
  derived future operators weak-until `W[a,b]` and strong-release `M[a,b]`
  (same precedence and left associativity as `U`/`R`) and lowers them through
  tl-syntax's future-operator lowering API, so the returned document only
  contains the canonical F/G/U/R core. The report is a `DerivedParseReport`
  (`tl-parse.derived-parse-report/v1`) with one `LoweringRecord` per derived
  operator; see `DerivedOperator`, `DerivedOperatorProfile`,
  `DerivedDialectRevision`, `DerivedParseSchemaVersion`,
  `DIALECT_V2_REVISION` and `docs/DIALECT-002-clean-ascii-v2.md`.
- **clean-ascii v3: past/history parsing and formatting.**
  `parse_clean_ascii_v3` parses the origin-complete past profile
  (`mltl.origin-complete-history/v1`) with `O`, `H`, strong `Y`, `S` and `T`
  into tl-syntax formula-v2 documents, and `format_clean_ascii_v3` emits their
  canonical text. The report is a `PastParseReport`
  (`tl-parse.past-parse-report/v1`); `PastParseReport::from_json_bytes(bytes,
  ParseArtifactLimits)` is its bounded canonical reader and
  `canonical_json_bytes()` its writer. See `PastOperatorProfile`,
  `PastDialectRevision`, `PastParseSchemaVersion`, `DIALECT_V3_REVISION` and
  `docs/DIALECT-003-clean-ascii-v3.md`.
- **Versioned dialect subsystem.** Lexing, parsing, formatting and diagnostics
  are organized around closed `v1`/`v2`/`v3` dialect policies that each own
  their spellings, precedence, associativity, semantic profile, owner schema
  and lowering permission, while sharing traversal and resource accounting.
  Each dialect publishes its normative record and document with digests
  (`DIALECT_V2_RECORD`, `DIALECT_V3_RECORD`, `DIALECT_V2_DOCUMENT`,
  `DIALECT_V3_DOCUMENT`, `dialect_v2_digest`, `dialect_v2_document_digest`,
  `dialect_v3_digest`, `dialect_v3_document_digest`). Every successful parse is
  re-admitted from canonical bytes through tl-syntax's strict
  `FormulaDocument::from_json_bytes` reader.
- **Strict artifact reader limits.** `ParseArtifactLimits` (with
  `OWNER_MAXIMA` and the `HARD_MAX_PARSE_ARTIFACT_*` ceilings) bounds strict
  report reads; callers can lower, never raise, them. Failures are
  `StrictParseArtifactReadError` (`#[non_exhaustive]`).
- **`unsupported_operator` diagnostic.** `DiagnosticCode::UnsupportedOperator`
  is emitted by the v2 and v3 entry points for an operator that belongs to a
  different dialect (for example `O` given to v2, or `F` given to v3). The v1
  entry point does not emit it.
- **Shared past-history corpus replay.** The parser replays tl-syntax's shared
  past-history corpus, read in place through `tl_syntax::CORPUS_DIR` rather
  than a vendored copy.

### Changed

- **MSRV is now Rust 1.98.1** (was 1.75). `rust-toolchain.toml` pins that exact
  toolchain.
- **Depends on tl-syntax 0.3.0.** The dependency is pinned by exact revision
  `4a5614193d21e5ae99950ae683b04ba0ec931358` (tag `v0.3.0`) with version
  requirement `=0.3.0`; `TL_SYNTAX_REVISION` names that revision. tl-syntax
  0.3.0 changes no source relative to the previous pin.
- **Typed signal catalog binding** (`parse_with_context`,
  `ContextualParseReport`, `BoundProposition`, first shipped in v0.1.0) is
  unchanged in behavior and now resolves the signal catalog, proposition map
  and requirement context types from tl-syntax 0.3.0.
- The v1 API (`parse`, `format_document`, `format_formula`) and the CLI accept
  and emit the same text as in v0.1.0.

### Breaking changes

- **`DiagnosticCode` has a new variant** `UnsupportedOperator`
  (`"unsupported_operator"` on the wire). The enum is exhaustive.
  *Migration:* add an arm for it (or a wildcard) to every `match` on
  `DiagnosticCode`, and accept the new string wherever serialized diagnostics
  are deserialized or compared.
- **The re-exported `tl_syntax` is 0.3.0**, so its breaking changes reach
  code that uses `tl_parse::tl_syntax`: `SemanticProfile` gains
  `OriginCompleteHistoryV1`, `NodeKind` gains `Once`, `Historically`,
  `StrongPrevious`, `Since` and `Triggered`, `FormulaSchemaVersion` gains `V2`,
  and formulas nested deeper than 4096 are refused. *Migration:* follow the
  tl-syntax 0.3.0 CHANGELOG; depend on tl-syntax at the same revision and
  `=0.3.0` if you also name it directly, so one tl-syntax resolves.
- **Rust 1.75 through 1.98.0 can no longer build the crate.** *Migration:*
  build with Rust 1.98.1 or newer and raise your own `rust-version` to match.
- **`TL_SYNTAX_REVISION` changes value** to
  `4a5614193d21e5ae99950ae683b04ba0ec931358`. *Migration:* update any
  recorded or asserted copy of it.
