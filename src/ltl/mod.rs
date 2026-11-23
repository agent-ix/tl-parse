//! `tl-parse.clean-ascii/v6`: bracket-free LTL and LTLf (TL-429).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL program decision
//! 2026-11-23). Every acceptance criterion is listed in the deferred
//! verification table of PLAN-045.
//!
//! The specification (tl-parse FR-058..FR-062, ADR-041) names this dialect
//! `v5`; program decision 2 renumbered it to `tl-parse.clean-ascii/v6` because
//! the numeric dialect (TL-421) keeps `v5`.
//!
//! The semantic profile is always the caller's [`LtlProfile`] argument. There is
//! no default, no `Option`, no inference from spelling, and no profile token in
//! the source (FR-059). `LtlProfile` deliberately implements neither `Default`
//! nor `From<&str>`/`FromStr`.

mod format;
mod model;
mod parser;

use std::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{InfiniteFormulaDocument, SourceSpan};

use crate::{
    diagnostic::preflight_parse_artifact, lexer::checked_span, ParseArtifactLimits, ParseLimits,
    ParseStats, StrictParseArtifactReadError,
};

pub use format::{format_clean_ascii_v6, LtlFormatError, LtlFormatErrorCode, LtlFormatReport};
// UNBUILT-PIN: `LtlfFormulaDocument` is the tl-syntax FR-332/FR-333 edition
// `tl-syntax.formula-ltlf/v1` (TL-428), written concurrently in
// TL-code-tl-syntax and absent from the pinned rev.
pub use model::LtlfFormulaDocument;

/// Stable identity of the bracket-free LTL dialect.
pub const DIALECT_V6_REVISION: &str = "tl-parse.clean-ascii/v6";

/// Stable identity of serialized v6 parse reports.
pub const LTL_PARSE_REPORT_SCHEMA_VERSION: &str = "tl-parse.ltl-parse-report/v1";

/// Normative v6 dialect document.
pub const DIALECT_V6_DOCUMENT: &str = include_str!("../../docs/DIALECT-006-clean-ascii-v6.md");

/// The two semantic profiles of the v6 dialect. Always chosen by the caller.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LtlProfile {
    /// `mltl.ltlf-finite-trace/v1`: nonempty finite traces, `Xs` and `Xw`.
    #[serde(rename = "mltl.ltlf-finite-trace/v1")]
    LtlfFiniteTrace,
    /// `mltl.infinite-trace/v1`: `F p` is `F[0,) p`, `X p` is `F[1,1] p`.
    #[serde(rename = "mltl.infinite-trace/v1")]
    InfiniteTrace,
}

impl LtlProfile {
    /// Both profiles, in declaration order.
    pub const ALL: [Self; 2] = [Self::LtlfFiniteTrace, Self::InfiniteTrace];

    /// Stable profile identity string.
    pub const fn identity(self) -> &'static str {
        match self {
            Self::LtlfFiniteTrace => "mltl.ltlf-finite-trace/v1",
            Self::InfiniteTrace => "mltl.infinite-trace/v1",
        }
    }
}

/// Closed v6 diagnostic code set.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LtlDiagnosticCode {
    /// Source exceeds the effective byte limit.
    SourceLimit,
    /// Token count exceeds the effective token limit.
    TokenLimit,
    /// Node count exceeds the effective node limit.
    NodeLimit,
    /// Nesting exceeds the effective depth limit.
    DepthLimit,
    /// Parser work exceeds the effective work limit.
    WorkLimit,
    /// A byte has no meaning in the dialect (non-ASCII, NUL, punctuation).
    UnexpectedCharacter,
    /// A word that is no v6 keyword or atom (for example `Xx`).
    UnknownIdentifier,
    /// An atom with a leading zero.
    NonCanonicalNumber,
    /// An atom beyond `u32`.
    IntegerOverflow,
    /// A token not valid at its position (including empty input).
    UnexpectedToken,
    /// A required token is absent.
    MissingToken,
    /// Input remains after a complete formula.
    TrailingInput,
    /// `W M O H Y S T`: recognized operators outside v6.
    UnsupportedOperator,
    /// An interval bracket; intervals are spelled in dialects v1 to v4.
    IntervalNotInDialect,
    /// Bare `X` under the LTLf profile: write `Xs` or `Xw`.
    NextStrengthRequired,
    /// `Xs` or `Xw` under the infinite profile, which has no end of trace.
    NextStrengthUndefined,
    /// A profile identity that is not a v6 profile (identity-string entry point).
    ProfileMismatch,
    /// The owner graph constructor refused an otherwise parsed graph.
    ValidationFailure,
}

impl LtlDiagnosticCode {
    /// Every code, in declaration order.
    pub const ALL: [Self; 18] = [
        Self::SourceLimit,
        Self::TokenLimit,
        Self::NodeLimit,
        Self::DepthLimit,
        Self::WorkLimit,
        Self::UnexpectedCharacter,
        Self::UnknownIdentifier,
        Self::NonCanonicalNumber,
        Self::IntegerOverflow,
        Self::UnexpectedToken,
        Self::MissingToken,
        Self::TrailingInput,
        Self::UnsupportedOperator,
        Self::IntervalNotInDialect,
        Self::NextStrengthRequired,
        Self::NextStrengthUndefined,
        Self::ProfileMismatch,
        Self::ValidationFailure,
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
            Self::UnknownIdentifier => "unknown_identifier",
            Self::NonCanonicalNumber => "non_canonical_number",
            Self::IntegerOverflow => "integer_overflow",
            Self::UnexpectedToken => "unexpected_token",
            Self::MissingToken => "missing_token",
            Self::TrailingInput => "trailing_input",
            Self::UnsupportedOperator => "unsupported_operator",
            Self::IntervalNotInDialect => "interval_not_in_dialect",
            Self::NextStrengthRequired => "next_strength_required",
            Self::NextStrengthUndefined => "next_strength_undefined",
            Self::ProfileMismatch => "profile_mismatch",
            Self::ValidationFailure => "validation_failure",
        }
    }

    /// Resource codes outrank syntax codes; an owner failure outranks both.
    pub const fn refusal_priority(self) -> u8 {
        match self {
            Self::ValidationFailure => 2,
            Self::SourceLimit
            | Self::TokenLimit
            | Self::NodeLimit
            | Self::DepthLimit
            | Self::WorkLimit => 1,
            _ => 0,
        }
    }
}

impl fmt::Display for LtlDiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One located v6 diagnostic. Only `code` and `span` are contractual.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LtlDiagnostic {
    /// Stable code.
    pub code: LtlDiagnosticCode,
    /// Half-open UTF-8 byte span.
    pub span: SourceSpan,
    /// English prose for humans.
    pub message: String,
}

/// Decisive refusal, retained even when diagnostics are capped.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LtlRefusal {
    /// Stable code.
    pub code: LtlDiagnosticCode,
    /// Byte locus.
    pub span: SourceSpan,
}

/// Wire identity of a v6 parse report.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LtlParseSchemaVersion {
    /// Initial report.
    #[serde(rename = "tl-parse.ltl-parse-report/v1")]
    V1,
}

/// Closed identity of the v6 dialect.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum LtlDialectRevision {
    /// `tl-parse.clean-ascii/v6`.
    #[serde(rename = "tl-parse.clean-ascii/v6")]
    V6,
}

/// The owner graph of one successful parse; its edition follows the profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LtlDocument {
    /// `tl-syntax.formula-ltlf/v1` under `mltl.ltlf-finite-trace/v1`.
    Ltlf(LtlfFormulaDocument),
    /// `tl-syntax.formula-unbounded/v1` under `mltl.infinite-trace/v1`.
    Infinite(InfiniteFormulaDocument),
}

impl LtlDocument {
    /// The profile this document's edition belongs to.
    pub const fn profile(&self) -> LtlProfile {
        match self {
            Self::Ltlf(_) => LtlProfile::LtlfFiniteTrace,
            Self::Infinite(_) => LtlProfile::InfiniteTrace,
        }
    }
}

/// Result of one bounded v6 parse (`tl-parse.ltl-parse-report/v1`).
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "LtlParseReportWire")]
pub struct LtlParseReport {
    /// Report identity.
    pub schema_version: LtlParseSchemaVersion,
    /// Dialect identity.
    pub dialect_revision: LtlDialectRevision,
    /// The caller-selected profile.
    pub profile: LtlProfile,
    /// Effective limits after clamping.
    pub limits: ParseLimits,
    /// Deterministic counters.
    pub stats: ParseStats,
    /// Owner graph on success only.
    pub document: Option<LtlDocument>,
    /// At most `limits.max_diagnostics`, in source order.
    pub diagnostics: Vec<LtlDiagnostic>,
    /// Present exactly when `document` is absent.
    pub refusal: Option<LtlRefusal>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LtlParseReportWire {
    schema_version: LtlParseSchemaVersion,
    dialect_revision: LtlDialectRevision,
    profile: LtlProfile,
    limits: ParseLimits,
    stats: ParseStats,
    document: Option<LtlDocument>,
    diagnostics: Vec<LtlDiagnostic>,
    refusal: Option<LtlRefusal>,
}

impl TryFrom<LtlParseReportWire> for LtlParseReport {
    type Error = &'static str;

    fn try_from(wire: LtlParseReportWire) -> Result<Self, Self::Error> {
        if wire.limits != wire.limits.clamped()
            || wire.stats.tokens > wire.limits.max_tokens
            || wire.stats.nodes > wire.limits.max_nodes
            || wire.stats.work > wire.limits.max_work
            || wire.stats.diagnostics != wire.diagnostics.len()
            || wire.stats.diagnostics > wire.limits.max_diagnostics
        {
            return Err("v6 report counters exceed effective limits");
        }
        let bounded = |span: &SourceSpan| {
            usize::try_from(span.end()).is_ok_and(|end| end <= wire.stats.source_bytes)
        };
        if wire.diagnostics.iter().any(|item| !bounded(&item.span))
            || wire.refusal.is_some_and(|refusal| !bounded(&refusal.span))
        {
            return Err("v6 report span exceeds source bytes");
        }
        if wire.document.is_some() == wire.refusal.is_some() {
            return Err("v6 refusal must occur exactly when the graph is absent");
        }
        if let Some(document) = wire.document.as_ref() {
            if document.profile() != wire.profile
                || !wire.diagnostics.is_empty()
                || wire.stats.diagnostics_truncated
            {
                return Err("successful v6 report has a conflicting profile or diagnostics");
            }
        }
        Ok(Self {
            schema_version: wire.schema_version,
            dialect_revision: wire.dialect_revision,
            profile: wire.profile,
            limits: wire.limits,
            stats: wire.stats,
            document: wire.document,
            diagnostics: wire.diagnostics,
            refusal: wire.refusal,
        })
    }
}

impl LtlParseReport {
    /// Strict-reads one canonical, bounded v6 report.
    pub fn from_json_bytes(
        bytes: &[u8],
        limits: ParseArtifactLimits,
    ) -> Result<Self, StrictParseArtifactReadError> {
        preflight_parse_artifact(bytes, limits)?;
        let report: Self =
            serde_json::from_slice(bytes).map_err(StrictParseArtifactReadError::InvalidDocument)?;
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
}

/// Bounded diagnostic sink for the v6 lexer and parser.
pub(crate) struct Sink {
    pub(crate) diagnostics: Vec<LtlDiagnostic>,
    pub(crate) truncated: bool,
    pub(crate) refusal: Option<LtlRefusal>,
    max: usize,
}

impl Sink {
    pub(crate) fn new(max: usize) -> Self {
        Self {
            diagnostics: Vec::new(),
            truncated: false,
            refusal: None,
            max,
        }
    }

    pub(crate) fn push(&mut self, code: LtlDiagnosticCode, start: usize, end: usize, message: String) {
        let span = checked_span(start, end);
        self.refusal = match self.refusal {
            Some(previous) if previous.code.refusal_priority() >= code.refusal_priority() => {
                Some(previous)
            }
            _ => Some(LtlRefusal { code, span }),
        };
        if self.diagnostics.len() >= self.max {
            self.truncated = true;
            return;
        }
        self.diagnostics.push(LtlDiagnostic {
            code,
            span,
            message,
        });
    }

    pub(crate) fn has_error(&self) -> bool {
        self.refusal.is_some()
    }
}

/// Parses `tl-parse.clean-ascii/v6` under the caller's profile.
///
/// Under `LtlfFiniteTrace` the graph is `tl-syntax.formula-ltlf/v1`; under
/// `InfiniteTrace` it is `tl-syntax.formula-unbounded/v1`, where `F p` is
/// `F[0,) p` (likewise `G`, `U`, `R`) and `X p` is `F[1,1] p`.
pub fn parse_clean_ascii_v6(source: &str, profile: LtlProfile, limits: ParseLimits) -> LtlParseReport {
    parser::parse(source, profile, limits)
}

/// Generic selection by profile identity string (FR-059): a string that is not
/// one of the two v6 profile identities is refused before lexing with
/// `profile_mismatch`. The identity is the caller's explicit choice; it is
/// never read from the source text.
pub fn parse_clean_ascii_v6_with_profile_identity(
    source: &str,
    profile_identity: &str,
    limits: ParseLimits,
) -> Result<LtlParseReport, LtlDiagnostic> {
    let profile = LtlProfile::ALL
        .into_iter()
        .find(|profile| profile.identity() == profile_identity);
    match profile {
        Some(profile) => Ok(parse_clean_ascii_v6(source, profile, limits)),
        None => Err(LtlDiagnostic {
            code: LtlDiagnosticCode::ProfileMismatch,
            span: checked_span(0, 0),
            message: format!("{profile_identity:?} is not a tl-parse.clean-ascii/v6 profile"),
        }),
    }
}
