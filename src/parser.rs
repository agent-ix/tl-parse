use tl_syntax::{
    FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile, SourceSpan,
};

use crate::{
    lexer::{checked_span, lex, Token, TokenKind},
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ExpectedToken, ParseLimits, ParseReport,
    RecoveryAction,
};

/// Parses one source string under the selected profile and effective limits.
///
/// Caller limits are clamped to process-safe hard maxima. Any diagnostic,
/// including a resource diagnostic, suppresses the formula document.
pub fn parse(source: &str, profile: SemanticProfile, limits: ParseLimits) -> ParseReport {
    let limits = limits.clamped();
    let lexed = lex(source, limits);
    let token_count = lexed.tokens.len().saturating_sub(1);
    let lex_stopped = lexed.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.code,
            DiagnosticCode::SourceLimit | DiagnosticCode::TokenLimit | DiagnosticCode::WorkLimit
        )
    });
    let mut parser = Parser {
        source,
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
            match FormulaDocument::new(profile, root, std::mem::take(&mut parser.nodes)) {
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
    report
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
    let start = limits.max_source_bytes;
    let end = start.saturating_add(1).min(source_bytes);
    let mut report = ParseReport::empty(profile, limits, source_bytes);
    report.diagnostics.push(Diagnostic {
        code: DiagnosticCode::SourceLimit,
        severity: DiagnosticSeverity::Error,
        span: SourceSpan::new(start as u32, end as u32)
            .expect("hard source limit and adjacent span fit u32"),
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
}

impl Parser<'_> {
    fn parse_expression(&mut self, minimum_binding_power: u8, depth: usize) -> Option<NodeId> {
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
                TokenKind::Until | TokenKind::Release => (5, 6),
                _ => break,
            };
            if left_power < minimum_binding_power {
                break;
            }
            self.advance();
            let interval = if matches!(operator.kind, TokenKind::Until | TokenKind::Release) {
                self.parse_interval()
            } else {
                None
            };
            if matches!(operator.kind, TokenKind::Until | TokenKind::Release) && interval.is_none()
            {
                return None;
            }
            let right = self.parse_expression(right_power, depth.saturating_add(1))?;
            let kind = match operator.kind {
                TokenKind::Equivalent => NodeKind::Equivalent { left, right },
                TokenKind::Implies => NodeKind::Implies { left, right },
                TokenKind::Or => NodeKind::Or { left, right },
                TokenKind::And => NodeKind::And { left, right },
                TokenKind::Until => NodeKind::Until {
                    interval: interval?,
                    left,
                    right,
                },
                TokenKind::Release => NodeKind::Release {
                    interval: interval?,
                    left,
                    right,
                },
                _ => return None,
            };
            let start = self.node_start(left);
            let end = self.node_end(right);
            left = self.push_node(kind, start, end)?;
        }
        Some(left)
    }

    fn parse_prefix(&mut self, depth: usize) -> Option<NodeId> {
        if !self.enter(depth) {
            return None;
        }
        let token = self.current();
        match token.kind {
            TokenKind::False => {
                self.advance();
                self.push_node(NodeKind::False, token.start, token.end)
            }
            TokenKind::True => {
                self.advance();
                self.push_node(NodeKind::True, token.start, token.end)
            }
            TokenKind::Proposition(proposition) => {
                self.advance();
                self.push_node(
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
                self.push_node(
                    NodeKind::Not { operand },
                    token.start,
                    self.node_end(operand),
                )
            }
            TokenKind::Future | TokenKind::Globally => {
                self.advance();
                let interval = self.parse_interval()?;
                let operand = self.parse_prefix(depth.saturating_add(1))?;
                let kind = match token.kind {
                    TokenKind::Future => NodeKind::Future { interval, operand },
                    TokenKind::Globally => NodeKind::Globally { interval, operand },
                    _ => return None,
                };
                self.push_node(kind, token.start, self.node_end(operand))
            }
            TokenKind::LeftParenthesis => {
                self.advance();
                let inner = self.parse_expression(1, depth.saturating_add(1))?;
                let closing = self.current();
                if closing.kind == TokenKind::RightParenthesis {
                    self.advance();
                    if let Some(node) = self.nodes.get_mut(inner.0 as usize) {
                        node.span = Some(checked_span(token.start, closing.end));
                    }
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
                }
                Some(inner)
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

    fn parse_interval(&mut self) -> Option<Interval> {
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
            Ok(interval) => Some(interval),
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

    fn current(&self) -> Token {
        self.tokens[self.cursor.min(self.tokens.len().saturating_sub(1))]
    }

    fn advance(&mut self) {
        if self.charge_work() && self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
    }

    fn node_start(&self, id: NodeId) -> usize {
        self.nodes[id.0 as usize]
            .span
            .map_or(0, |span| span.start() as usize)
    }

    fn node_end(&self, id: NodeId) -> usize {
        self.nodes[id.0 as usize]
            .span
            .map_or(0, |span| span.end() as usize)
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
