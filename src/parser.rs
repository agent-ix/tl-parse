use tl_syntax::{
    Formula, FormulaDocument, FutureKind, FutureLoweringRequest, Interval, Node, NodeId, NodeKind,
    PropositionId, RawBounds, SemanticProfile, SourceSpan, FUTURE_LOWERING_NODE_CHARGE,
    FUTURE_LOWERING_REQUEST_V1, FUTURE_OPERATORS_V1,
};

use crate::{
    lexer::{checked_span, lex, Dialect, Token, TokenKind},
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ExpectedToken, LoweringRecord, ParseLimits,
    ParseReport, RecoveryAction,
};

/// Parses one source string under the selected profile and effective limits.
///
/// Caller limits are clamped to process-safe hard maxima. Any diagnostic,
/// including a resource diagnostic, suppresses the formula document.
pub fn parse(source: &str, profile: SemanticProfile, limits: ParseLimits) -> ParseReport {
    parse_dialect(source, profile, limits, Dialect::V1).report
}

/// A v1-shaped report plus the lowering records a v2 parse produced.
pub(crate) struct DialectParse {
    pub(crate) report: ParseReport,
    pub(crate) lowerings: Vec<LoweringRecord>,
}

pub(crate) fn parse_dialect(
    source: &str,
    profile: SemanticProfile,
    limits: ParseLimits,
    dialect: Dialect,
) -> DialectParse {
    let limits = limits.clamped();
    let lexed = lex(source, limits, dialect);
    let token_count = lexed.tokens.len().saturating_sub(1);
    let lex_stopped = lexed.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code,
            DiagnosticCode::SourceLimit | DiagnosticCode::TokenLimit | DiagnosticCode::WorkLimit
        )
    });
    let mut parser = Parser {
        source,
        profile,
        limits,
        tokens: lexed.tokens,
        cursor: 0,
        nodes: Vec::new(),
        diagnostics: lexed.diagnostics,
        diagnostics_truncated: lexed.diagnostics_truncated,
        had_error: lexed.had_error,
        work: lexed.work,
        max_depth: 0,
        stopped: lex_stopped,
        lowerings: Vec::new(),
    };

    let root = if parser.stopped {
        None
    } else {
        parser.parse_expression(1, 1)
    };
    while parser.current().kind != TokenKind::Eof && !parser.stopped {
        if parser.current().kind == TokenKind::Invalid {
            parser.advance();
            continue;
        }
        let token = parser.current();
        parser.push_diagnostic(
            DiagnosticCode::TrailingInput,
            token,
            vec![ExpectedToken::EndOfInput],
            RecoveryAction::SkippedToken,
            format!(
                "unexpected trailing token {}; expected end of input",
                token.found(source)
            ),
        );
        parser.advance();
    }

    let mut report = ParseReport::empty(profile, limits, source.len());
    report.stats.tokens = token_count;
    report.stats.nodes = parser.nodes.len();
    report.stats.work = parser.work;
    report.stats.max_depth = parser.max_depth;
    report.stats.diagnostics_truncated = parser.diagnostics_truncated;

    if !parser.had_error && !parser.stopped {
        if let Some(root) = root {
            match FormulaDocument::new(profile, root.node, std::mem::take(&mut parser.nodes)) {
                Ok(document) => report.document = Some(document),
                Err(error) => {
                    let eof = parser.current();
                    parser.push_diagnostic(
                        DiagnosticCode::ValidationFailure,
                        eof,
                        Vec::new(),
                        RecoveryAction::Stopped,
                        format!("pinned tl-syntax validation failed: {error}"),
                    );
                }
            }
        }
    }

    report.diagnostics = parser.diagnostics;
    report.stats.diagnostics = report.diagnostics.len();
    report.stats.diagnostics_truncated = parser.diagnostics_truncated;
    if !report.diagnostics.is_empty() || parser.had_error || parser.stopped {
        report.document = None;
    }
    let lowerings = if report.document.is_some() {
        parser.lowerings
    } else {
        Vec::new()
    };
    DialectParse { report, lowerings }
}

// Trace: TC-021, FR-005-AC-3, NFR-001-AC-1
/// Builds the same fail-closed report as [`parse`] for an input whose full byte
/// count is known but whose contents were intentionally not retained.
///
/// Streaming front ends use this after counting an input beyond the effective
/// source limit, avoiding a fabricated replacement source and preserving the
/// submitted byte count.
pub fn source_limit_report(
    source_bytes: usize,
    profile: SemanticProfile,
    limits: ParseLimits,
) -> ParseReport {
    let limits = limits.clamped();
    let start = limits.max_source_bytes.min(source_bytes);
    let end = start.saturating_add(1).min(source_bytes);
    let mut report = ParseReport::empty(profile, limits, source_bytes);
    let Ok(span) = SourceSpan::new(start as u32, end as u32) else {
        // The clamped offsets above are ordered and bounded by the hard u32
        // source limit. If those invariants ever change, preserve a total,
        // fail-closed report instead of panicking in this public helper.
        return report;
    };
    report.diagnostics.push(Diagnostic {
        code: DiagnosticCode::SourceLimit,
        severity: DiagnosticSeverity::Error,
        span,
        found: "<source>".to_owned(),
        expected: Vec::new(),
        recovery: RecoveryAction::Stopped,
        message: format!(
            "source length {source_bytes} exceeds effective limit {}",
            limits.max_source_bytes
        ),
    });
    report.stats.diagnostics = 1;
    report
}

struct Parser<'a> {
    source: &'a str,
    profile: SemanticProfile,
    limits: ParseLimits,
    tokens: Vec<Token>,
    cursor: usize,
    nodes: Vec<Node>,
    diagnostics: Vec<Diagnostic>,
    diagnostics_truncated: bool,
    had_error: bool,
    work: usize,
    max_depth: usize,
    stopped: bool,
    lowerings: Vec<LoweringRecord>,
}

#[derive(Clone, Copy)]
struct Parsed {
    node: NodeId,
    extent_start: usize,
    extent_end: usize,
}

impl Parser<'_> {
    fn parse_expression(&mut self, minimum_binding_power: u8, depth: usize) -> Option<Parsed> {
        if !self.enter(depth) {
            return None;
        }
        let mut left = self.parse_prefix(depth)?;
        loop {
            if self.stopped {
                return None;
            }
            let operator = self.current();
            let (left_power, right_power) = match operator.kind {
                TokenKind::Equivalent => (1, 2),
                TokenKind::Implies => (2, 2),
                TokenKind::Or => (3, 4),
                TokenKind::And => (4, 5),
                TokenKind::Until
                | TokenKind::Release
                | TokenKind::WeakUntil
                | TokenKind::StrongRelease => (5, 6),
                _ => break,
            };
            if left_power < minimum_binding_power {
                break;
            }
            self.advance();
            let temporal = matches!(
                operator.kind,
                TokenKind::Until
                    | TokenKind::Release
                    | TokenKind::WeakUntil
                    | TokenKind::StrongRelease
            );
            let bracketed = if temporal {
                Some(self.parse_interval()?)
            } else {
                None
            };
            let interval = bracketed.map(|(interval, _)| interval);
            let right = self.parse_expression(right_power, depth.saturating_add(1))?;
            let future_kind = match operator.kind {
                TokenKind::WeakUntil => Some(FutureKind::WeakUntil),
                TokenKind::StrongRelease => Some(FutureKind::StrongRelease),
                _ => None,
            };
            if let (Some(future_kind), Some((interval, interval_end))) = (future_kind, bracketed) {
                let operator_span = (operator.start, interval_end);
                left = self.lower_future(future_kind, interval, operator_span, left, right)?;
                continue;
            }
            let kind = match operator.kind {
                TokenKind::Equivalent => NodeKind::Equivalent {
                    left: left.node,
                    right: right.node,
                },
                TokenKind::Implies => NodeKind::Implies {
                    left: left.node,
                    right: right.node,
                },
                TokenKind::Or => NodeKind::Or {
                    left: left.node,
                    right: right.node,
                },
                TokenKind::And => NodeKind::And {
                    left: left.node,
                    right: right.node,
                },
                TokenKind::Until => NodeKind::Until {
                    interval: interval?,
                    left: left.node,
                    right: right.node,
                },
                TokenKind::Release => NodeKind::Release {
                    interval: interval?,
                    left: left.node,
                    right: right.node,
                },
                _ => return None,
            };
            left = self.push_parsed_node(kind, left.extent_start, right.extent_end)?;
        }
        Some(left)
    }

    fn parse_prefix(&mut self, depth: usize) -> Option<Parsed> {
        if !self.enter(depth) {
            return None;
        }
        let token = self.current();
        match token.kind {
            TokenKind::False => {
                self.advance();
                self.push_parsed_node(NodeKind::False, token.start, token.end)
            }
            TokenKind::True => {
                self.advance();
                self.push_parsed_node(NodeKind::True, token.start, token.end)
            }
            TokenKind::Proposition(proposition) => {
                self.advance();
                self.push_parsed_node(
                    NodeKind::Proposition {
                        proposition: PropositionId(proposition),
                    },
                    token.start,
                    token.end,
                )
            }
            TokenKind::Not => {
                self.advance();
                let operand = self.parse_prefix(depth.saturating_add(1))?;
                self.push_parsed_node(
                    NodeKind::Not {
                        operand: operand.node,
                    },
                    token.start,
                    operand.extent_end,
                )
            }
            TokenKind::Future | TokenKind::Globally => {
                self.advance();
                let (interval, _) = self.parse_interval()?;
                let operand = self.parse_prefix(depth.saturating_add(1))?;
                let kind = match token.kind {
                    TokenKind::Future => NodeKind::Future {
                        interval,
                        operand: operand.node,
                    },
                    TokenKind::Globally => NodeKind::Globally {
                        interval,
                        operand: operand.node,
                    },
                    _ => return None,
                };
                self.push_parsed_node(kind, token.start, operand.extent_end)
            }
            TokenKind::LeftParenthesis => {
                self.advance();
                let inner = self.parse_expression(1, depth.saturating_add(1))?;
                let closing = self.current();
                let extent_end = if closing.kind == TokenKind::RightParenthesis {
                    self.advance();
                    closing.end
                } else {
                    self.push_diagnostic(
                        DiagnosticCode::MissingToken,
                        closing,
                        vec![ExpectedToken::RightParenthesis],
                        RecoveryAction::InsertedToken,
                        format!(
                            "missing closing parenthesis before {}",
                            closing.found(self.source)
                        ),
                    );
                    inner.extent_end
                };
                Some(Parsed {
                    node: inner.node,
                    extent_start: token.start,
                    extent_end,
                })
            }
            TokenKind::Invalid => {
                self.advance();
                None
            }
            _ => {
                self.push_diagnostic(
                    DiagnosticCode::UnexpectedToken,
                    token,
                    vec![ExpectedToken::Expression],
                    if token.kind == TokenKind::Eof {
                        RecoveryAction::InsertedToken
                    } else {
                        RecoveryAction::SkippedToken
                    },
                    format!("expected expression, found {}", token.found(self.source)),
                );
                if token.kind != TokenKind::Eof {
                    self.advance();
                }
                None
            }
        }
    }

    /// Parses a bracketed interval, returning it with its closing-bracket end.
    fn parse_interval(&mut self) -> Option<(Interval, usize)> {
        let opening = self.current();
        if opening.kind != TokenKind::LeftBracket {
            self.push_diagnostic(
                DiagnosticCode::MissingToken,
                opening,
                vec![ExpectedToken::LeftBracket],
                RecoveryAction::InsertedToken,
                format!(
                    "expected interval opening bracket, found {}",
                    opening.found(self.source)
                ),
            );
            return None;
        }
        self.advance();
        let start_token = self.current();
        let start = if let TokenKind::Number(value) = start_token.kind {
            self.advance();
            value
        } else {
            self.push_diagnostic(
                DiagnosticCode::UnexpectedToken,
                start_token,
                vec![ExpectedToken::Integer],
                RecoveryAction::SkippedToken,
                format!(
                    "expected interval start integer, found {}",
                    start_token.found(self.source)
                ),
            );
            if start_token.kind != TokenKind::Eof {
                self.advance();
            }
            return None;
        };
        let comma = self.current();
        if comma.kind != TokenKind::Comma {
            self.push_diagnostic(
                DiagnosticCode::MissingToken,
                comma,
                vec![ExpectedToken::Comma],
                RecoveryAction::InsertedToken,
                format!(
                    "expected interval comma, found {}",
                    comma.found(self.source)
                ),
            );
            return None;
        }
        self.advance();
        let end_token = self.current();
        let end = if let TokenKind::Number(value) = end_token.kind {
            self.advance();
            value
        } else {
            self.push_diagnostic(
                DiagnosticCode::UnexpectedToken,
                end_token,
                vec![ExpectedToken::Integer],
                RecoveryAction::SkippedToken,
                format!(
                    "expected interval end integer, found {}",
                    end_token.found(self.source)
                ),
            );
            if end_token.kind != TokenKind::Eof {
                self.advance();
            }
            return None;
        };
        let closing = self.current();
        if closing.kind != TokenKind::RightBracket {
            self.push_diagnostic(
                DiagnosticCode::MissingToken,
                closing,
                vec![ExpectedToken::RightBracket],
                RecoveryAction::InsertedToken,
                format!(
                    "expected interval closing bracket, found {}",
                    closing.found(self.source)
                ),
            );
            return None;
        }
        self.advance();
        match Interval::new(start, end) {
            Ok(interval) => Some((interval, closing.end)),
            Err(error) => {
                self.push_diagnostic(
                    DiagnosticCode::InvalidInterval,
                    Token {
                        kind: TokenKind::Invalid,
                        start: opening.start,
                        end: closing.end,
                    },
                    Vec::new(),
                    RecoveryAction::None,
                    error.to_string(),
                );
                None
            }
        }
    }

    /// Lowers one derived expression through tl-syntax and appends its nodes.
    ///
    /// The three-node charge is checked before anything is appended, and
    /// validating the borrowed formula charges one work unit per existing node.
    fn lower_future(
        &mut self,
        kind: FutureKind,
        interval: Interval,
        (operator_start, operator_end): (usize, usize),
        left: Parsed,
        right: Parsed,
    ) -> Option<Parsed> {
        let expression = Token {
            kind: TokenKind::Invalid,
            start: left.extent_start,
            end: right.extent_end,
        };
        if self.nodes.len().saturating_add(FUTURE_LOWERING_NODE_CHARGE) > self.limits.max_nodes {
            self.push_diagnostic(
                DiagnosticCode::NodeLimit,
                expression,
                Vec::new(),
                RecoveryAction::Stopped,
                format!(
                    "formula node count exceeds effective limit {}",
                    self.limits.max_nodes
                ),
            );
            self.stopped = true;
            return None;
        }
        if !self.charge_work_units(self.nodes.len(), expression) {
            return None;
        }
        let root = NodeId(self.nodes.len().saturating_sub(1) as u32);
        let lowered = Formula::new(self.profile, root, &self.nodes)
            .map_err(|error| format!("pinned tl-syntax validation failed: {error}"))
            .and_then(|formula| {
                FutureLoweringRequest {
                    request_identity: FUTURE_LOWERING_REQUEST_V1.as_bytes(),
                    operator_profile: FUTURE_OPERATORS_V1.as_bytes(),
                    kind: kind.as_str().as_bytes(),
                    semantic_profile: self.profile.as_str().as_bytes(),
                    formula,
                    left: u64::from(left.node.0),
                    right: u64::from(right.node.0),
                    interval: Some(RawBounds::new(
                        u64::from(interval.start()),
                        u64::from(interval.end()),
                    )),
                    operator_span: Some(RawBounds::new(operator_start as u64, operator_end as u64)),
                    expression_span: Some(RawBounds::new(
                        left.extent_start as u64,
                        right.extent_end as u64,
                    )),
                }
                .lower()
                .map_err(|refusal| format!("pinned tl-syntax lowering refused: {refusal}"))
            });
        let lowering = match lowered {
            Ok(lowering) => lowering,
            Err(message) => {
                self.push_diagnostic(
                    DiagnosticCode::ValidationFailure,
                    expression,
                    Vec::new(),
                    RecoveryAction::Stopped,
                    message,
                );
                self.stopped = true;
                return None;
            }
        };
        self.nodes.extend_from_slice(lowering.nodes());
        let report = lowering.report();
        self.lowerings.push(LoweringRecord {
            kind: kind.into(),
            operator_span: checked_span(operator_start, operator_end),
            expression_span: checked_span(left.extent_start, right.extent_end),
            left: report.left(),
            right: report.right(),
            first_generated: report.first_generated(),
            root: report.root(),
        });
        Some(Parsed {
            node: lowering.root(),
            extent_start: left.extent_start,
            extent_end: right.extent_end,
        })
    }

    fn push_node(&mut self, kind: NodeKind, start: usize, end: usize) -> Option<NodeId> {
        if self.nodes.len() >= self.limits.max_nodes {
            let token = Token {
                kind: TokenKind::Invalid,
                start,
                end,
            };
            self.push_diagnostic(
                DiagnosticCode::NodeLimit,
                token,
                Vec::new(),
                RecoveryAction::Stopped,
                format!(
                    "formula node count exceeds effective limit {}",
                    self.limits.max_nodes
                ),
            );
            self.stopped = true;
            return None;
        }
        let id = NodeId(self.nodes.len() as u32);
        self.nodes
            .push(Node::with_span(kind, checked_span(start, end)));
        Some(id)
    }

    fn push_parsed_node(&mut self, kind: NodeKind, start: usize, end: usize) -> Option<Parsed> {
        let node = self.push_node(kind, start, end)?;
        Some(Parsed {
            node,
            extent_start: start,
            extent_end: end,
        })
    }

    fn enter(&mut self, depth: usize) -> bool {
        self.max_depth = self.max_depth.max(depth);
        if depth > self.limits.max_depth {
            let token = self.current();
            self.push_diagnostic(
                DiagnosticCode::DepthLimit,
                token,
                Vec::new(),
                RecoveryAction::Stopped,
                format!(
                    "expression depth exceeds effective limit {}",
                    self.limits.max_depth
                ),
            );
            self.stopped = true;
            return false;
        }
        self.charge_work()
    }

    fn charge_work(&mut self) -> bool {
        if self.work >= self.limits.max_work {
            let token = self.current();
            self.push_diagnostic(
                DiagnosticCode::WorkLimit,
                token,
                Vec::new(),
                RecoveryAction::Stopped,
                format!(
                    "lexer/parser work exceeds effective limit {}",
                    self.limits.max_work
                ),
            );
            self.stopped = true;
            false
        } else {
            self.work += 1;
            true
        }
    }

    /// Charges `units` of work, refusing at `token` (the offending expression).
    fn charge_work_units(&mut self, units: usize, token: Token) -> bool {
        if self.work.saturating_add(units) > self.limits.max_work {
            self.push_diagnostic(
                DiagnosticCode::WorkLimit,
                token,
                Vec::new(),
                RecoveryAction::Stopped,
                format!(
                    "lexer/parser work exceeds effective limit {}",
                    self.limits.max_work
                ),
            );
            self.stopped = true;
            false
        } else {
            self.work += units;
            true
        }
    }

    fn current(&self) -> Token {
        self.tokens[self.cursor.min(self.tokens.len().saturating_sub(1))]
    }

    fn advance(&mut self) {
        if self.charge_work() && self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
    }

    fn push_diagnostic(
        &mut self,
        code: DiagnosticCode,
        token: Token,
        expected: Vec<ExpectedToken>,
        recovery: RecoveryAction,
        message: String,
    ) {
        self.had_error = true;
        if self.diagnostics.len() >= self.limits.max_diagnostics {
            self.diagnostics_truncated = true;
            return;
        }
        self.diagnostics.push(Diagnostic {
            code,
            severity: DiagnosticSeverity::Error,
            span: token.span(),
            found: token.found(self.source),
            expected,
            recovery,
            message,
        });
    }
}
