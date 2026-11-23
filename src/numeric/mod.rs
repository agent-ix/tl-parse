//! `tl-parse.clean-ascii/v5`: numeric predicates over a signal catalog (TL-421).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL program decision
//! 2026-11-23). Every acceptance criterion is listed in the deferred
//! verification table of PLAN-015.
//!
//! The dialect is explicitly selected, always context-bound (a validated
//! `tl-syntax.signal-catalog/v1` document is a required argument), and admits
//! only the `mltl.closed-trace/v1` and `mltl.online-prefix/v1` profiles
//! (FR-028). Comparisons parse into a `tl-syntax.predicate-table/v1` document;
//! the temporal skeleton is the v2 vocabulary with `W`/`M` lowered through the
//! owner lowering. Source text accepted here is derived interchange and makes no
//! claim about an FSL requirement (ADR-020 decision 6).
//!
//! The v5 diagnostics vocabulary is separate from [`crate::DiagnosticCode`],
//! which is not extended (FR-028 behavior).

mod format;
mod lexer;
mod model;
mod parser;

use std::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{
    ExprNode, FormulaDocument, Predicate, PredicateTableSchemaVersion, SemanticProfile,
    SignalCatalogDocument, SourceSpan,
};

use crate::{
    diagnostic::preflight_parse_artifact, lexer::checked_span, ParseArtifactLimits, ParseLimits,
    ParseStats, StrictParseArtifactReadError,
};

pub use format::format_clean_ascii_v5;
// UNBUILT-PIN: `PredicateTableDocument` is the tl-syntax FR-303 owned document
// (TL-420), reached through the `[patch]` in Cargo.toml until the merged rev
// is pinned.
pub use model::PredicateTableDocument;

/// Stable identity of the numeric predicate dialect.
// SPEC-GAP: FR-028 calls `DIALECT_V5_REVISION` "an integer"; every earlier
// `DIALECT_V*_REVISION` is the identity string, so the string keeps that name
// and the integer is `DIALECT_V5_REVISION_NUMBER`.
pub const DIALECT_V5_REVISION: &str = "tl-parse.clean-ascii/v5";

/// Integer revision of the v5 dialect (no digest record, unlike v4; ADR-020).
pub const DIALECT_V5_REVISION_NUMBER: u32 = 1;

/// Stable identity of serialized v5 parse reports.
pub const NUMERIC_PARSE_REPORT_SCHEMA_VERSION: &str = "tl-parse.numeric-parse-report/v1";

/// Stable identity of the closed v5 diagnostics vocabulary.
pub const NUMERIC_DIAGNOSTICS_SCHEMA_VERSION: &str = "tl-parse.numeric-diagnostics/v1";

/// Normative v5 dialect document.
pub const DIALECT_V5_DOCUMENT: &str = include_str!("../../docs/DIALECT-005-clean-ascii-v5.md");

/// Maximum fractional digits (decimal scale) of a v5 literal.
pub const MAX_NUMERIC_SCALE: u8 = tl_syntax::MAX_DECIMAL_SCALE;

/// Maximum bytes of a v5 identifier (equal to the catalog signal-name bound).
pub const MAX_NUMERIC_IDENTIFIER_BYTES: usize = 255;

/// Closed `tl-parse.numeric-diagnostics/v1` code set (FR-030).
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericDiagnosticCode {
    /// Source exceeds the effective byte limit.
    SourceLimit,
    /// Token count exceeds the effective token limit.
    TokenLimit,
    /// Formula plus expression nodes exceed the effective node limit.
    NodeLimit,
    /// Nesting exceeds the effective depth limit.
    DepthLimit,
    /// Parser work exceeds the effective work limit.
    WorkLimit,
    /// A byte has no meaning in the dialect.
    UnexpectedCharacter,
    /// A token is not valid at its grammar position.
    UnexpectedToken,
    /// A required token is absent.
    MissingToken,
    /// Input remains after a complete document.
    TrailingInput,
    /// A literal is not in its one canonical spelling.
    NonCanonicalNumber,
    /// A literal does not fit its integer range.
    IntegerOverflow,
    /// A recognized operator or form outside v5.
    UnsupportedOperator,
    /// The selected profile is not a v5 profile.
    ProfileMismatch,
    /// An identifier names no catalog signal.
    UnknownSignal,
    /// A Boolean signal is read as a number.
    NonNumericSignal,
    /// Operands of `+`, `-` or a comparison differ in scale.
    ScaleMismatch,
    /// A scale exceeds 18.
    ScaleOverflow,
    /// `scale(e, k)` with `k` below the scale of `e`.
    RescaleNarrowing,
    /// A derived coefficient bound does not fit `i64`.
    RangeOverflow,
    /// A reserved word or `p<digits>` is used as a signal name.
    ReservedIdentifier,
    /// A chained comparison such as `a < b < c`.
    NonAssociativeComparison,
    /// A comparison outside parentheses.
    UnparenthesizedComparison,
    /// A quantifier names an undeclared set (TL-424).
    UndefinedSet,
    /// A set name is declared twice (TL-424).
    DuplicateSet,
    /// A bound variable is used outside its template (TL-424).
    SlotOutsideTemplate,
    /// An expansion exceeds the tl-syntax bound (TL-424).
    AggregationTooLarge,
    /// A contract name is declared twice (TL-425).
    DuplicateContract,
    /// A contract lacks `assume` or `guarantee` (TL-425).
    MissingContractClause,
    /// Fresh proposition allocation passes `u32::MAX`.
    PropositionOverflow,
}

impl NumericDiagnosticCode {
    /// Every code of the closed set, in declaration order.
    pub const ALL: [Self; 29] = [
        Self::SourceLimit,
        Self::TokenLimit,
        Self::NodeLimit,
        Self::DepthLimit,
        Self::WorkLimit,
        Self::UnexpectedCharacter,
        Self::UnexpectedToken,
        Self::MissingToken,
        Self::TrailingInput,
        Self::NonCanonicalNumber,
        Self::IntegerOverflow,
        Self::UnsupportedOperator,
        Self::ProfileMismatch,
        Self::UnknownSignal,
        Self::NonNumericSignal,
        Self::ScaleMismatch,
        Self::ScaleOverflow,
        Self::RescaleNarrowing,
        Self::RangeOverflow,
        Self::ReservedIdentifier,
        Self::NonAssociativeComparison,
        Self::UnparenthesizedComparison,
        Self::UndefinedSet,
        Self::DuplicateSet,
        Self::SlotOutsideTemplate,
        Self::AggregationTooLarge,
        Self::DuplicateContract,
        Self::MissingContractClause,
        Self::PropositionOverflow,
    ];

    /// Returns the stable wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceLimit => "source_limit",
            Self::TokenLimit => "token_limit",
            Self::NodeLimit => "node_limit",
            Self::DepthLimit => "depth_limit",
            Self::WorkLimit => "work_limit",
            Self::UnexpectedCharacter => "unexpected_character",
            Self::UnexpectedToken => "unexpected_token",
            Self::MissingToken => "missing_token",
            Self::TrailingInput => "trailing_input",
            Self::NonCanonicalNumber => "non_canonical_number",
            Self::IntegerOverflow => "integer_overflow",
            Self::UnsupportedOperator => "unsupported_operator",
            Self::ProfileMismatch => "profile_mismatch",
            Self::UnknownSignal => "unknown_signal",
            Self::NonNumericSignal => "non_numeric_signal",
            Self::ScaleMismatch => "scale_mismatch",
            Self::ScaleOverflow => "scale_overflow",
            Self::RescaleNarrowing => "rescale_narrowing",
            Self::RangeOverflow => "range_overflow",
            Self::ReservedIdentifier => "reserved_identifier",
            Self::NonAssociativeComparison => "non_associative_comparison",
            Self::UnparenthesizedComparison => "unparenthesized_comparison",
            Self::UndefinedSet => "undefined_set",
            Self::DuplicateSet => "duplicate_set",
            Self::SlotOutsideTemplate => "slot_outside_template",
            Self::AggregationTooLarge => "aggregation_too_large",
            Self::DuplicateContract => "duplicate_contract",
            Self::MissingContractClause => "missing_contract_clause",
            Self::PropositionOverflow => "proposition_overflow",
        }
    }

    /// Resource codes outrank syntax and typing codes (the v4 priority rule).
    pub const fn refusal_priority(self) -> u8 {
        match self {
            Self::SourceLimit
            | Self::TokenLimit
            | Self::NodeLimit
            | Self::DepthLimit
            | Self::WorkLimit => 1,
            _ => 0,
        }
    }

    /// Whether this code is a resource-limit code.
    pub const fn is_resource_limit(self) -> bool {
        self.refusal_priority() == 1
    }
}

impl fmt::Display for NumericDiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One source-located v5 diagnostic. Only `code` and `span` are contractual.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NumericDiagnostic {
    /// Stable code from the closed set.
    pub code: NumericDiagnosticCode,
    /// Half-open UTF-8 byte span.
    pub span: SourceSpan,
    /// English prose for humans; not for programmatic matching.
    pub message: String,
}

/// Decisive refusal retained even when diagnostics are capped.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NumericRefusal {
    /// Stable typed reason.
    pub code: NumericDiagnosticCode,
    /// Byte locus of that reason.
    pub span: SourceSpan,
}

impl NumericRefusal {
    /// Keeps the first cause within a priority class and a later higher-priority cause.
    pub(crate) fn accumulate(
        current: Option<Self>,
        code: NumericDiagnosticCode,
        span: SourceSpan,
    ) -> Option<Self> {
        match current {
            Some(previous) if previous.code.refusal_priority() >= code.refusal_priority() => {
                Some(previous)
            }
            _ => Some(Self { code, span }),
        }
    }
}

/// Wire identity of a v5 parse report.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NumericParseSchemaVersion {
    /// Initial numeric report.
    #[serde(rename = "tl-parse.numeric-parse-report/v1")]
    V1,
}

/// Wire identity of the v5 diagnostics vocabulary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NumericDiagnosticsSchemaVersion {
    /// Initial closed vocabulary.
    #[serde(rename = "tl-parse.numeric-diagnostics/v1")]
    V1,
}

/// Closed identity of the numeric dialect.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum NumericDialectRevision {
    /// `tl-parse.clean-ascii/v5`.
    #[serde(rename = "tl-parse.clean-ascii/v5")]
    V5,
}

/// Structural echo of the supplied catalog; no digest (ADR-020 decision 2).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NumericCatalogEcho {
    /// Catalog schema identity.
    pub schema_version: String,
    /// Declared signals.
    pub signal_count: usize,
    /// Direct Boolean proposition bindings.
    pub binding_count: usize,
}

// SPEC-GAP: FR-028 names `tl-syntax.signal-catalog/v1`; tl-syntax now also
// admits v2 catalogs. v5 accepts any catalog tl-syntax validates, and the echo
// records which schema it was.
impl NumericCatalogEcho {
    pub(crate) fn of(catalog: &SignalCatalogDocument) -> Self {
        Self {
            schema_version: catalog.schema_version().as_str().to_owned(),
            signal_count: catalog.signals().len(),
            binding_count: catalog.bindings().len(),
        }
    }
}

/// Result of one bounded v5 parse attempt (`tl-parse.numeric-parse-report/v1`).
///
/// The report serializes through the owner encodings of its documents. It is
/// read back only by [`NumericParseReport::from_json_bytes`], which needs the
/// catalog: a predicate table is valid only against a catalog, so it is
/// re-validated by the tl-syntax constructor rather than trusted (FR-030-AC-3).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NumericParseReport {
    /// Report wire identity.
    pub schema_version: NumericParseSchemaVersion,
    /// Diagnostics vocabulary identity.
    pub diagnostics_schema: NumericDiagnosticsSchemaVersion,
    /// Selected dialect.
    pub dialect_revision: NumericDialectRevision,
    /// Requested semantic profile.
    pub semantic_profile: SemanticProfile,
    /// Effective limits after clamping.
    pub limits: ParseLimits,
    /// Deterministic counters; `nodes` counts formula nodes and the expression
    /// nodes of distinct (table) predicates.
    pub stats: ParseStats,
    /// Structural echo of the catalog.
    pub catalog: NumericCatalogEcho,
    /// Validated formula on success only.
    pub document: Option<FormulaDocument>,
    /// Validated predicate table on success only (absent when no comparison occurs).
    pub predicate_table: Option<PredicateTableDocument>,
    /// Source span of the first occurrence of each table entry, in table order.
    pub predicate_spans: Vec<SourceSpan>,
    /// At most `limits.max_diagnostics` diagnostics in source order.
    pub diagnostics: Vec<NumericDiagnostic>,
    /// Decisive refusal; present exactly when `document` is absent.
    pub refusal: Option<NumericRefusal>,
}

/// Report-local mirror of the owner predicate-table wire shape. It is never
/// trusted: [`PredicateTableDocument::new`] re-types it against the catalog.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PredicateTableMirror {
    schema_version: PredicateTableSchemaVersion,
    expressions: Vec<ExprNode>,
    predicates: Vec<Predicate>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NumericParseReportWire {
    schema_version: NumericParseSchemaVersion,
    diagnostics_schema: NumericDiagnosticsSchemaVersion,
    dialect_revision: NumericDialectRevision,
    semantic_profile: SemanticProfile,
    limits: ParseLimits,
    stats: ParseStats,
    catalog: NumericCatalogEcho,
    document: Option<FormulaDocument>,
    predicate_table: Option<PredicateTableMirror>,
    predicate_spans: Vec<SourceSpan>,
    diagnostics: Vec<NumericDiagnostic>,
    refusal: Option<NumericRefusal>,
}

impl NumericParseReportWire {
    fn admit(self, catalog: &SignalCatalogDocument) -> Result<NumericParseReport, String> {
        let wire = self;
        if wire.limits != wire.limits.clamped()
            || wire.stats.tokens > wire.limits.max_tokens
            || wire.stats.nodes > wire.limits.max_nodes
            || wire.stats.work > wire.limits.max_work
            || wire.stats.diagnostics != wire.diagnostics.len()
            || wire.stats.diagnostics > wire.limits.max_diagnostics
        {
            return Err("v5 report counters exceed effective limits".to_owned());
        }
        if wire.catalog != NumericCatalogEcho::of(catalog) {
            return Err("v5 report catalog echo differs from the supplied catalog".to_owned());
        }
        let bounded = |span: &SourceSpan| {
            usize::try_from(span.end()).is_ok_and(|end| end <= wire.stats.source_bytes)
        };
        if wire.predicate_spans.iter().any(|span| !bounded(span))
            || wire.diagnostics.iter().any(|item| !bounded(&item.span))
            || wire.refusal.is_some_and(|refusal| !bounded(&refusal.span))
        {
            return Err("v5 report span exceeds source bytes".to_owned());
        }
        if wire.document.is_some() == wire.refusal.is_some() {
            return Err("v5 refusal must occur exactly when the graph is absent".to_owned());
        }
        if wire.document.is_some() {
            if !wire.diagnostics.is_empty() || wire.stats.diagnostics_truncated {
                return Err("successful v5 report carries diagnostics".to_owned());
            }
            if !matches!(
                wire.semantic_profile,
                SemanticProfile::ClosedTraceV1 | SemanticProfile::OnlinePrefixV1
            ) {
                return Err("successful v5 report has a non-v5 profile".to_owned());
            }
        } else if wire.predicate_table.is_some() || !wire.predicate_spans.is_empty() {
            return Err("failed v5 report carries a predicate table".to_owned());
        }
        let predicate_table = match wire.predicate_table {
            None => None,
            Some(mirror) => {
                let _ = mirror.schema_version;
                let view = catalog.validate().map_err(|error| error.to_string())?;
                let table =
                    PredicateTableDocument::new(view, mirror.expressions, mirror.predicates)
                        .map_err(|error| error.to_string())?;
                Some(table)
            }
        };
        let entries = predicate_table
            .as_ref()
            .map_or(0, |table| table.predicates().len());
        if entries != wire.predicate_spans.len() {
            return Err("v5 predicate spans differ from the table".to_owned());
        }
        Ok(NumericParseReport {
            schema_version: wire.schema_version,
            diagnostics_schema: wire.diagnostics_schema,
            dialect_revision: wire.dialect_revision,
            semantic_profile: wire.semantic_profile,
            limits: wire.limits,
            stats: wire.stats,
            catalog: wire.catalog,
            document: wire.document,
            predicate_table,
            predicate_spans: wire.predicate_spans,
            diagnostics: wire.diagnostics,
            refusal: wire.refusal,
        })
    }
}

impl NumericParseReport {
    /// Strict-reads one canonical, bounded v5 report against `catalog`
    /// (FR-030-AC-3).
    ///
    /// Unknown fields, unknown codes, trailing data, non-canonical bytes,
    /// over-limit documents, a catalog echo that differs from `catalog`, and a
    /// predicate table that does not type against `catalog` are refused.
    pub fn from_json_bytes(
        bytes: &[u8],
        limits: ParseArtifactLimits,
        catalog: &SignalCatalogDocument,
    ) -> Result<Self, StrictParseArtifactReadError> {
        preflight_parse_artifact(bytes, limits)?;
        let wire: NumericParseReportWire =
            serde_json::from_slice(bytes).map_err(StrictParseArtifactReadError::InvalidDocument)?;
        let report = wire.admit(catalog).map_err(|message| {
            StrictParseArtifactReadError::InvalidDocument(serde::de::Error::custom(message))
        })?;
        let encoded =
            serde_json::to_vec(&report).map_err(StrictParseArtifactReadError::InvalidDocument)?;
        if encoded != bytes {
            return Err(StrictParseArtifactReadError::NonCanonicalDocument);
        }
        Ok(report)
    }

    /// Serializes this report as compact, stable-key-order JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub(crate) fn empty(
        profile: SemanticProfile,
        catalog: &SignalCatalogDocument,
        limits: ParseLimits,
        source_bytes: usize,
    ) -> Self {
        Self {
            schema_version: NumericParseSchemaVersion::V1,
            diagnostics_schema: NumericDiagnosticsSchemaVersion::V1,
            dialect_revision: NumericDialectRevision::V5,
            semantic_profile: profile,
            limits,
            stats: ParseStats {
                source_bytes,
                ..ParseStats::default()
            },
            catalog: NumericCatalogEcho::of(catalog),
            document: None,
            predicate_table: None,
            predicate_spans: Vec::new(),
            diagnostics: Vec::new(),
            refusal: None,
        }
    }
}

/// Bounded diagnostic sink shared by the v5 lexer and parser.
#[derive(Default)]
pub(crate) struct Sink {
    pub(crate) diagnostics: Vec<NumericDiagnostic>,
    pub(crate) truncated: bool,
    pub(crate) refusal: Option<NumericRefusal>,
    pub(crate) max: usize,
}

impl Sink {
    pub(crate) fn new(max: usize) -> Self {
        Self {
            max,
            ..Self::default()
        }
    }

    pub(crate) fn push(
        &mut self,
        code: NumericDiagnosticCode,
        start: usize,
        end: usize,
        message: String,
    ) {
        let span = checked_span(start, end);
        self.refusal = NumericRefusal::accumulate(self.refusal, code, span);
        if self.diagnostics.len() >= self.max {
            self.truncated = true;
            return;
        }
        self.diagnostics.push(NumericDiagnostic {
            code,
            span,
            message,
        });
    }

    pub(crate) fn has_error(&self) -> bool {
        self.refusal.is_some()
    }
}

/// Parses explicitly selected `tl-parse.clean-ascii/v5` text against a catalog.
///
/// The catalog is a required argument, so a v5 request without a catalog is
/// unrepresentable. A profile other than `mltl.closed-trace/v1` or
/// `mltl.online-prefix/v1` is refused before lexing as `profile_mismatch`.
/// Any diagnostic suppresses the graph and the table.
pub fn parse_clean_ascii_v5(
    source: &str,
    catalog: &SignalCatalogDocument,
    profile: SemanticProfile,
    limits: ParseLimits,
) -> NumericParseReport {
    parser::parse(source, catalog, profile, limits)
}
