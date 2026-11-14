# Changelog

All notable user-visible changes to `tl-parse` are recorded here. The crate is
distributed as a git source release (`publish = false`); versions are git tags.

## 0.4.0

Part of the second coordinated release of the MLTL crates (`tl-syntax`,
`tl-parse`, `tl-mltl`, `tl-rewrite`). All four crates move to 0.4.0 together,
and this crate depends on `tl-syntax` 0.4.0. Changes are relative to v0.3.0.
This is a breaking 0.x release; the API migration inventory below lists every
break reported by `cargo-semver-checks` 0.50.0.

### Added

- **clean-ascii v4: infinite-trace parsing (TL-14).** The parser accepts the
  infinite-trace profile with exact loci (`DIALECT_V4_REVISION`,
  `DIALECT_V4_DOCUMENT`, `docs/DIALECT-004-clean-ascii-v4.md`). New diagnostics:
  `DiagnosticCode::{InfiniteProfileMismatch, InfiniteClockMismatch,
  DuplicateFairnessPremise}`, `ExpectedToken::{LeftBrace, RightBrace, Semicolon,
  Colon}` and `FormatErrorCode::UnrepresentableGraph`.
- The `HARD_MAX_*` process-safe limit constants are re-exported from the crate
  root, so callers can name the ceilings that clamp `ParseLimits`,
  `FormatLimits`, and `ParseArtifactLimits`.
- `make miri` runs the pure-computation suites under Miri on the pinned nightly
  and is part of `make ci` (NFR-005).
- Boundary tests accept each declared limit at its value and refuse one past it
  with the typed error that names the limit.
- A benchmark harness, a v4 locus corpus and parser fuzz run records (TL-235,
  TL-220, TL-58), plus the cargo-mutants configuration and kill-rate gate
  (TL-231, TL-234).

### Changed

- The v4 canonical-text depth bound is `HARD_MAX_DEPTH` instead of a separate
  literal `256`; the value is unchanged.
- Strict reader reject-class tables (TL-364).
- The mutation gate is the shared Rust tool `tl-gate`, built from `tl-syntax`
  (TL-370).

### Removed

- `ContextualParseError::InvalidCatalog` and `ContextualParseError::Identity`.
  No input can produce either: `SignalCatalogDocument` is validated on
  construction and on deserialization, and the binding digests serialize derive
  trees to a `Vec`. This is a breaking change to an exhaustive `match` over the
  enum.
- `ContextualParseError::MissingParserSpan`. `parse` spans every node it emits,
  so no value is producible; the strict wire reader still refuses a span-less
  node with a string error.
- The compiled-in `tl-syntax` revision: `TL_SYNTAX_REVISION`, the
  `tl_syntax_revision` field of the report structs, and the document digest
  functions. The pinned revision is recorded in `Cargo.toml` only.
- **Wire break:** serialized reports no longer carry `tl_syntax_revision`, and
  the report types reject unknown fields, so 0.4.0 refuses reports written by
  v0.3.0 that still contain it. Schema identities were not bumped, by decision
  for the pre-release 0.x line.
  - `tl-parse.diagnostics/v1`: Migration: regenerate reports; v0.3.0 reports carrying `tl_syntax_revision` are refused by 0.4.0. The release gate's legacy decoder replay proves this one for the parser report, and proves the refusal comes from that field alone.
  - `tl-parse.past-parse-report/v1`, `tl-parse.contextual-binding/v2` and `tl-parse.derived-parse-report/v1`: Migration: regenerate reports; these carry the same removed field. This is reviewer-checked, not replay-proven.

### API migration inventory

- `enum_no_repr_variant_discriminant_changed` `ExpectedToken::Comma`: Migration: do not cast `ExpectedToken` variants to integers with `as`; discriminants shifted when variants were added, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `ExpectedToken::EndOfInput`: Migration: do not cast `ExpectedToken` variants to integers with `as`; discriminants shifted when variants were added, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `FormatErrorCode::OutputLimit`: Migration: do not cast `FormatErrorCode` variants to integers with `as`; discriminants shifted when variants were added, so match on the variant or use an explicit mapping.
- `enum_no_repr_variant_discriminant_changed` `FormatErrorCode::WorkLimit`: Migration: do not cast `FormatErrorCode` variants to integers with `as`; discriminants shifted when variants were added, so match on the variant or use an explicit mapping.
- `enum_variant_added` `DiagnosticCode:DuplicateFairnessPremise`: Migration: add a match arm for `DiagnosticCode::DuplicateFairnessPremise` or a wildcard arm to every exhaustive `match` over `DiagnosticCode`.
- `enum_variant_added` `DiagnosticCode:InfiniteClockMismatch`: Migration: add a match arm for `DiagnosticCode::InfiniteClockMismatch` or a wildcard arm to every exhaustive `match` over `DiagnosticCode`.
- `enum_variant_added` `DiagnosticCode:InfiniteProfileMismatch`: Migration: add a match arm for `DiagnosticCode::InfiniteProfileMismatch` or a wildcard arm to every exhaustive `match` over `DiagnosticCode`.
- `enum_variant_added` `ExpectedToken:Colon`: Migration: add a match arm for `ExpectedToken::Colon` or a wildcard arm to every exhaustive `match` over `ExpectedToken`.
- `enum_variant_added` `ExpectedToken:LeftBrace`: Migration: add a match arm for `ExpectedToken::LeftBrace` or a wildcard arm to every exhaustive `match` over `ExpectedToken`.
- `enum_variant_added` `ExpectedToken:RightBrace`: Migration: add a match arm for `ExpectedToken::RightBrace` or a wildcard arm to every exhaustive `match` over `ExpectedToken`.
- `enum_variant_added` `ExpectedToken:Semicolon`: Migration: add a match arm for `ExpectedToken::Semicolon` or a wildcard arm to every exhaustive `match` over `ExpectedToken`.
- `enum_variant_added` `FormatErrorCode:UnrepresentableGraph`: Migration: add a match arm for `FormatErrorCode::UnrepresentableGraph` or a wildcard arm to every exhaustive `match` over `FormatErrorCode`.
- `enum_variant_missing` `ContextualParseError::Identity`: Migration: delete the match arm; no input could produce this variant.
- `enum_variant_missing` `ContextualParseError::InvalidCatalog`: Migration: delete the match arm; no input could produce this variant.
- `enum_variant_missing` `ContextualParseError::MissingParserSpan`: Migration: delete the match arm; no input could produce this variant.
- `function_missing` `tl_parse::attribution_document_digest`: Migration: delete calls and read the document text from the `DIALECT_*_DOCUMENT` constants if a digest is needed.
- `function_missing` `tl_parse::dialect_document_digest`: Migration: delete calls and read the document text from the `DIALECT_*_DOCUMENT` constants if a digest is needed.
- `function_missing` `tl_parse::dialect_v2_document_digest`: Migration: delete calls and read the document text from the `DIALECT_*_DOCUMENT` constants if a digest is needed.
- `function_missing` `tl_parse::dialect_v3_document_digest`: Migration: delete calls and read the document text from the `DIALECT_*_DOCUMENT` constants if a digest is needed.
- `partial_ord_enum_variants_reordered` `ExpectedToken::Comma`: Migration: do not rely on the derived `PartialOrd` order of `ExpectedToken`; sort by an explicit key.
- `partial_ord_enum_variants_reordered` `ExpectedToken::EndOfInput`: Migration: do not rely on the derived `PartialOrd` order of `ExpectedToken`; sort by an explicit key.
- `pub_module_level_const_missing` `TL_SYNTAX_REVISION`: Migration: delete uses; read the pinned `tl-syntax` revision from `Cargo.toml` or `Cargo.lock` instead.
- `struct_pub_field_missing` `ContextualParseReport::tl_syntax_revision`: Migration: stop reading `ContextualParseReport.tl_syntax_revision`; serialized reports no longer carry it, and the report decoders refuse reports that still do (see the wire break above).
- `struct_pub_field_missing` `DerivedParseReport::tl_syntax_revision`: Migration: stop reading `DerivedParseReport.tl_syntax_revision`; serialized reports no longer carry it, and the report decoders refuse reports that still do (see the wire break above).
- `struct_pub_field_missing` `ParseReport::tl_syntax_revision`: Migration: stop reading `ParseReport.tl_syntax_revision`; serialized reports no longer carry it, and the report decoders refuse reports that still do (see the wire break above).
- `struct_pub_field_missing` `PastParseReport::tl_syntax_revision`: Migration: stop reading `PastParseReport.tl_syntax_revision`; serialized reports no longer carry it, and the report decoders refuse reports that still do (see the wire break above).

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
  Each dialect publishes its normative record and document, with a record
  digest (`DIALECT_V2_RECORD`, `DIALECT_V3_RECORD`, `DIALECT_V2_DOCUMENT`,
  `DIALECT_V3_DOCUMENT`, `dialect_v2_digest`, `dialect_v3_digest`). Every successful parse is
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

- **MSRV is now Rust 1.98.1.**
- **Depends on tl-syntax 0.3.0.**
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
  tl-syntax 0.3.0 CHANGELOG.
