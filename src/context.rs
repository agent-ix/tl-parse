//! Additive shared-catalog/context binding for successful parser results.

use core::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use tl_syntax::{
    FormulaDocument, NodeKind, PropositionId, RequirementContextDocument, SemanticProfile,
    SignalCatalogDocument, SignalId, SourceSpan,
};

use crate::{parse, ParseLimits, TL_SYNTAX_REVISION};

/// Strict wire identity for a context-bound parser result.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ContextualParseSchemaVersion {
    /// Initial context-bound parser result wire format.
    #[serde(rename = "tl-parse.contextual-binding/v2")]
    V2,
}

/// One free proposition resolved through the supplied shared signal catalog.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoundProposition {
    /// Proposition identity from the parsed formula graph.
    pub proposition: PropositionId,
    /// Exact byte span of that proposition's first parser node.
    pub parser_span: SourceSpan,
    /// Shared catalog signal identity; no local signal schema is introduced.
    pub signal: SignalId,
}

/// Complete versioned result of parsing and binding a formula to shared inputs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "ContextualParseReportWire")]
pub struct ContextualParseReport {
    /// Closed wire identity for this report family.
    pub schema_version: ContextualParseSchemaVersion,
    /// Exact pinned tl-syntax revision used to validate the shared documents.
    pub tl_syntax_revision: String,
    /// Successfully parsed, validated formula document.
    pub formula_document: FormulaDocument,
    /// Exact caller-supplied shared signal catalog.
    pub signal_catalog: SignalCatalogDocument,
    /// Digest of the exact serialized shared signal catalog.
    pub signal_catalog_sha256: String,
    /// Explicitly nullable caller-supplied shared requirement context.
    pub requirement_context: Option<RequirementContextDocument>,
    /// Domain-separated digest of all binding request identities.
    pub request_sha256: String,
    /// Resolved free propositions in first parser-node order.
    pub bindings: Vec<BoundProposition>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContextualParseReportWire {
    schema_version: ContextualParseSchemaVersion,
    #[serde(deserialize_with = "deserialize_tl_syntax_revision")]
    tl_syntax_revision: String,
    formula_document: FormulaDocument,
    signal_catalog: SignalCatalogDocument,
    signal_catalog_sha256: String,
    requirement_context: ExplicitNullableContext,
    request_sha256: String,
    bindings: Vec<BoundProposition>,
}

/// A present wire field whose value may be explicitly `null`.
///
/// The wrapper is intentionally not an `Option`: serde therefore rejects an
/// omitted field, while its inner `Option` accepts JSON `null`.
#[derive(Deserialize)]
struct ExplicitNullableContext(Option<RequirementContextDocument>);

impl TryFrom<ContextualParseReportWire> for ContextualParseReport {
    type Error = String;

    fn try_from(wire: ContextualParseReportWire) -> Result<Self, Self::Error> {
        let requirement_context = wire.requirement_context.0;
        let expected_catalog_digest =
            digest(&wire.signal_catalog).map_err(|error| error.to_string())?;
        if wire.signal_catalog_sha256 != expected_catalog_digest {
            return Err("signal catalog digest does not match the retained catalog".into());
        }
        let expected_request_digest = request_digest(
            &wire.formula_document,
            &wire.signal_catalog,
            requirement_context.as_ref(),
        )
        .map_err(|error| error.to_string())?;
        if wire.request_sha256 != expected_request_digest {
            return Err("request digest does not match the retained inputs".into());
        }
        let expected_bindings = bindings_for(&wire.formula_document, &wire.signal_catalog)
            .map_err(|error| error.to_string())?;
        if wire.bindings != expected_bindings {
            return Err("bindings do not match the retained formula and catalog".into());
        }
        Ok(Self {
            schema_version: wire.schema_version,
            tl_syntax_revision: wire.tl_syntax_revision,
            formula_document: wire.formula_document,
            signal_catalog: wire.signal_catalog,
            signal_catalog_sha256: wire.signal_catalog_sha256,
            requirement_context,
            request_sha256: wire.request_sha256,
            bindings: wire.bindings,
        })
    }
}

fn deserialize_tl_syntax_revision<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let revision = String::deserialize(deserializer)?;
    if revision == TL_SYNTAX_REVISION {
        Ok(revision)
    } else {
        Err(serde::de::Error::custom("unexpected tl-syntax revision"))
    }
}

/// Non-success outcomes for context-bound parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualParseError {
    /// The ordinary parser produced no valid formula document.
    ParseFailed,
    /// The supplied shared signal catalog is invalid.
    InvalidCatalog(String),
    /// A parsed proposition node had no parser span, which violates this API's contract.
    MissingParserSpan { proposition: PropositionId },
    /// The shared catalog does not bind a proposition referenced by the formula.
    MissingProposition {
        /// Unresolved proposition identity.
        proposition: PropositionId,
        /// Exact source byte span on the first parser node.
        parser_span: SourceSpan,
    },
    /// A deterministic identity could not be serialized.
    Identity(String),
}

impl fmt::Display for ContextualParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseFailed => formatter.write_str("parse failed"),
            Self::InvalidCatalog(error) => write!(formatter, "invalid signal catalog: {error}"),
            Self::MissingParserSpan { proposition } => {
                write!(
                    formatter,
                    "missing parser span for proposition {}",
                    proposition.0
                )
            }
            Self::MissingProposition {
                proposition,
                parser_span,
            } => write!(
                formatter,
                "missing signal binding for proposition {} at {}..{}",
                proposition.0,
                parser_span.start(),
                parser_span.end()
            ),
            Self::Identity(error) => {
                write!(formatter, "binding identity serialization failed: {error}")
            }
        }
    }
}

impl std::error::Error for ContextualParseError {}

/// Parses as usual, then binds each free proposition to the shared catalog.
pub fn parse_with_context(
    source: &str,
    profile: SemanticProfile,
    limits: ParseLimits,
    catalog_document: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<ContextualParseReport, ContextualParseError> {
    let report = parse(source, profile, limits);
    let formula_document = report.document.ok_or(ContextualParseError::ParseFailed)?;
    let bindings = bindings_for(&formula_document, catalog_document)?;

    let signal_catalog_sha256 = digest(catalog_document)?;
    let request_sha256 = request_digest(&formula_document, catalog_document, requirement_context)?;
    Ok(ContextualParseReport {
        schema_version: ContextualParseSchemaVersion::V2,
        tl_syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        formula_document,
        signal_catalog: catalog_document.clone(),
        signal_catalog_sha256,
        requirement_context: requirement_context.cloned(),
        request_sha256,
        bindings,
    })
}

fn bindings_for(
    formula_document: &FormulaDocument,
    catalog_document: &SignalCatalogDocument,
) -> Result<Vec<BoundProposition>, ContextualParseError> {
    let catalog = catalog_document
        .validate()
        .map_err(|error| ContextualParseError::InvalidCatalog(error.to_string()))?;
    let mut bindings = Vec::new();
    for node in formula_document.nodes() {
        let NodeKind::Proposition { proposition } = node.kind else {
            continue;
        };
        if bindings
            .iter()
            .any(|binding: &BoundProposition| binding.proposition == proposition)
        {
            continue;
        }
        let parser_span = node
            .span
            .ok_or(ContextualParseError::MissingParserSpan { proposition })?;
        let signal = catalog.signal_for_proposition(proposition).ok_or(
            ContextualParseError::MissingProposition {
                proposition,
                parser_span,
            },
        )?;
        bindings.push(BoundProposition {
            proposition,
            parser_span,
            signal: signal.id(),
        });
    }

    Ok(bindings)
}

fn request_digest(
    formula_document: &FormulaDocument,
    catalog_document: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<String, ContextualParseError> {
    digest(&serde_json::json!({
        "domain": "tl-parse.contextual-binding/request/v2",
        "formulaDocument": formula_document,
        "signalCatalog": catalog_document,
        "requirementContext": requirement_context,
        "tlSyntaxRevision": TL_SYNTAX_REVISION,
    }))
}

fn digest(value: &impl Serialize) -> Result<String, ContextualParseError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| ContextualParseError::Identity(error.to_string()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
