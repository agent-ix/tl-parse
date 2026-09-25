#![forbid(unsafe_code)]
//! Deterministic, bounded parsing and canonical formatting for bounded MLTL.
//!
//! The crate implements the independently authored ASCII dialect identified by
//! [`DIALECT_REVISION`]. Successful parsing returns the exact pinned
//! [`tl_syntax::FormulaDocument`] model; tl-parse owns no second temporal AST.

mod context;
mod diagnostic;
mod dialect;
mod formatter;
mod infinite;
mod lexer;
mod parser;

pub use context::{
    parse_with_context, BoundProposition, ContextualParseError, ContextualParseReport,
    ContextualParseSchemaVersion,
};
pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ExpectedToken, FormatError, FormatErrorCode,
    FormatLimits, FormatReport, FormatStats, ParseArtifactLimits, ParseLimits, ParseReport,
    ParseStats, RecoveryAction, StrictParseArtifactReadError,
};
pub use dialect::v2::{
    parse_clean_ascii_v2, DerivedDialectRevision, DerivedOperator, DerivedOperatorProfile,
    DerivedParseReport, DerivedParseSchemaVersion, LoweringRecord,
};
pub use dialect::v3::{
    parse_clean_ascii_v3, PastDialectRevision, PastOperatorProfile, PastParseReport,
    PastParseSchemaVersion,
};
pub use formatter::{format_clean_ascii_v3, format_document, format_formula};
pub use infinite::{
    format_clean_ascii_v4, parse_clean_ascii_v4, InfiniteDialectRevision, InfiniteDisposition,
    InfiniteParseReport, InfiniteParseSchemaVersion,
};
pub use parser::{parse, source_limit_report};
pub use tl_syntax;

/// Stable identity of the independently authored textual dialect.
pub const DIALECT_REVISION: &str = "tl-parse.clean-ascii/v1";

/// Stable identity of the explicitly selected derived-operator input dialect.
pub const DIALECT_V2_REVISION: &str = "tl-parse.clean-ascii/v2";

/// Stable identity of the origin-complete past-profile input dialect.
pub const DIALECT_V3_REVISION: &str = "tl-parse.clean-ascii/v3";

/// Stable identity of the infinite-trace input dialect.
pub const DIALECT_V4_REVISION: &str = "tl-parse.clean-ascii/v4";

/// Normative v4 dialect document.
pub const DIALECT_V4_DOCUMENT: &str = include_str!("../docs/DIALECT-004-clean-ascii-v4.md");

/// SHA-256 digest of the complete v4 dialect document.
pub fn dialect_v4_document_digest() -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(DIALECT_V4_DOCUMENT.as_bytes()))
}

/// Stable identity of serialized v2 derived-operator parse reports.
pub const DERIVED_PARSE_REPORT_SCHEMA_VERSION: &str = "tl-parse.derived-parse-report/v1";

/// Stable identity of serialized clean-ascii/v3 parse reports.
pub const PAST_PARSE_REPORT_SCHEMA_VERSION: &str = "tl-parse.past-parse-report/v1";

/// Stable identity of serialized parser diagnostic reports.
pub const DIAGNOSTIC_SCHEMA_VERSION: &str = "tl-parse.diagnostics/v1";

/// Exact tl-syntax source revision compiled into this crate.
pub const TL_SYNTAX_REVISION: &str = "6e2fc17fcfba60c33ab264772bb25550a9c81853";

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

/// Normative v2 dialect record hashed by [`dialect_v2_digest`].
pub const DIALECT_V2_RECORD: &str = concat!(
    "tl-parse.clean-ascii/v1|",
    "W[canonical-u32,canonical-u32]|M[canonical-u32,canonical-u32]|",
    "precedence:WM=UR,left|unsupported:X,Y,O,H,S,T|",
    "lowering:tl-syntax.future-lowering-request/v1|operators:tl-syntax.future-operators/v1"
);

/// Normative v3 dialect record hashed by [`dialect_v3_digest`].
pub const DIALECT_V3_RECORD: &str = concat!(
    "boolean:tl-parse.clean-ascii/v1|",
    "O[canonical-u32,canonical-u32]|H[canonical-u32,canonical-u32]|Y|",
    "S[canonical-u32,canonical-u32]|T[canonical-u32,canonical-u32]|",
    "precedence:prefix>ST>&>|>implies-right>equivalent-left|associativity:ST-left|",
    "profile:mltl.origin-complete-history/v1|operators:tl-syntax.past-operators/v1|",
    "unsupported:X,F,G,U,R,W,M,weak-previous,long-names"
);

/// Complete normative v2 dialect document retained with the implementation.
pub const DIALECT_V2_DOCUMENT: &str = include_str!("../docs/DIALECT-002-clean-ascii-v2.md");

/// Complete normative v3 dialect document retained with the implementation.
pub const DIALECT_V3_DOCUMENT: &str = include_str!("../docs/DIALECT-003-clean-ascii-v3.md");

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

/// Returns the lowercase SHA-256 digest of the normative v2 dialect record.
pub fn dialect_v2_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(DIALECT_V2_RECORD.as_bytes()))
}

/// Returns the SHA-256 digest of the complete normative v2 dialect document.
pub fn dialect_v2_document_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(DIALECT_V2_DOCUMENT.as_bytes()))
}

/// Returns the lowercase SHA-256 digest of the normative v3 dialect record.
pub fn dialect_v3_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(DIALECT_V3_RECORD.as_bytes()))
}

/// Returns the SHA-256 digest of the complete normative v3 dialect document.
pub fn dialect_v3_document_digest() -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(DIALECT_V3_DOCUMENT.as_bytes()))
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
