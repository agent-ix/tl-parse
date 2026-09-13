//! The explicitly selected `tl-parse.clean-ascii/v2` derived-operator dialect.

use serde::{Deserialize, Serialize};
use tl_syntax::{FormulaDocument, FutureKind, NodeId, SemanticProfile, SourceSpan};

use crate::{
    lexer::Dialect, parser::parse_dialect, Diagnostic, ParseLimits, ParseStats, TL_SYNTAX_REVISION,
};

/// Strict wire identity for a derived-operator parse report.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DerivedParseSchemaVersion {
    /// Initial derived-operator parse report wire format.
    #[serde(rename = "tl-parse.derived-parse-report/v1")]
    V1,
}

/// Strict identity of the dialect a derived report was parsed under.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DerivedDialectRevision {
    /// The v1 dialect plus bounded `W` and `M`.
    #[serde(rename = "tl-parse.clean-ascii/v2")]
    V2,
}

/// Strict identity of the tl-syntax operator profile used for lowering.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DerivedOperatorProfile {
    /// `tl-syntax.future-operators/v1`.
    #[serde(rename = "tl-syntax.future-operators/v1")]
    FutureOperatorsV1,
}

/// A derived operator spelling accepted by v2.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DerivedOperator {
    /// Bounded weak until, `W[a,b]`.
    #[serde(rename = "W")]
    WeakUntil,
    /// Bounded strong release, `M[a,b]`.
    #[serde(rename = "M")]
    StrongRelease,
}

impl From<FutureKind> for DerivedOperator {
    fn from(kind: FutureKind) -> Self {
        match kind {
            FutureKind::WeakUntil => Self::WeakUntil,
            FutureKind::StrongRelease => Self::StrongRelease,
        }
    }
}

/// One derived expression and the primitive nodes it lowered to.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoweringRecord {
    /// Derived operator spelling.
    pub kind: DerivedOperator,
    /// Keyword start through the interval's closing bracket.
    pub operator_span: SourceSpan,
    /// Left operand extent start through right operand extent end.
    pub expression_span: SourceSpan,
    /// Left operand node.
    pub left: NodeId,
    /// Right operand node.
    pub right: NodeId,
    /// First of the three generated primitive nodes.
    pub first_generated: NodeId,
    /// Root of the generated expansion.
    pub root: NodeId,
}

/// Complete versioned result of one v2 parse attempt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "DerivedParseReportWire")]
pub struct DerivedParseReport {
    /// Closed wire identity for this report family.
    pub schema_version: DerivedParseSchemaVersion,
    /// Closed dialect identity.
    pub dialect_revision: DerivedDialectRevision,
    /// Closed tl-syntax operator-profile identity.
    pub operator_profile: DerivedOperatorProfile,
    /// Exact compiled tl-syntax source revision.
    pub tl_syntax_revision: String,
    /// Requested and preserved semantic profile.
    pub semantic_profile: SemanticProfile,
    /// Effective limits after process-safe clamping.
    pub limits: ParseLimits,
    /// Deterministic observed counts.
    pub stats: ParseStats,
    /// Present only on a diagnostic-free, validated parse.
    pub document: Option<FormulaDocument>,
    /// Lowering records in lowering order, present only with the document.
    pub lowerings: Vec<LoweringRecord>,
    /// Stable diagnostics in encounter order.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DerivedParseReportWire {
    schema_version: DerivedParseSchemaVersion,
    dialect_revision: DerivedDialectRevision,
    operator_profile: DerivedOperatorProfile,
    tl_syntax_revision: String,
    semantic_profile: SemanticProfile,
    limits: ParseLimits,
    stats: ParseStats,
    document: Option<FormulaDocument>,
    lowerings: Vec<LoweringRecord>,
    diagnostics: Vec<Diagnostic>,
}

impl TryFrom<DerivedParseReportWire> for DerivedParseReport {
    type Error = &'static str;

    fn try_from(wire: DerivedParseReportWire) -> Result<Self, Self::Error> {
        if wire.document.is_none() && !wire.lowerings.is_empty() {
            return Err("lowering records require a document");
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
            lowerings: wire.lowerings,
            diagnostics: wire.diagnostics,
        })
    }
}

/// Parses one source string under the explicitly selected v2 dialect.
///
/// Each `W`/`M` expression lowers through tl-syntax into primitive nodes. Any
/// diagnostic suppresses both the document and the lowering records.
pub fn parse_clean_ascii_v2(
    source: &str,
    profile: SemanticProfile,
    limits: ParseLimits,
) -> DerivedParseReport {
    let parsed = parse_dialect(source, profile, limits, Dialect::V2);
    let report = parsed.report;
    DerivedParseReport {
        schema_version: DerivedParseSchemaVersion::V1,
        dialect_revision: DerivedDialectRevision::V2,
        operator_profile: DerivedOperatorProfile::FutureOperatorsV1,
        tl_syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        semantic_profile: report.semantic_profile,
        limits: report.limits,
        stats: report.stats,
        document: report.document,
        lowerings: parsed.lowerings,
        diagnostics: report.diagnostics,
    }
}
