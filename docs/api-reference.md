# API reference

Everything is exported from the crate root, and `tl_parse::tl_syntax` re-exports the tl-syntax crate that the pinned revision provides. The public API is not stable.

## Parsing

| Item | Purpose |
| --- | --- |
| `parse(source, profile, limits) -> ParseReport` | v1 dialect. |
| `parse_clean_ascii_v2(source, profile, limits) -> DerivedParseReport` | v2 dialect, with `W` and `M` lowered. |
| `parse_clean_ascii_v3(source, limits) -> PastParseReport` | v3 dialect. The profile is fixed at `OriginCompleteHistoryV1`. |
| `parse_clean_ascii_v4(source, profile, clock, limits) -> InfiniteParseReport` | v4 dialect. Requires `InfiniteTraceV1` and `"event_position"`. |
| `parse_with_context(source, profile, limits, catalog, context) -> Result<ContextualParseReport, ContextualParseError>` | v1 parse, then binds each free proposition to a signal in a `SignalCatalogDocument`. `context` is an optional `RequirementContextDocument`. |
| `source_limit_report(source_bytes, profile, limits) -> ParseReport` | The refusal for an input whose length is known but whose bytes were not kept. Used by the command-line tool. |

## Formatting

| Item | Purpose |
| --- | --- |
| `format_document(&FormulaDocument, FormatLimits) -> FormatReport` | Canonical v1 text for a closed or online document, or v3 text for a formula-v2 document under the history profile. |
| `format_clean_ascii_v3(&FormulaDocument, FormatLimits) -> FormatReport` | v3 text. Refuses any other schema or profile. |
| `format_clean_ascii_v4(&InfiniteFormulaDocument, Option<&FairnessPremisesDocument>, FormatLimits) -> FormatReport` | v4 text, with the `fair {...}:` envelope when premises are present. |
| `format_formula(Formula<'_>, FormatLimits) -> FormatReport` | Formats a borrowed tl-syntax formula. |

## Reports

| Type | Fields of note |
| --- | --- |
| `ParseReport` | `schema_version`, `dialect_revision`, `semantic_profile`, `limits`, `stats`, `document: Option<FormulaDocument>`, `diagnostics`. |
| `DerivedParseReport` | The same, plus `operator_profile` and `lowerings: Vec<LoweringRecord>`. Deserialization checks that lowering records name document nodes. |
| `PastParseReport` | The same as `ParseReport` with `operator_profile`. `from_json_bytes(bytes, ParseArtifactLimits)` and `canonical_json_bytes()`. |
| `InfiniteParseReport` | `clock`, `document: Option<InfiniteFormulaDocument>`, `fairness`, `interval_spans`, `premise_spans`, `diagnostics`, `refusal`. `from_json_bytes` and `disposition()`. |
| `ContextualParseReport` | `formula_document`, `signal_catalog`, `signal_catalog_sha256`, `requirement_context`, `request_sha256`, `bindings: Vec<BoundProposition>`. |
| `FormatReport` | `limits`, `stats`, `text: Option<String>`, `error: Option<FormatError>`. |
| `ParseStats`, `FormatStats` | Observed counts. |
| `LoweringRecord` | `kind: DerivedOperator`, `operator_span`, `expression_span`, `left`, `right`, `first_generated`, `root`. |

`report_json(&ParseReport) -> Result<String, serde_json::Error>` serializes a v1 report as compact JSON.

## Diagnostics and errors

| Type | Purpose |
| --- | --- |
| `Diagnostic` | `code`, `severity`, `span`, `found`, `expected`, `recovery`, `message`. |
| `DiagnosticCode` | The 18 stable codes. `as_str()`. See [diagnostics.md](diagnostics.md). |
| `DiagnosticSeverity`, `ExpectedToken`, `RecoveryAction` | Closed enums used by `Diagnostic`. |
| `InfiniteRefusal` | The decisive v4 refusal: `code` and `span`. |
| `InfiniteDisposition` | `Unsupported`, `ResourceIncomplete`, `Failed`, `NoTemporalVerdict`. |
| `FormatError`, `FormatErrorCode` | Formatting failures. `as_str()`. |
| `ContextualParseError` | `ParseFailed`, `InvalidCatalog`, `MissingParserSpan`, `MissingProposition`, `Identity`. |
| `StrictParseArtifactReadError` | Strict reader failures. `#[non_exhaustive]`. |

## Limits

`ParseLimits`, `FormatLimits` and `ParseArtifactLimits` are plain structs with public fields and a `Default`. `ParseArtifactLimits::OWNER_MAXIMA` is the ceiling for strict readers. Values and maxima are in [diagnostics.md](diagnostics.md).

## Identities and records

| Constant | Value |
| --- | --- |
| `DIALECT_REVISION`, `DIALECT_V2_REVISION`, `DIALECT_V3_REVISION`, `DIALECT_V4_REVISION` | `tl-parse.clean-ascii/v1` to `v4`. |
| `DIAGNOSTIC_SCHEMA_VERSION` | `tl-parse.diagnostics/v1`. |
| `DERIVED_PARSE_REPORT_SCHEMA_VERSION` | `tl-parse.derived-parse-report/v1`. |
| `PAST_PARSE_REPORT_SCHEMA_VERSION` | `tl-parse.past-parse-report/v1`. |
| `CORPUS_REVISION` | `tl-parse-corpus/v1`, the revision of `corpus/v1`. |
| `DIALECT_RECORD`, `DIALECT_V2_RECORD`, `DIALECT_V3_RECORD` | The normative production and precedence records. |
| `DIALECT_DOCUMENT`, `DIALECT_V2_DOCUMENT` to `DIALECT_V4_DOCUMENT`, `ATTRIBUTION_DOCUMENT` | The files in this directory, embedded in the crate. |

`dialect_digest()`, `dialect_v2_digest()` and `dialect_v3_digest()` return the lowercase SHA-256 of the matching record.

The enums `ContextualParseSchemaVersion`, `DerivedParseSchemaVersion`, `PastParseSchemaVersion`, `InfiniteParseSchemaVersion` and the dialect and operator-profile enums (`DerivedDialectRevision`, `DerivedOperatorProfile`, `PastDialectRevision`, `PastOperatorProfile`, `InfiniteDialectRevision`) carry the wire identities of the reports.
