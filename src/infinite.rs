//! Explicit infinite-trace v4 parser. Its nodes belong to tl-syntax directly.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tl_syntax::{
    lower_infinite_future, FairnessPremisesDocument, FutureKind, InfiniteClock,
    InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind as Kind, Interval, NodeId,
    PropositionId, SemanticProfile, SourceSpan, SyntaxArtifactLimits, TemporalInterval,
    UnboundedInterval,
};

use crate::{
    diagnostic::preflight_parse_artifact,
    dialect::Dialect,
    lexer::{checked_span, lex, Token, TokenKind},
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ExpectedToken, FormatError, FormatErrorCode,
    FormatLimits, FormatReport, FormatStats, ParseArtifactLimits, ParseLimits, ParseStats,
    RecoveryAction, StrictParseArtifactReadError, TL_SYNTAX_REVISION,
};

/// Wire identity of a v4 parse attempt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InfiniteParseSchemaVersion {
    /// Initial infinite parser report.
    #[serde(rename = "tl-parse.infinite-parse-report/v1")]
    V1,
}

/// Closed infinite text dialect identity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum InfiniteDialectRevision {
    /// Mixed future and past syntax with `[a,)` and fairness.
    #[serde(rename = "tl-parse.clean-ascii/v4")]
    V4,
}

/// FR-341 routing for parsing; parsing never makes a temporal judgment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InfiniteDisposition {
    /// Syntax, profile, or clock refusal before evaluation.
    Unsupported,
    /// A work or memory ceiling stopped the request.
    ResourceIncomplete,
    /// An internal owner validation failure stopped the request.
    Failed,
    /// A graph was constructed, with no temporal verdict.
    NoTemporalVerdict,
}

/// Result of one bounded v4 parse attempt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "InfiniteParseReportWire")]
pub struct InfiniteParseReport {
    /// Report wire identity.
    pub schema_version: InfiniteParseSchemaVersion,
    /// Selected text dialect.
    pub dialect_revision: InfiniteDialectRevision,
    /// Exact compiled tl-syntax revision.
    pub tl_syntax_revision: String,
    /// Selected semantic profile.
    pub semantic_profile: SemanticProfile,
    /// Selected clock spelling.
    pub clock: String,
    /// Effective process-safe limits.
    pub limits: ParseLimits,
    /// Deterministic counters.
    pub stats: ParseStats,
    /// Validated owner graph on success only.
    pub document: Option<InfiniteFormulaDocument>,
    /// Ordered fairness roots in the same graph on success only.
    pub fairness: Option<FairnessPremisesDocument>,
    /// Byte spans of every interval, in source order.
    pub interval_spans: Vec<SourceSpan>,
    /// Byte spans of fairness premises, in source order.
    pub premise_spans: Vec<SourceSpan>,
    /// Typed source-located refusals.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InfiniteParseReportWire {
    schema_version: InfiniteParseSchemaVersion,
    dialect_revision: InfiniteDialectRevision,
    tl_syntax_revision: String,
    semantic_profile: SemanticProfile,
    clock: String,
    limits: ParseLimits,
    stats: ParseStats,
    document: Option<InfiniteFormulaDocument>,
    fairness: Option<FairnessPremisesDocument>,
    interval_spans: Vec<SourceSpan>,
    premise_spans: Vec<SourceSpan>,
    diagnostics: Vec<Diagnostic>,
}

impl TryFrom<InfiniteParseReportWire> for InfiniteParseReport {
    type Error = &'static str;

    fn try_from(wire: InfiniteParseReportWire) -> Result<Self, Self::Error> {
        if wire.tl_syntax_revision != TL_SYNTAX_REVISION {
            return Err("unexpected compiled tl-syntax revision");
        }
        if wire.limits != wire.limits.clamped()
            || wire.stats.tokens > wire.limits.max_tokens
            || wire.stats.nodes > wire.limits.max_nodes
            || wire.stats.work > wire.limits.max_work
            || wire.stats.diagnostics != wire.diagnostics.len()
            || wire.stats.diagnostics > wire.limits.max_diagnostics
        {
            return Err("v4 report counters exceed effective limits");
        }
        let bounded = |span: &SourceSpan| {
            usize::try_from(span.end()).is_ok_and(|end| end <= wire.stats.source_bytes)
        };
        if wire
            .interval_spans
            .iter()
            .chain(&wire.premise_spans)
            .any(|span| !bounded(span))
            || wire
                .diagnostics
                .iter()
                .any(|diagnostic| !bounded(&diagnostic.span))
        {
            return Err("v4 report span exceeds source bytes");
        }
        if wire.document.is_none()
            && wire.diagnostics.is_empty()
            && !wire.stats.diagnostics_truncated
        {
            return Err("failed v4 report requires a diagnostic or truncated diagnostics");
        }
        if let Some(document) = wire.document.as_ref() {
            if wire.semantic_profile != SemanticProfile::InfiniteTraceV1
                || wire.clock != InfiniteClock::EventPosition.as_str()
                || !wire.diagnostics.is_empty()
                || wire.stats.diagnostics_truncated
                || wire.stats.source_bytes > wire.limits.max_source_bytes
                || wire.stats.max_depth > wire.limits.max_depth
                || wire.stats.nodes != document.nodes().len()
            {
                return Err("v4 successful report has conflicting identities or diagnostics");
            }
            if document
                .nodes()
                .iter()
                .any(|node| node.span.is_none_or(|span| !bounded(&span)))
            {
                return Err("v4 graph node span exceeds source bytes");
            }
            if let Some(fairness) = wire.fairness.as_ref() {
                let identity = document
                    .content_identity()
                    .map_err(|_| "v4 graph identity encoding failed")?;
                if fairness.graph_identity() != identity
                    || fairness.clock() != document.clock()
                    || fairness
                        .roots()
                        .iter()
                        .any(|root| document.formula().node(*root).is_none())
                    || fairness.roots().len() != wire.premise_spans.len()
                {
                    return Err("v4 fairness binding differs from owner graph");
                }
            } else if !wire.premise_spans.is_empty() {
                return Err("v4 premise spans require fairness roots");
            }
        } else if wire.fairness.is_some() {
            return Err("v4 fairness requires a successful graph");
        }
        Ok(Self {
            schema_version: wire.schema_version,
            dialect_revision: wire.dialect_revision,
            tl_syntax_revision: wire.tl_syntax_revision,
            semantic_profile: wire.semantic_profile,
            clock: wire.clock,
            limits: wire.limits,
            stats: wire.stats,
            document: wire.document,
            fairness: wire.fairness,
            interval_spans: wire.interval_spans,
            premise_spans: wire.premise_spans,
            diagnostics: wire.diagnostics,
        })
    }
}

impl InfiniteParseReport {
    /// Strict-reads one canonical, bounded v4 report.
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

    /// Maps a diagnostic code to FR-341 without inspecting message text.
    pub fn disposition(&self) -> InfiniteDisposition {
        if self.semantic_profile != SemanticProfile::InfiniteTraceV1
            || self.clock != InfiniteClock::EventPosition.as_str()
        {
            return InfiniteDisposition::Unsupported;
        }
        match self.diagnostics.first().map(|d| d.code) {
            None if self.document.is_some() => InfiniteDisposition::NoTemporalVerdict,
            None => InfiniteDisposition::ResourceIncomplete,
            Some(
                DiagnosticCode::SourceLimit
                | DiagnosticCode::TokenLimit
                | DiagnosticCode::NodeLimit
                | DiagnosticCode::DepthLimit
                | DiagnosticCode::WorkLimit,
            ) => InfiniteDisposition::ResourceIncomplete,
            Some(DiagnosticCode::ValidationFailure) => InfiniteDisposition::Failed,
            Some(_) => InfiniteDisposition::Unsupported,
        }
    }
}

#[derive(Clone, Copy)]
struct Parsed {
    id: NodeId,
    start: usize,
    end: usize,
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
    nodes: Vec<InfiniteNode>,
    canonical_classes: Vec<NodeId>,
    canonical_map: BTreeMap<Kind, NodeId>,
    limits: ParseLimits,
    work: usize,
    max_depth: usize,
    diagnostics: Vec<Diagnostic>,
    diagnostics_truncated: bool,
    premise_roots: Vec<NodeId>,
    premise_spans: Vec<SourceSpan>,
    interval_spans: Vec<SourceSpan>,
}

/// Parses the explicitly selected v4 dialect and infinite profile.
///
/// A failed request never returns a usable graph. Fairness assumptions are
/// captured as syntax and are never evaluated here.
pub fn parse_clean_ascii_v4(
    source: &str,
    profile: SemanticProfile,
    clock: &str,
    limits: ParseLimits,
) -> InfiniteParseReport {
    let limits = limits.clamped();
    let mut report = InfiniteParseReport {
        schema_version: InfiniteParseSchemaVersion::V1,
        dialect_revision: InfiniteDialectRevision::V4,
        tl_syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        semantic_profile: profile,
        clock: clock.to_owned(),
        limits,
        stats: ParseStats {
            source_bytes: source.len(),
            ..ParseStats::default()
        },
        document: None,
        fairness: None,
        interval_spans: Vec::new(),
        premise_spans: Vec::new(),
        diagnostics: Vec::new(),
    };
    if profile != SemanticProfile::InfiniteTraceV1 {
        let refusal = diagnostic(
            source,
            DiagnosticCode::InfiniteProfileMismatch,
            0,
            0,
            vec![],
            "v4 requires mltl.infinite-trace/v1",
        );
        if limits.max_diagnostics > 0 {
            report.diagnostics.push(refusal);
            report.stats.diagnostics = 1;
        } else {
            report.stats.diagnostics_truncated = true;
        }
        return report;
    }
    if clock != InfiniteClock::EventPosition.as_str() {
        let refusal = diagnostic(
            source,
            DiagnosticCode::InfiniteClockMismatch,
            0,
            0,
            vec![],
            "v4 requires event_position clock",
        );
        if limits.max_diagnostics > 0 {
            report.diagnostics.push(refusal);
            report.stats.diagnostics = 1;
        } else {
            report.stats.diagnostics_truncated = true;
        }
        return report;
    }
    let lexed = lex(source, limits, Dialect::V4);
    report.stats.tokens = lexed.tokens.len().saturating_sub(1);
    report.stats.work = lexed.work;
    report.stats.diagnostics_truncated = lexed.diagnostics_truncated;
    if lexed.had_error {
        report.diagnostics = lexed.diagnostics;
        report.stats.diagnostics = report.diagnostics.len();
        return report;
    }
    let mut parser = Parser {
        source,
        tokens: lexed.tokens,
        cursor: 0,
        nodes: Vec::new(),
        canonical_classes: Vec::new(),
        canonical_map: BTreeMap::new(),
        limits,
        work: lexed.work,
        max_depth: 0,
        diagnostics: Vec::new(),
        diagnostics_truncated: false,
        premise_roots: Vec::new(),
        premise_spans: Vec::new(),
        interval_spans: Vec::new(),
    };
    let root = parser.parse_document();
    report.stats.nodes = parser.nodes.len();
    report.stats.work = parser.work;
    report.stats.max_depth = parser.max_depth;
    report.interval_spans = parser.interval_spans;
    report.premise_spans = parser.premise_spans;
    report.diagnostics = parser.diagnostics;
    report.stats.diagnostics_truncated = parser.diagnostics_truncated;
    if let Some(root) =
        root.filter(|_| report.diagnostics.is_empty() && !report.stats.diagnostics_truncated)
    {
        match InfiniteFormulaDocument::new(
            profile,
            InfiniteClock::EventPosition,
            root.id,
            parser.nodes,
        ) {
            Ok(document) => {
                let document = match strictly_admit_document(document) {
                    Ok(document) => document,
                    Err(message) => {
                        report.diagnostics.push(diagnostic(
                            source,
                            DiagnosticCode::ValidationFailure,
                            root.start,
                            root.end,
                            vec![],
                            &message,
                        ));
                        report.stats.diagnostics = report.diagnostics.len();
                        return report;
                    }
                };
                if !parser.premise_roots.is_empty() {
                    match document
                        .content_identity()
                        .map_err(|e| e.to_string())
                        .and_then(|identity| {
                            FairnessPremisesDocument::new(
                                &document,
                                identity,
                                InfiniteClock::EventPosition,
                                parser.premise_roots,
                            )
                            .map_err(|e| e.to_string())
                        }) {
                        Ok(fairness) => match strictly_admit_fairness(fairness, &document) {
                            Ok(fairness) => report.fairness = Some(fairness),
                            Err(error) => report.diagnostics.push(diagnostic(
                                source,
                                DiagnosticCode::ValidationFailure,
                                root.start,
                                root.end,
                                vec![],
                                &error,
                            )),
                        },
                        Err(error) => report.diagnostics.push(diagnostic(
                            source,
                            DiagnosticCode::ValidationFailure,
                            root.start,
                            root.end,
                            vec![],
                            &error,
                        )),
                    }
                }
                if report.diagnostics.is_empty() {
                    report.document = Some(document);
                }
            }
            Err(error) => report.diagnostics.push(diagnostic(
                source,
                DiagnosticCode::ValidationFailure,
                root.start,
                root.end,
                vec![],
                &error.to_string(),
            )),
        }
    }
    report.stats.diagnostics = report.diagnostics.len();
    report
}

fn strictly_admit_document(
    document: InfiniteFormulaDocument,
) -> Result<InfiniteFormulaDocument, String> {
    let bytes = document
        .canonical_json_bytes()
        .map_err(|error| error.to_string())?;
    let admitted =
        InfiniteFormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default())
            .map_err(|error| error.to_string())?;
    if admitted == document {
        Ok(admitted)
    } else {
        Err("strict owner admission changed the v4 graph".to_owned())
    }
}

fn strictly_admit_fairness(
    fairness: FairnessPremisesDocument,
    document: &InfiniteFormulaDocument,
) -> Result<FairnessPremisesDocument, String> {
    let bytes = fairness
        .canonical_json_bytes()
        .map_err(|error| error.to_string())?;
    let admitted = FairnessPremisesDocument::from_json_bytes(
        &bytes,
        SyntaxArtifactLimits::default(),
        document,
    )
    .map_err(|error| error.to_string())?;
    if admitted == fairness {
        Ok(admitted)
    } else {
        Err("strict owner admission changed v4 fairness".to_owned())
    }
}

fn diagnostic(
    source: &str,
    code: DiagnosticCode,
    start: usize,
    end: usize,
    expected: Vec<ExpectedToken>,
    message: &str,
) -> Diagnostic {
    let found = if start == end {
        "<eof>".to_owned()
    } else {
        format!("{:?}", &source[start..end])
    };
    Diagnostic {
        code,
        severity: DiagnosticSeverity::Error,
        span: checked_span(start, end),
        found,
        expected,
        recovery: RecoveryAction::Stopped,
        message: message.to_owned(),
    }
}

impl Parser<'_> {
    fn current(&self) -> Token {
        self.tokens[self.cursor]
    }
    fn advance(&mut self) {
        if self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
        if self.work < self.limits.max_work {
            self.work += 1;
        } else {
            let token = self.current();
            self.error(
                DiagnosticCode::WorkLimit,
                token,
                vec![],
                "v4 parser work limit exceeded",
            );
        }
    }
    fn error(
        &mut self,
        code: DiagnosticCode,
        token: Token,
        expected: Vec<ExpectedToken>,
        message: &str,
    ) {
        if self.diagnostics.len() < self.limits.max_diagnostics {
            self.diagnostics.push(diagnostic(
                self.source,
                code,
                token.start,
                token.end,
                expected,
                message,
            ));
        } else {
            self.diagnostics_truncated = true;
        }
    }
    fn check_work(&mut self) -> bool {
        if self.work >= self.limits.max_work {
            let token = self.current();
            self.error(
                DiagnosticCode::WorkLimit,
                token,
                vec![],
                "v4 parser work limit exceeded",
            );
            false
        } else {
            self.work += 1;
            true
        }
    }
    fn expect(&mut self, kind: TokenKind, expected: ExpectedToken) -> Option<Token> {
        let token = self.current();
        if token.kind == kind {
            self.advance();
            Some(token)
        } else {
            self.error(
                DiagnosticCode::MissingToken,
                token,
                vec![expected],
                "required delimiter is missing",
            );
            None
        }
    }
    fn parse_document(&mut self) -> Option<Parsed> {
        if self.current().kind == TokenKind::Fair {
            self.advance();
            self.expect(TokenKind::LeftBrace, ExpectedToken::LeftBrace)?;
            while self.current().kind != TokenKind::RightBrace {
                let premise = self.expression(1, 1)?;
                if self.premise_roots.iter().any(|id| {
                    self.canonical_classes[id.0 as usize]
                        == self.canonical_classes[premise.id.0 as usize]
                }) {
                    self.error(
                        DiagnosticCode::DuplicateFairnessPremise,
                        Token {
                            kind: TokenKind::Invalid,
                            start: premise.start,
                            end: premise.end,
                        },
                        vec![],
                        "duplicate fairness premise",
                    );
                    return None;
                }
                self.premise_roots.push(premise.id);
                self.premise_spans
                    .push(checked_span(premise.start, premise.end));
                if self.current().kind == TokenKind::Semicolon {
                    self.advance();
                } else if self.current().kind != TokenKind::RightBrace {
                    let token = self.current();
                    self.error(
                        DiagnosticCode::MissingToken,
                        token,
                        vec![ExpectedToken::Semicolon],
                        "fairness premise separator is missing",
                    );
                    return None;
                }
            }
            self.expect(TokenKind::RightBrace, ExpectedToken::RightBrace)?;
            self.expect(TokenKind::Colon, ExpectedToken::Colon)?;
        }
        let root = self.expression(1, 1)?;
        if self.current().kind != TokenKind::Eof {
            let token = self.current();
            self.error(
                DiagnosticCode::TrailingInput,
                token,
                vec![ExpectedToken::EndOfInput],
                "unexpected input after formula",
            );
            return None;
        }
        Some(root)
    }
    fn expression(&mut self, min_power: u8, depth: usize) -> Option<Parsed> {
        if depth > self.limits.max_depth {
            let token = self.current();
            self.error(
                DiagnosticCode::DepthLimit,
                token,
                vec![],
                "v4 parser depth limit exceeded",
            );
            return None;
        }
        self.max_depth = self.max_depth.max(depth);
        if !self.check_work() {
            return None;
        }
        let mut left = self.prefix(depth + 1)?;
        loop {
            let operator = self.current();
            let Some((left_power, right_power)) =
                crate::dialect::v4::binary_binding_power(operator.kind)
            else {
                break;
            };
            if left_power < min_power {
                break;
            }
            self.advance();
            let temporal = matches!(
                operator.kind,
                TokenKind::Until
                    | TokenKind::Release
                    | TokenKind::WeakUntil
                    | TokenKind::StrongRelease
                    | TokenKind::Since
                    | TokenKind::Triggered
            );
            let interval = if temporal {
                Some(self.interval()?)
            } else {
                None
            };
            let right = self.expression(right_power, depth + 1)?;
            if matches!(
                operator.kind,
                TokenKind::WeakUntil | TokenKind::StrongRelease
            ) {
                left = self.lower_derived(
                    operator.kind == TokenKind::StrongRelease,
                    interval?,
                    left,
                    right,
                )?;
                continue;
            }
            let kind = match operator.kind {
                TokenKind::Equivalent => Kind::Equivalent {
                    left: left.id,
                    right: right.id,
                },
                TokenKind::Implies => Kind::Implies {
                    left: left.id,
                    right: right.id,
                },
                TokenKind::Or => Kind::Or {
                    left: left.id,
                    right: right.id,
                },
                TokenKind::And => Kind::And {
                    left: left.id,
                    right: right.id,
                },
                TokenKind::Until => Kind::Until {
                    interval: interval?,
                    left: left.id,
                    right: right.id,
                },
                TokenKind::Release => Kind::Release {
                    interval: interval?,
                    left: left.id,
                    right: right.id,
                },
                TokenKind::Since => Kind::Since {
                    interval: interval?,
                    left: left.id,
                    right: right.id,
                },
                TokenKind::Triggered => Kind::Triggered {
                    interval: interval?,
                    left: left.id,
                    right: right.id,
                },
                _ => return None,
            };
            left = self.push(kind, left.start, right.end)?;
        }
        Some(left)
    }
    fn prefix(&mut self, depth: usize) -> Option<Parsed> {
        let token = self.current();
        match token.kind {
            TokenKind::False => {
                self.advance();
                self.push(Kind::False, token.start, token.end)
            }
            TokenKind::True => {
                self.advance();
                self.push(Kind::True, token.start, token.end)
            }
            TokenKind::Proposition(value) => {
                self.advance();
                self.push(
                    Kind::Proposition {
                        proposition: PropositionId(value),
                    },
                    token.start,
                    token.end,
                )
            }
            TokenKind::LeftParenthesis => {
                self.advance();
                let mut inner = self.expression(1, depth + 1)?;
                let close =
                    self.expect(TokenKind::RightParenthesis, ExpectedToken::RightParenthesis)?;
                inner.start = token.start;
                inner.end = close.end;
                Some(inner)
            }
            TokenKind::Not | TokenKind::StrongPrevious => {
                self.advance();
                let operand = self.expression(6, depth + 1)?;
                let kind = if token.kind == TokenKind::Not {
                    Kind::Not {
                        operand: operand.id,
                    }
                } else {
                    Kind::StrongPrevious {
                        operand: operand.id,
                    }
                };
                self.push(kind, token.start, operand.end)
            }
            TokenKind::Future | TokenKind::Globally | TokenKind::Once | TokenKind::Historically => {
                self.advance();
                let interval = self.interval()?;
                let operand = self.expression(6, depth + 1)?;
                let kind = match token.kind {
                    TokenKind::Future => Kind::Future {
                        interval,
                        operand: operand.id,
                    },
                    TokenKind::Globally => Kind::Globally {
                        interval,
                        operand: operand.id,
                    },
                    TokenKind::Once => Kind::Once {
                        interval,
                        operand: operand.id,
                    },
                    TokenKind::Historically => Kind::Historically {
                        interval,
                        operand: operand.id,
                    },
                    _ => return None,
                };
                self.push(kind, token.start, operand.end)
            }
            _ => {
                self.error(
                    DiagnosticCode::UnexpectedToken,
                    token,
                    vec![ExpectedToken::Expression],
                    "expected v4 formula expression",
                );
                None
            }
        }
    }
    fn interval(&mut self) -> Option<TemporalInterval> {
        let open = self.expect(TokenKind::LeftBracket, ExpectedToken::LeftBracket)?;
        let lower = self.current();
        let TokenKind::Number(start) = lower.kind else {
            self.error(
                DiagnosticCode::UnexpectedToken,
                lower,
                vec![ExpectedToken::Integer],
                "expected lower bound",
            );
            return None;
        };
        self.advance();
        self.expect(TokenKind::Comma, ExpectedToken::Comma)?;
        let upper = self.current();
        let interval = match upper.kind {
            TokenKind::Number(end) => {
                self.advance();
                let closed = Interval::new(start, end).ok();
                if closed.is_none() {
                    self.error(
                        DiagnosticCode::InvalidInterval,
                        upper,
                        vec![],
                        "interval lower bound exceeds upper bound",
                    );
                }
                TemporalInterval::Closed(closed?)
            }
            TokenKind::RightParenthesis => {
                TemporalInterval::Unbounded(UnboundedInterval::new(start))
            }
            _ => {
                self.error(
                    if upper.kind == TokenKind::RightBracket {
                        DiagnosticCode::MissingToken
                    } else {
                        DiagnosticCode::UnexpectedToken
                    },
                    upper,
                    vec![ExpectedToken::Integer],
                    "expected upper bound or open-upper parenthesis",
                );
                return None;
            }
        };
        let close = if matches!(interval, TemporalInterval::Unbounded(_)) {
            self.expect(TokenKind::RightParenthesis, ExpectedToken::RightParenthesis)?
        } else {
            self.expect(TokenKind::RightBracket, ExpectedToken::RightBracket)?
        };
        self.interval_spans
            .push(checked_span(open.start, close.end));
        Some(interval)
    }
    fn push(&mut self, kind: Kind, start: usize, end: usize) -> Option<Parsed> {
        if self.nodes.len() >= self.limits.max_nodes {
            self.error(
                DiagnosticCode::NodeLimit,
                Token {
                    kind: TokenKind::Invalid,
                    start,
                    end,
                },
                vec![],
                "v4 node limit exceeded",
            );
            return None;
        }
        let id = NodeId(u32::try_from(self.nodes.len()).ok()?);
        let canonical_kind = remap_kind(kind, &self.canonical_classes);
        let canonical = *self.canonical_map.entry(canonical_kind).or_insert(id);
        self.canonical_classes.push(canonical);
        self.nodes
            .push(InfiniteNode::with_span(kind, checked_span(start, end)));
        Some(Parsed { id, start, end })
    }
    fn lower_derived(
        &mut self,
        strong: bool,
        interval: TemporalInterval,
        left: Parsed,
        right: Parsed,
    ) -> Option<Parsed> {
        if self.nodes.len().saturating_add(3) > self.limits.max_nodes {
            self.error(
                DiagnosticCode::NodeLimit,
                Token {
                    kind: TokenKind::Invalid,
                    start: left.start,
                    end: right.end,
                },
                vec![],
                "v4 lowering exceeds node limit",
            );
            return None;
        }
        let first = NodeId(u32::try_from(self.nodes.len()).ok()?);
        let kind = if strong {
            FutureKind::StrongRelease
        } else {
            FutureKind::WeakUntil
        };
        let generated = lower_infinite_future(
            kind,
            left.id,
            right.id,
            interval,
            first,
            Some(checked_span(left.start, right.end)),
        )
        .ok()?;
        for node in generated {
            self.push(node.kind, left.start, right.end)?;
        }
        Some(Parsed {
            id: NodeId(first.0.checked_add(2)?),
            start: left.start,
            end: right.end,
        })
    }
}

fn remap_kind(kind: Kind, classes: &[NodeId]) -> Kind {
    let class = |id: NodeId| classes[id.0 as usize];
    match kind {
        Kind::False | Kind::True | Kind::Proposition { .. } => kind,
        Kind::Not { operand } => Kind::Not {
            operand: class(operand),
        },
        Kind::And { left, right } => Kind::And {
            left: class(left),
            right: class(right),
        },
        Kind::Or { left, right } => Kind::Or {
            left: class(left),
            right: class(right),
        },
        Kind::Implies { left, right } => Kind::Implies {
            left: class(left),
            right: class(right),
        },
        Kind::Equivalent { left, right } => Kind::Equivalent {
            left: class(left),
            right: class(right),
        },
        Kind::Future { interval, operand } => Kind::Future {
            interval,
            operand: class(operand),
        },
        Kind::Globally { interval, operand } => Kind::Globally {
            interval,
            operand: class(operand),
        },
        Kind::Once { interval, operand } => Kind::Once {
            interval,
            operand: class(operand),
        },
        Kind::Historically { interval, operand } => Kind::Historically {
            interval,
            operand: class(operand),
        },
        Kind::StrongPrevious { operand } => Kind::StrongPrevious {
            operand: class(operand),
        },
        Kind::Until {
            interval,
            left,
            right,
        } => Kind::Until {
            interval,
            left: class(left),
            right: class(right),
        },
        Kind::Release {
            interval,
            left,
            right,
        } => Kind::Release {
            interval,
            left: class(left),
            right: class(right),
        },
        Kind::Since {
            interval,
            left,
            right,
        } => Kind::Since {
            interval,
            left: class(left),
            right: class(right),
        },
        Kind::Triggered {
            interval,
            left,
            right,
        } => Kind::Triggered {
            interval,
            left: class(left),
            right: class(right),
        },
    }
}

/// Formats one validated infinite graph with its optional ordered premises.
pub fn format_clean_ascii_v4(
    document: &InfiniteFormulaDocument,
    fairness: Option<&FairnessPremisesDocument>,
    limits: FormatLimits,
) -> FormatReport {
    let limits = limits.clamped();
    let mut formatter = InfiniteFormatter {
        document,
        limits,
        stats: FormatStats::default(),
        text: String::new(),
    };
    let result = (|| {
        if document.semantic_profile() != SemanticProfile::InfiniteTraceV1
            || document.clock() != InfiniteClock::EventPosition
        {
            return Err(format_error(
                FormatErrorCode::InvalidGraph,
                "v4 graph identity mismatch",
            ));
        }
        if let Some(fairness) = fairness {
            let identity = document
                .content_identity()
                .map_err(|error| format_error(FormatErrorCode::InvalidGraph, &error.to_string()))?;
            if fairness.graph_identity() != identity
                || fairness.clock() != document.clock()
                || fairness
                    .roots()
                    .iter()
                    .any(|root| document.formula().node(*root).is_none())
            {
                return Err(format_error(
                    FormatErrorCode::InvalidGraph,
                    "foreign fairness document",
                ));
            }
            if !fairness.roots().is_empty() {
                formatter.append("fair {")?;
                for (index, root) in fairness.roots().iter().enumerate() {
                    if index > 0 {
                        formatter.append("; ")?;
                    }
                    formatter.node(*root, 1)?;
                }
                formatter.append("}: ")?;
            }
        }
        formatter.node(document.root(), 1)
    })();
    match result {
        Ok(()) => {
            formatter.stats.output_bytes = formatter.text.len();
            FormatReport {
                limits,
                stats: formatter.stats,
                text: Some(formatter.text),
                error: None,
            }
        }
        Err(error) => FormatReport {
            limits,
            stats: formatter.stats,
            text: None,
            error: Some(error),
        },
    }
}

fn format_error(code: FormatErrorCode, message: &str) -> FormatError {
    FormatError {
        code,
        message: message.to_owned(),
    }
}

struct InfiniteFormatter<'a> {
    document: &'a InfiniteFormulaDocument,
    limits: FormatLimits,
    stats: FormatStats,
    text: String,
}

impl InfiniteFormatter<'_> {
    fn append(&mut self, chunk: &str) -> Result<(), FormatError> {
        let new_len = self.text.len().checked_add(chunk.len()).ok_or_else(|| {
            format_error(FormatErrorCode::OutputLimit, "v4 output length overflow")
        })?;
        if new_len > self.limits.max_output_bytes {
            return Err(format_error(
                FormatErrorCode::OutputLimit,
                "v4 output limit exceeded",
            ));
        }
        self.charge(chunk.len())?;
        self.text.push_str(chunk);
        Ok(())
    }
    fn charge(&mut self, units: usize) -> Result<(), FormatError> {
        let new_work =
            self.stats.work.checked_add(units).ok_or_else(|| {
                format_error(FormatErrorCode::WorkLimit, "v4 format work overflow")
            })?;
        if new_work > self.limits.max_work {
            return Err(format_error(
                FormatErrorCode::WorkLimit,
                "v4 format work limit exceeded",
            ));
        }
        self.stats.work = new_work;
        Ok(())
    }
    fn node(&mut self, id: NodeId, depth: usize) -> Result<(), FormatError> {
        if depth > 256 {
            return Err(format_error(
                FormatErrorCode::InvalidGraph,
                "v4 graph exceeds format depth",
            ));
        }
        self.charge(1)?;
        self.stats.nodes += 1;
        let kind = self
            .document
            .formula()
            .node(id)
            .ok_or_else(|| {
                format_error(FormatErrorCode::InvalidGraph, "v4 graph has missing node")
            })?
            .kind;
        match kind {
            Kind::False => self.append("false"),
            Kind::True => self.append("true"),
            Kind::Proposition { proposition } => self.append(&format!("p{}", proposition.0)),
            Kind::Not { operand } => {
                self.append("!")?;
                self.node(operand, depth + 1)
            }
            Kind::StrongPrevious { operand } => {
                self.append("Y")?;
                self.node(operand, depth + 1)
            }
            Kind::Future { interval, operand } => self.unary("F", interval, operand, depth),
            Kind::Globally { interval, operand } => self.unary("G", interval, operand, depth),
            Kind::Once { interval, operand } => self.unary("O", interval, operand, depth),
            Kind::Historically { interval, operand } => self.unary("H", interval, operand, depth),
            Kind::And { left, right } => self.binary(left, "&", right, depth),
            Kind::Or { left, right } => self.binary(left, "|", right, depth),
            Kind::Implies { left, right } => self.binary(left, "->", right, depth),
            Kind::Equivalent { left, right } => self.binary(left, "<->", right, depth),
            Kind::Until {
                interval,
                left,
                right,
            } => self.temporal(left, "U", interval, right, depth),
            Kind::Release {
                interval,
                left,
                right,
            } => self.temporal(left, "R", interval, right, depth),
            Kind::Since {
                interval,
                left,
                right,
            } => self.temporal(left, "S", interval, right, depth),
            Kind::Triggered {
                interval,
                left,
                right,
            } => self.temporal(left, "T", interval, right, depth),
        }
    }
    fn unary(
        &mut self,
        operator: &str,
        interval: TemporalInterval,
        operand: NodeId,
        depth: usize,
    ) -> Result<(), FormatError> {
        self.append(operator)?;
        self.append(&interval_text(interval))?;
        self.node(operand, depth + 1)
    }
    fn binary(
        &mut self,
        left: NodeId,
        operator: &str,
        right: NodeId,
        depth: usize,
    ) -> Result<(), FormatError> {
        self.append("(")?;
        self.node(left, depth + 1)?;
        self.append(" ")?;
        self.append(operator)?;
        self.append(" ")?;
        self.node(right, depth + 1)?;
        self.append(")")
    }
    fn temporal(
        &mut self,
        left: NodeId,
        operator: &str,
        interval: TemporalInterval,
        right: NodeId,
        depth: usize,
    ) -> Result<(), FormatError> {
        self.append("(")?;
        self.node(left, depth + 1)?;
        self.append(" ")?;
        self.append(operator)?;
        self.append(&interval_text(interval))?;
        self.append(" ")?;
        self.node(right, depth + 1)?;
        self.append(")")
    }
}

fn interval_text(interval: TemporalInterval) -> String {
    match interval {
        TemporalInterval::Closed(value) => format!("[{},{}]", value.start(), value.end()),
        TemporalInterval::Unbounded(value) => format!("[{},)", value.start()),
    }
}
