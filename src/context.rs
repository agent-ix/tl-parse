//! Additive typed-signal binding for successful parser results.

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_syntax::{PropositionId, RequirementContextDocument, SemanticProfile, SignalCatalogDocument, SignalId, SourceSpan};

use crate::{lexer::{lex, TokenKind}, parse, ParseLimits, TL_SYNTAX_REVISION};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoundProposition {
    pub proposition: PropositionId,
    pub parser_span: SourceSpan,
    pub signal: SignalId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextualParseReport {
    pub schema_version: String,
    pub tl_syntax_revision: String,
    pub signal_catalog_sha256: String,
    pub request_sha256: String,
    pub requirement_context: Option<RequirementContextDocument>,
    pub bindings: Vec<BoundProposition>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualParseError {
    ParseFailed,
    InvalidCatalog(String),
    MissingProposition { proposition: PropositionId, parser_span: SourceSpan },
    Identity(String),
}

/// Parses as usual, then binds each first source occurrence to the shared catalog.
pub fn parse_with_context(
    source: &str,
    profile: SemanticProfile,
    limits: ParseLimits,
    catalog_document: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<ContextualParseReport, ContextualParseError> {
    let report = parse(source, profile, limits);
    let document = report.document.ok_or(ContextualParseError::ParseFailed)?;
    let catalog = catalog_document.validate().map_err(|error| ContextualParseError::InvalidCatalog(error.to_string()))?;
    let mut bindings = Vec::new();
    for token in lex(source, report.limits).tokens {
        let TokenKind::Proposition(raw) = token.kind else { continue };
        let proposition = PropositionId(raw);
        if bindings.iter().any(|binding: &BoundProposition| binding.proposition == proposition) { continue; }
        let parser_span = token.span();
        let signal = catalog.signal_for_proposition(proposition).ok_or(ContextualParseError::MissingProposition { proposition, parser_span })?;
        bindings.push(BoundProposition { proposition, parser_span, signal: signal.id() });
    }
    let catalog_bytes = serde_json::to_vec(catalog_document).map_err(|error| ContextualParseError::Identity(error.to_string()))?;
    let signal_catalog_sha256 = format!("{:x}", Sha256::digest(&catalog_bytes));
    let request = serde_json::json!({"document": document, "catalog": catalog_document, "context": requirement_context, "syntaxRevision": TL_SYNTAX_REVISION});
    let request_sha256 = format!("{:x}", Sha256::digest(serde_json::to_vec(&request).map_err(|error| ContextualParseError::Identity(error.to_string()))?));
    Ok(ContextualParseReport { schema_version: "tl-parse.contextual-binding/v2".to_owned(), tl_syntax_revision: TL_SYNTAX_REVISION.to_owned(), signal_catalog_sha256, request_sha256, requirement_context: requirement_context.cloned(), bindings })
}
