//! Explicit clean-ascii/v3 parsing for the origin-complete past profile.

use serde::{Deserialize, Serialize};
use tl_syntax::{
    FormulaDocument, FormulaSchemaVersion, SemanticProfile, SourceSpan, SyntaxArtifactLimits,
};

use crate::{
    diagnostic::preflight_parse_artifact, lexer::TokenKind, parser::parse_dialect, Diagnostic,
    ParseArtifactLimits, ParseLimits, ParseStats, StrictParseArtifactReadError, TL_SYNTAX_REVISION,
};

use super::{BinarySpelling, Dialect, UnarySpelling};

const UNSUPPORTED_OPERATORS: [&str; 7] = ["X", "F", "G", "U", "R", "W", "M"];

pub(crate) fn classify_identifier(lexeme: &str) -> Option<TokenKind> {
    match lexeme {
        "false" => Some(TokenKind::False),
        "true" => Some(TokenKind::True),
        "O" => Some(TokenKind::Once),
        "H" => Some(TokenKind::Historically),
        "S" => Some(TokenKind::Since),
        "T" => Some(TokenKind::Triggered),
        _ => None,
    }
}

pub(crate) fn unsupported_operator_profile(lexeme: &str) -> Option<&'static str> {
    UNSUPPORTED_OPERATORS
        .contains(&lexeme)
        .then_some("tl-syntax.past-operators/v1")
}

pub(crate) const fn atom_keyword_boundary(next: u8, followed_by_bracket: bool) -> bool {
    matches!(next, b'S' | b'T') && followed_by_bracket
}

pub(crate) const fn binary_binding_power(token: TokenKind) -> Option<(u8, u8)> {
    match token {
        TokenKind::Equivalent => Some((1, 2)),
        TokenKind::Implies => Some((2, 2)),
        TokenKind::Or => Some((3, 4)),
        TokenKind::And => Some((4, 5)),
        TokenKind::Since | TokenKind::Triggered => Some((5, 6)),
        _ => None,
    }
}

pub(crate) const fn permits_temporal_prefix(token: TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Once | TokenKind::Historically | TokenKind::StrongPrevious
    )
}

pub(crate) fn build_document(
    profile: SemanticProfile,
    root: tl_syntax::NodeId,
    nodes: Vec<tl_syntax::Node>,
) -> Result<FormulaDocument, tl_syntax::FormulaError> {
    FormulaDocument::new_v2(profile, root, nodes)
}

pub(crate) const fn unary_spelling(kind: tl_syntax::NodeKind) -> Option<UnarySpelling> {
    match kind {
        tl_syntax::NodeKind::Not { .. } => Some(UnarySpelling {
            operator: "!",
            interval: None,
        }),
        tl_syntax::NodeKind::Once { interval, .. } => Some(UnarySpelling {
            operator: "O",
            interval: Some(interval),
        }),
        tl_syntax::NodeKind::Historically { interval, .. } => Some(UnarySpelling {
            operator: "H",
            interval: Some(interval),
        }),
        tl_syntax::NodeKind::StrongPrevious { .. } => Some(UnarySpelling {
            operator: "Y",
            interval: None,
        }),
        _ => None,
    }
}

pub(crate) const fn binary_spelling(kind: tl_syntax::NodeKind) -> Option<BinarySpelling> {
    let (operator, interval, precedence, right_associative) = match kind {
        tl_syntax::NodeKind::And { .. } => ("&", None, 4, false),
        tl_syntax::NodeKind::Or { .. } => ("|", None, 3, false),
        tl_syntax::NodeKind::Implies { .. } => ("->", None, 2, true),
        tl_syntax::NodeKind::Equivalent { .. } => ("<->", None, 1, false),
        tl_syntax::NodeKind::Since { interval, .. } => ("S", Some(interval), 5, false),
        tl_syntax::NodeKind::Triggered { interval, .. } => ("T", Some(interval), 5, false),
        _ => return None,
    };
    Some(BinarySpelling {
        operator,
        interval,
        precedence,
        right_associative,
    })
}

/// Strict wire identity for a clean-ascii/v3 parse report.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PastParseSchemaVersion {
    /// Initial past-profile parse report wire format.
    #[serde(rename = "tl-parse.past-parse-report/v1")]
    V1,
}

/// Strict identity of the past-profile text dialect.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PastDialectRevision {
    /// Boolean plus O/H/Y/S/T with no future-time operator spelling.
    #[serde(rename = "tl-parse.clean-ascii/v3")]
    V3,
}

/// Strict identity of the tl-syntax past operator catalog.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PastOperatorProfile {
    /// `tl-syntax.past-operators/v1`.
    #[serde(rename = "tl-syntax.past-operators/v1")]
    PastOperatorsV1,
}

/// Complete versioned result of one clean-ascii/v3 parse attempt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "PastParseReportWire")]
pub struct PastParseReport {
    /// Closed wire identity for this report family.
    pub schema_version: PastParseSchemaVersion,
    /// Closed text-dialect identity.
    pub dialect_revision: PastDialectRevision,
    /// Closed tl-syntax operator-profile identity.
    pub operator_profile: PastOperatorProfile,
    /// Exact compiled tl-syntax source revision.
    pub tl_syntax_revision: String,
    /// Fixed origin-complete history profile.
    pub semantic_profile: SemanticProfile,
    /// Effective limits after process-safe clamping.
    pub limits: ParseLimits,
    /// Deterministic observed counts.
    pub stats: ParseStats,
    /// Present only on a diagnostic-free, validated parse.
    pub document: Option<FormulaDocument>,
    /// Stable diagnostics in encounter order.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PastParseReportWire {
    schema_version: PastParseSchemaVersion,
    dialect_revision: PastDialectRevision,
    operator_profile: PastOperatorProfile,
    tl_syntax_revision: String,
    semantic_profile: SemanticProfile,
    limits: ParseLimits,
    stats: ParseStats,
    document: Option<FormulaDocument>,
    diagnostics: Vec<Diagnostic>,
}

impl TryFrom<PastParseReportWire> for PastParseReport {
    type Error = &'static str;

    fn try_from(wire: PastParseReportWire) -> Result<Self, Self::Error> {
        if wire.tl_syntax_revision != TL_SYNTAX_REVISION {
            return Err("unexpected tl-syntax revision");
        }
        if wire.limits != wire.limits.clamped() {
            return Err("parse limits exceed the process-safe maximum");
        }
        if wire.semantic_profile != SemanticProfile::OriginCompleteHistoryV1 {
            return Err("clean-ascii/v3 requires the origin-complete history profile");
        }
        if wire.stats.diagnostics != wire.diagnostics.len() {
            return Err("diagnostic count does not match retained diagnostics");
        }
        if wire.stats.tokens > wire.limits.max_tokens
            || wire.stats.nodes > wire.limits.max_nodes
            || wire.stats.diagnostics > wire.limits.max_diagnostics
            || wire.stats.work > wire.limits.max_work
            || wire.stats.max_depth > wire.limits.max_depth.saturating_add(1)
        {
            return Err("parse statistics exceed the effective limits");
        }
        if wire
            .diagnostics
            .iter()
            .any(|diagnostic| !span_is_source_bounded(diagnostic.span, wire.stats.source_bytes))
        {
            return Err("diagnostic span exceeds the submitted source");
        }
        if wire.document.is_none()
            && wire.diagnostics.is_empty()
            && !wire.stats.diagnostics_truncated
        {
            return Err("a failed parse requires a retained or truncated diagnostic");
        }
        if let Some(document) = wire.document.as_ref() {
            if !wire.diagnostics.is_empty() || wire.stats.diagnostics_truncated {
                return Err("a document requires a diagnostic-free parse");
            }
            if wire.stats.source_bytes > wire.limits.max_source_bytes
                || wire.stats.max_depth > wire.limits.max_depth
            {
                return Err("a successful parse exceeds its effective source/depth limits");
            }
            if document.schema_version() != FormulaSchemaVersion::V2
                || document.semantic_profile() != SemanticProfile::OriginCompleteHistoryV1
            {
                return Err("clean-ascii/v3 documents require formula-v2 and the past profile");
            }
            if document.nodes().len() != wire.stats.nodes || document.validate().is_err() {
                return Err("document graph does not match the parse report");
            }
            let owner_bytes = document
                .canonical_json_bytes()
                .map_err(|_| "document cannot be encoded as canonical owner JSON")?;
            let admitted =
                FormulaDocument::from_json_bytes(&owner_bytes, SyntaxArtifactLimits::default())
                    .map_err(|_| "document fails the pinned tl-syntax strict reader")?;
            if admitted != *document {
                return Err("strictly admitted document differs from the parse report");
            }
            if document.nodes().iter().any(|node| {
                node.span
                    .is_none_or(|span| !span_is_source_bounded(span, wire.stats.source_bytes))
            }) {
                return Err("document nodes require source-bounded spans");
            }
        }
        Ok(Self {
            schema_version: wire.schema_version,
            dialect_revision: wire.dialect_revision,
            operator_profile: wire.operator_profile,
            tl_syntax_revision: wire.tl_syntax_revision,
            semantic_profile: wire.semantic_profile,
            limits: wire.limits,
            stats: wire.stats,
            document: wire.document,
            diagnostics: wire.diagnostics,
        })
    }
}

impl PastParseReport {
    /// Strict-reads one complete canonical past-parse report under caller-lowered limits.
    ///
    /// Unknown or duplicate fields, trailing bytes, noncanonical JSON, identity drift,
    /// out-of-bounds spans/counters, and invalid embedded owner documents are refused.
    pub fn from_json_bytes(
        bytes: &[u8],
        limits: ParseArtifactLimits,
    ) -> Result<Self, StrictParseArtifactReadError> {
        preflight_parse_artifact(bytes, limits)?;
        let report: Self =
            serde_json::from_slice(bytes).map_err(StrictParseArtifactReadError::InvalidDocument)?;
        let canonical =
            serde_json::to_vec(&report).map_err(StrictParseArtifactReadError::InvalidDocument)?;
        if canonical != bytes {
            return Err(StrictParseArtifactReadError::NonCanonicalDocument);
        }
        Ok(report)
    }

    /// Returns the one canonical compact JSON encoding of this report.
    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

fn span_is_source_bounded(span: SourceSpan, source_bytes: usize) -> bool {
    match usize::try_from(span.end()) {
        Ok(end) => end <= source_bytes,
        Err(_) => false,
    }
}

/// Parses one source string under clean-ascii/v3 and the fixed past profile.
///
/// The entry point cannot be asked to parse under a future profile. Any
/// diagnostic suppresses the formula document.
pub fn parse_clean_ascii_v3(source: &str, limits: ParseLimits) -> PastParseReport {
    let parsed = parse_dialect(
        source,
        SemanticProfile::OriginCompleteHistoryV1,
        limits,
        Dialect::V3,
    );
    let report = parsed.report;
    PastParseReport {
        schema_version: PastParseSchemaVersion::V1,
        dialect_revision: PastDialectRevision::V3,
        operator_profile: PastOperatorProfile::PastOperatorsV1,
        tl_syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        semantic_profile: report.semantic_profile,
        limits: report.limits,
        stats: report.stats,
        document: report.document,
        diagnostics: report.diagnostics,
    }
}
