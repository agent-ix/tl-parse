use std::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{FormulaDocument, SemanticProfile, SourceSpan};

use crate::{DIAGNOSTIC_SCHEMA_VERSION, DIALECT_REVISION, TL_SYNTAX_REVISION};

/// Hard maximum source bytes processed by one request.
pub const HARD_MAX_SOURCE_BYTES: usize = 1_048_576;
/// Hard maximum non-EOF tokens processed by one request.
pub const HARD_MAX_TOKENS: usize = 100_000;
/// Hard maximum formula nodes produced by one request.
pub const HARD_MAX_NODES: usize = 10_000;
/// Hard maximum recursive parser nesting.
pub const HARD_MAX_DEPTH: usize = 256;
/// Hard maximum retained diagnostics.
pub const HARD_MAX_DIAGNOSTICS: usize = 64;
/// Hard maximum deterministic parser work units.
pub const HARD_MAX_PARSE_WORK: usize = 1_000_000;
/// Hard maximum canonical output bytes.
pub const HARD_MAX_OUTPUT_BYTES: usize = 1_048_576;
/// Hard maximum deterministic formatter work units.
pub const HARD_MAX_FORMAT_WORK: usize = 4_194_304;

/// Caller-selectable parse limits, clamped to process-safe hard maxima.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParseLimits {
    /// Maximum source bytes examined.
    pub max_source_bytes: usize,
    /// Maximum non-EOF tokens emitted.
    pub max_tokens: usize,
    /// Maximum graph nodes emitted.
    pub max_nodes: usize,
    /// Maximum expression nesting.
    pub max_depth: usize,
    /// Maximum diagnostics retained.
    pub max_diagnostics: usize,
    /// Maximum deterministic parser work units.
    pub max_work: usize,
}

impl ParseLimits {
    pub(crate) fn clamped(self) -> Self {
        Self {
            max_source_bytes: self.max_source_bytes.min(HARD_MAX_SOURCE_BYTES),
            max_tokens: self.max_tokens.min(HARD_MAX_TOKENS),
            max_nodes: self.max_nodes.min(HARD_MAX_NODES),
            max_depth: self.max_depth.min(HARD_MAX_DEPTH),
            max_diagnostics: self.max_diagnostics.min(HARD_MAX_DIAGNOSTICS),
            max_work: self.max_work.min(HARD_MAX_PARSE_WORK),
        }
    }
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: HARD_MAX_SOURCE_BYTES,
            max_tokens: HARD_MAX_TOKENS,
            max_nodes: HARD_MAX_NODES,
            max_depth: HARD_MAX_DEPTH,
            max_diagnostics: 32,
            max_work: HARD_MAX_PARSE_WORK,
        }
    }
}

/// Deterministic observations retained for every parse attempt.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParseStats {
    /// Bytes in the submitted source, including bytes beyond a source limit.
    pub source_bytes: usize,
    /// Non-EOF tokens emitted.
    pub tokens: usize,
    /// Nodes appended before success or failure.
    pub nodes: usize,
    /// Diagnostics retained in the report.
    pub diagnostics: usize,
    /// Deterministic parser work units consumed.
    pub work: usize,
    /// Maximum parser nesting observed.
    pub max_depth: usize,
    /// Whether diagnostics were suppressed by the diagnostic cap.
    pub diagnostics_truncated: bool,
}

/// Stable error codes for lexical, syntactic, validation, and limit failures.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    /// Source exceeds the effective byte limit.
    SourceLimit,
    /// Lexer token count exceeds the effective token limit.
    TokenLimit,
    /// Parser node count exceeds the effective node limit.
    NodeLimit,
    /// Parser nesting exceeds the effective depth limit.
    DepthLimit,
    /// Parser work exceeds the effective logical-work limit.
    WorkLimit,
    /// A character has no meaning in the dialect.
    UnexpectedCharacter,
    /// An ASCII identifier is not in the closed vocabulary.
    UnknownIdentifier,
    /// A decimal representation has a forbidden leading zero.
    NonCanonicalNumber,
    /// A decimal value exceeds u32.
    IntegerOverflow,
    /// A token is not valid in the current grammar position.
    UnexpectedToken,
    /// A required delimiter or operand is absent.
    MissingToken,
    /// An inclusive interval has start greater than end.
    InvalidInterval,
    /// Input remains after a complete expression.
    TrailingInput,
    /// Directly constructed graph failed the pinned tl-syntax validator.
    ValidationFailure,
}

impl DiagnosticCode {
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
            Self::InvalidInterval => "invalid_interval",
            Self::TrailingInput => "trailing_input",
            Self::ValidationFailure => "validation_failure",
        }
    }
}

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Closed diagnostic severity set for v0.1.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    /// The report cannot contain a successful formula document.
    Error,
}

/// Stable expected-token categories.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedToken {
    /// Any valid expression prefix.
    Expression,
    /// A canonical unsigned decimal integer.
    Integer,
    /// Opening interval bracket.
    LeftBracket,
    /// Closing interval bracket.
    RightBracket,
    /// Closing grouping parenthesis.
    RightParenthesis,
    /// Interval comma.
    Comma,
    /// End of source.
    EndOfInput,
}

/// Deterministic recovery action associated with a diagnostic.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryAction {
    /// No token stream change was required.
    None,
    /// The offending token was skipped.
    SkippedToken,
    /// A required token was treated as inserted for continued diagnosis.
    InsertedToken,
    /// Processing stopped at a hard logical boundary.
    Stopped,
}

/// One stable, source-located parse diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    /// Machine-readable stable code.
    pub code: DiagnosticCode,
    /// Closed severity value.
    pub severity: DiagnosticSeverity,
    /// Half-open UTF-8 byte span.
    pub span: SourceSpan,
    /// Stable rendering of the observed token or end-of-input.
    pub found: String,
    /// Ordered expected-token categories.
    pub expected: Vec<ExpectedToken>,
    /// Recovery action taken.
    pub recovery: RecoveryAction,
    /// Human-readable message not intended for programmatic matching.
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}..{}: {}",
            self.code,
            self.span.start(),
            self.span.end(),
            self.message
        )
    }
}

impl std::error::Error for Diagnostic {}

/// Complete versioned result of one parse attempt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParseReport {
    /// Diagnostic report schema identity.
    pub schema_version: String,
    /// Text dialect identity.
    pub dialect_revision: String,
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
    /// Stable diagnostics in encounter order.
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseReport {
    pub(crate) fn empty(
        profile: SemanticProfile,
        limits: ParseLimits,
        source_bytes: usize,
    ) -> Self {
        Self {
            schema_version: DIAGNOSTIC_SCHEMA_VERSION.to_owned(),
            dialect_revision: DIALECT_REVISION.to_owned(),
            tl_syntax_revision: TL_SYNTAX_REVISION.to_owned(),
            semantic_profile: profile,
            limits,
            stats: ParseStats {
                source_bytes,
                ..ParseStats::default()
            },
            document: None,
            diagnostics: Vec::new(),
        }
    }
}

/// Caller-selectable canonical-format limits, clamped to hard maxima.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatLimits {
    /// Maximum final canonical output bytes.
    pub max_output_bytes: usize,
    /// Maximum deterministic formatter work units.
    pub max_work: usize,
}

impl FormatLimits {
    pub(crate) fn clamped(self) -> Self {
        Self {
            max_output_bytes: self.max_output_bytes.min(HARD_MAX_OUTPUT_BYTES),
            max_work: self.max_work.min(HARD_MAX_FORMAT_WORK),
        }
    }
}

impl Default for FormatLimits {
    fn default() -> Self {
        Self {
            max_output_bytes: HARD_MAX_OUTPUT_BYTES,
            max_work: HARD_MAX_FORMAT_WORK,
        }
    }
}

/// Formatter observations.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatStats {
    /// Reachable graph nodes rendered.
    pub nodes: usize,
    /// Deterministic work units consumed: one per expanded node plus one per emitted byte.
    pub work: usize,
    /// Bytes in the final canonical output, or zero on error.
    pub output_bytes: usize,
}

/// Stable canonical-format failure codes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatErrorCode {
    /// Input document did not validate through tl-syntax.
    InvalidGraph,
    /// Final or intermediate text exceeds the output-byte boundary.
    OutputLimit,
    /// Total formatter work exceeds the logical-work boundary.
    WorkLimit,
}

impl FormatErrorCode {
    /// Returns the stable wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidGraph => "invalid_graph",
            Self::OutputLimit => "output_limit",
            Self::WorkLimit => "work_limit",
        }
    }
}

impl fmt::Display for FormatErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Typed canonical-format failure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatError {
    /// Machine-readable stable code.
    pub code: FormatErrorCode,
    /// Human-readable context.
    pub message: String,
}

impl fmt::Display for FormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for FormatError {}

/// Complete result of bounded canonical formatting.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormatReport {
    /// Effective limits after process-safe clamping.
    pub limits: FormatLimits,
    /// Deterministic observed counts.
    pub stats: FormatStats,
    /// Canonical text, present only on success.
    pub text: Option<String>,
    /// Typed failure, present only on failure.
    pub error: Option<FormatError>,
}
