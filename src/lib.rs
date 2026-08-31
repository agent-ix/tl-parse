//! Deterministic, bounded parsing and canonical formatting for bounded MLTL.
//!
//! The crate implements the independently authored ASCII dialect identified by
//! [`DIALECT_REVISION`]. Successful parsing returns the exact pinned
//! [`tl_syntax::FormulaDocument`] model; tl-parse owns no second temporal AST.

mod diagnostic;
mod format;
mod lexer;
mod parser;

pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ExpectedToken, FormatError, FormatErrorCode,
    FormatLimits, FormatReport, FormatStats, ParseLimits, ParseReport, ParseStats, RecoveryAction,
};
pub use format::{format_document, format_formula};
pub use parser::{parse, source_limit_report};
pub use tl_syntax;

/// Stable identity of the independently authored textual dialect.
pub const DIALECT_REVISION: &str = "tl-parse.clean-ascii/v1";

/// Stable identity of serialized parser diagnostic reports.
pub const DIAGNOSTIC_SCHEMA_VERSION: &str = "tl-parse.diagnostics/v1";

/// Exact tl-syntax source revision compiled into this crate.
pub const TL_SYNTAX_REVISION: &str = "740182f13b84858008d6f176f75136737d405c1b";

/// Stable revision of the checked-in hostile-input and fuzz-seed corpus.
pub const CORPUS_REVISION: &str = "tl-parse-corpus/v1";

/// Normative dialect record hashed by [`dialect_digest`].
pub const DIALECT_RECORD: &str = concat!(
    "false|true|p<canonical-u32>|!|&|||->|<->|",
    "F[canonical-u32,canonical-u32]|G[canonical-u32,canonical-u32]|",
    "U[canonical-u32,canonical-u32]|R[canonical-u32,canonical-u32]|",
    "precedence:prefix>UR>&>|>implies-right>equivalent-left|",
    "whitespace:space,tab,cr,lf|profile:external"
);

/// Complete normative dialect document retained with the implementation.
pub const DIALECT_DOCUMENT: &str = include_str!("../docs/DIALECT-001-clean-room-mltl-v1.md");

/// Complete clean-room source and license attribution record.
pub const ATTRIBUTION_DOCUMENT: &str = include_str!("../docs/ATTRIBUTION.md");

/// Returns the lowercase SHA-256 digest of the normative dialect record.
pub fn dialect_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(DIALECT_RECORD.as_bytes()))
}

/// Returns the SHA-256 digest of the complete normative dialect document.
pub fn dialect_document_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(DIALECT_DOCUMENT.as_bytes()))
}

/// Returns the SHA-256 digest of the complete attribution boundary document.
pub fn attribution_document_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(ATTRIBUTION_DOCUMENT.as_bytes()))
}

/// Serializes a parse report as compact, stable-key-order JSON.
pub fn report_json(report: &ParseReport) -> Result<String, serde_json::Error> {
    serde_json::to_string(report)
}
