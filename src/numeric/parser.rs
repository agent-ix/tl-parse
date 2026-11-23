//! Bounded recursive-descent parser for `tl-parse.clean-ascii/v5` (FR-028, FR-029).
//!
//! Every recursive step passes through [`Parser::enter`], so formula and
//! arithmetic nesting share the 256 depth limit and recursion is bounded before
//! the stack is at risk (NFR-016). The first syntax or typing refusal stops the
//! parse: a refusal never cascades (FR-030).

use std::collections::BTreeMap;

use tl_syntax::{
    Formula, FormulaDocument, FutureKind, FutureLoweringRequest, Interval, Node, NodeId, NodeKind,
    PropositionId, RawBounds, SemanticProfile, SignalCatalogDocument, StrictDocumentReadError,
    SyntaxArtifactLimits, FUTURE_LOWERING_NODE_CHARGE, FUTURE_LOWERING_REQUEST_V1,
    FUTURE_OPERATORS_V1,
};

use super::{
    format::render_predicate,
    lexer::{lex, Keyword, Tok, Token},
    model::{build_table, type_arena, CatalogView, Cmp, Expr, Predicate},
    NumericDiagnosticCode as Code, NumericParseReport, Sink,
};
use crate::{lexer::checked_span, ParseLimits};

pub(crate) fn parse(
    source: &str,
    catalog: &SignalCatalogDocument,
    profile: SemanticProfile,
    limits: ParseLimits,
) -> NumericParseReport {
    let limits = limits.clamped();
    let mut report = NumericParseReport::empty(profile, catalog, limits, source.len());
    let mut sink = Sink::new(limits.max_diagnostics);
    if !matches!(
        profile,
        SemanticProfile::ClosedTraceV1 | SemanticProfile::OnlinePrefixV1
    ) {
        sink.push(
            Code::ProfileMismatch,
            0,
            0,
            format!(
                "v5 admits mltl.closed-trace/v1 and mltl.online-prefix/v1, not {}",
                profile.as_str()
            ),
        );
        return finish(report, sink);
    }
    let lexed = lex(source, limits, &mut sink);
    report.stats.tokens = lexed.tokens.len().saturating_sub(1);
    report.stats.work = lexed.work;
    if sink.has_error() || lexed.stopped {
        return finish(report, sink);
    }
    let view = CatalogView::new(catalog);
    // SPEC-GAP: FR-029 allocates from `1 + max(explicit, bindings)` and is
    // silent on an empty set; with neither, allocation starts at `p0`.
    let first_fresh = match (lexed.max_explicit_proposition, view.max_binding) {
        (None, None) => Some(0),
        (left, right) => left.max(right).and_then(|max| max.checked_add(1)),
    };
    let mut parser = Parser {
        source,
        profile,
        tokens: lexed.tokens,
        opens_comparison: lexed.opens_comparison,
        mixed_formula: lexed.mixed_formula,
        comparison_end: lexed.comparison_end,
        cursor: 0,
        nodes: Vec::new(),
        expression_nodes: 0,
        arena: Vec::new(),
        predicates: Vec::new(),
        predicate_spans: Vec::new(),
        by_text: BTreeMap::new(),
        next_fresh: first_fresh,
        limits,
        work: lexed.work,
        max_depth: 0,
        stopped: false,
        sink,
        view,
    };
    let root = parser.parse_formula(1, 1);
    if !parser.stopped {
        let token = parser.current();
        if token.kind != Tok::Eof {
            parser.fail::<()>(
                Code::TrailingInput,
                token.start,
                token.end,
                "unexpected input after a complete formula".to_owned(),
            );
        }
    }
    if root.is_none() && !parser.sink.has_error() {
        // Defensive: every `None` path records a refusal; never return a
        // report with neither a graph nor a typed refusal.
        parser.sink.push(
            Code::UnexpectedToken,
            0,
            0,
            "v5 parser stopped without a typed refusal".to_owned(),
        );
    }
    report.stats.nodes = parser.nodes.len().saturating_add(parser.expression_nodes);
    report.stats.work = parser.work;
    report.stats.max_depth = parser.max_depth;
    let Some(root) = root.filter(|_| !parser.stopped && !parser.sink.has_error()) else {
        return finish(report, parser.sink);
    };
    let (root_start, root_end) = (root.start, root.end);
    let nodes = std::mem::take(&mut parser.nodes);
    // SPEC-GAP: no `validation_failure` in the closed v5 set; an owner
    // construction refusal after a clean parse is reported as `node_limit`.
    let document = FormulaDocument::new(profile, root.node, nodes)
        .map_err(|error| (Code::NodeLimit, error.to_string()))
        .and_then(strictly_admit_document);
    let document = match document {
        Ok(document) => document,
        Err((code, message)) => {
            parser.sink.push(code, root_start, root_end, message);
            return finish(report, parser.sink);
        }
    };
    if !parser.predicates.is_empty() {
        match build_table(catalog, &parser.arena, &parser.predicates) {
            Ok(table) => report.predicate_table = Some(table),
            Err(code) => {
                parser.sink.push(
                    code,
                    root_start,
                    root_end,
                    "tl-syntax refused the predicate table".to_owned(),
                );
                return finish(report, parser.sink);
            }
        }
        report.predicate_spans = parser
            .predicate_spans
            .iter()
            .map(|(start, end)| checked_span(*start, *end))
            .collect();
    }
    report.document = Some(document);
    finish(report, parser.sink)
}

fn finish(mut report: NumericParseReport, sink: Sink) -> NumericParseReport {
    report.stats.diagnostics_truncated = sink.truncated;
    report.refusal = sink.refusal;
    report.diagnostics = sink.diagnostics;
    report.stats.diagnostics = report.diagnostics.len();
    if report.refusal.is_some() {
        report.document = None;
        report.predicate_table = None;
        report.predicate_spans = Vec::new();
    }
    report
}

/// Mirrors the v1 parser's second, real owner boundary over canonical bytes.
fn strictly_admit_document(
    document: FormulaDocument,
) -> Result<FormulaDocument, (Code, String)> {
    let bytes = document
        .canonical_json_bytes()
        .map_err(|error| (Code::NodeLimit, format!("canonical encoding failed: {error}")))?;
    match FormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default()) {
        Ok(admitted) if admitted == document => Ok(admitted),
        Ok(_) => Err((
            Code::NodeLimit,
            "strict admission changed the document".to_owned(),
        )),
        Err(error) => {
            let code = match error {
                StrictDocumentReadError::ResourceLimitExceeded {
                    resource: "formula depth",
                    ..
                }
                | StrictDocumentReadError::DepthLimitExceeded { .. } => Code::DepthLimit,
                StrictDocumentReadError::WorkLimitExceeded { .. } => Code::WorkLimit,
                // SPEC-GAP: the closed v5 set has no `validation_failure`; an
                // owner admission refusal is reported as the resource code
                // nearest to the owner's limits.
                _ => Code::NodeLimit,
            };
            Err((code, format!("strict admission failed: {error}")))
        }
    }
}

#[derive(Clone, Copy)]
struct Parsed {
    node: NodeId,
    start: usize,
    end: usize,
}

/// One arithmetic node in a comparison's local arena.
#[derive(Clone, Copy)]
struct Local {
    index: usize,
    start: usize,
    end: usize,
}

struct Parser<'a> {
    source: &'a str,
    profile: SemanticProfile,
    tokens: Vec<Token>,
    opens_comparison: Vec<bool>,
    mixed_formula: Vec<bool>,
    comparison_end: Vec<usize>,
    cursor: usize,
    nodes: Vec<Node>,
    expression_nodes: usize,
    arena: Vec<Expr>,
    predicates: Vec<Predicate>,
    predicate_spans: Vec<(usize, usize)>,
    by_text: BTreeMap<String, PropositionId>,
    next_fresh: Option<u32>,
    limits: ParseLimits,
    work: usize,
    max_depth: usize,
    stopped: bool,
    sink: Sink,
    view: CatalogView,
}

const fn binding_power(kind: Tok) -> Option<(u8, u8)> {
    match kind {
        Tok::Equivalent => Some((1, 2)),
        Tok::Implies => Some((2, 2)),
        Tok::Or => Some((3, 4)),
        Tok::And => Some((4, 5)),
        Tok::Keyword(
            Keyword::Until | Keyword::Release | Keyword::WeakUntil | Keyword::StrongRelease,
        ) => Some((5, 6)),
        _ => None,
    }
}

const fn begins_arithmetic(kind: Tok) -> bool {
    matches!(
        kind,
        Tok::Identifier
            | Tok::Number { .. }
            | Tok::Minus
            | Tok::Keyword(Keyword::Abs | Keyword::Scale)
    )
}

impl Parser<'_> {
    // ---------------------------------------------------------------- formula

    fn parse_formula(&mut self, minimum: u8, depth: usize) -> Option<Parsed> {
        if !self.enter(depth) {
            return None;
        }
        let mut left = self.parse_prefix(depth)?;
        loop {
            if self.stopped {
                return None;
            }
            let operator = self.current();
            if operator.kind == Tok::Keyword(Keyword::Unsupported) {
                // Binary past operators `S`/`T` in infix position.
                return self.fail(
                    Code::UnsupportedOperator,
                    operator.start,
                    operator.end,
                    "past operators are outside v5".to_owned(),
                );
            }
            let Some((left_power, right_power)) = binding_power(operator.kind) else {
                break;
            };
            if left_power < minimum {
                break;
            }
            self.advance();
            let interval = match operator.kind {
                Tok::Keyword(_) => Some(self.parse_interval()?),
                _ => None,
            };
            let right = self.parse_formula(right_power, depth.saturating_add(1))?;
            let (start, end) = (left.start, right.end);
            let kind = match (operator.kind, interval) {
                (Tok::Equivalent, _) => NodeKind::Equivalent {
                    left: left.node,
                    right: right.node,
                },
                (Tok::Implies, _) => NodeKind::Implies {
                    left: left.node,
                    right: right.node,
                },
                (Tok::Or, _) => NodeKind::Or {
                    left: left.node,
                    right: right.node,
                },
                (Tok::And, _) => NodeKind::And {
                    left: left.node,
                    right: right.node,
                },
                (Tok::Keyword(Keyword::Until), Some((interval, _))) => NodeKind::Until {
                    interval,
                    left: left.node,
                    right: right.node,
                },
                (Tok::Keyword(Keyword::Release), Some((interval, _))) => NodeKind::Release {
                    interval,
                    left: left.node,
                    right: right.node,
                },
                (Tok::Keyword(Keyword::WeakUntil), Some((interval, interval_end))) => {
                    left = self.lower(
                        FutureKind::WeakUntil,
                        interval,
                        (operator.start, interval_end),
                        left,
                        right,
                    )?;
                    continue;
                }
                (Tok::Keyword(Keyword::StrongRelease), Some((interval, interval_end))) => {
                    left = self.lower(
                        FutureKind::StrongRelease,
                        interval,
                        (operator.start, interval_end),
                        left,
                        right,
                    )?;
                    continue;
                }
                _ => return None,
            };
            left = self.push_formula_node(kind, start, end)?;
        }
        Some(left)
    }

    fn parse_prefix(&mut self, depth: usize) -> Option<Parsed> {
        if !self.enter(depth) {
            return None;
        }
        let token = self.current();
        match token.kind {
            Tok::Keyword(Keyword::True) => {
                self.advance();
                self.push_formula_node(NodeKind::True, token.start, token.end)
            }
            Tok::Keyword(Keyword::False) => {
                self.advance();
                self.push_formula_node(NodeKind::False, token.start, token.end)
            }
            Tok::Proposition(value) => {
                self.advance();
                self.push_formula_node(
                    NodeKind::Proposition {
                        proposition: PropositionId(value),
                    },
                    token.start,
                    token.end,
                )
            }
            Tok::Not => {
                self.advance();
                let operand = self.parse_prefix(depth.saturating_add(1))?;
                self.push_formula_node(
                    NodeKind::Not {
                        operand: operand.node,
                    },
                    token.start,
                    operand.end,
                )
            }
            Tok::Keyword(Keyword::Future | Keyword::Globally) => {
                self.advance();
                let (interval, _) = self.parse_interval()?;
                let operand = self.parse_prefix(depth.saturating_add(1))?;
                let kind = if token.kind == Tok::Keyword(Keyword::Future) {
                    NodeKind::Future {
                        interval,
                        operand: operand.node,
                    }
                } else {
                    NodeKind::Globally {
                        interval,
                        operand: operand.node,
                    }
                };
                self.push_formula_node(kind, token.start, operand.end)
            }
            Tok::LeftParenthesis => {
                if self.opens_comparison.get(self.cursor).copied().unwrap_or(false) {
                    return self.parse_comparison(depth);
                }
                self.advance();
                let inner = self.parse_formula(1, depth.saturating_add(1))?;
                let closing = self.current();
                if closing.kind != Tok::RightParenthesis {
                    return self.fail(
                        Code::MissingToken,
                        closing.start,
                        closing.end,
                        "expected `)`".to_owned(),
                    );
                }
                self.advance();
                Some(Parsed {
                    node: inner.node,
                    start: token.start,
                    end: closing.end,
                })
            }
            Tok::Keyword(Keyword::Unsupported) => self.fail(
                Code::UnsupportedOperator,
                token.start,
                token.end,
                "past, next and fairness operators are outside v5".to_owned(),
            ),
            // SPEC-GAP: set aggregation and contracts (FR-032) belong to
            // TL-424/TL-425 and are not implemented by TL-421; their keywords
            // are refused as `unsupported_operator` until then.
            Tok::Keyword(Keyword::Aggregation) => self.fail(
                Code::UnsupportedOperator,
                token.start,
                token.end,
                "set aggregation and contract forms are not yet supported".to_owned(),
            ),
            kind if begins_arithmetic(kind) => self.refuse_bare_arithmetic(depth),
            _ => self.fail(
                Code::UnexpectedToken,
                token.start,
                token.end,
                "expected a formula".to_owned(),
            ),
        }
    }

    /// Arithmetic at formula position: a bare comparison is
    /// `unparenthesized_comparison`; a bare expression lacks its comparison.
    fn refuse_bare_arithmetic(&mut self, depth: usize) -> Option<Parsed> {
        let start = self.current().start;
        let mut scratch = Vec::new();
        let mut spans = Vec::new();
        self.parse_arith(&mut scratch, &mut spans, depth)?;
        let next = self.current();
        if next.kind.is_comparison() {
            return self.fail(
                Code::UnparenthesizedComparison,
                start,
                next.end,
                "a comparison must be parenthesized".to_owned(),
            );
        }
        self.fail(
            Code::MissingToken,
            next.start,
            next.end,
            "expected a comparison operator".to_owned(),
        )
    }

    fn parse_interval(&mut self) -> Option<(Interval, usize)> {
        let opening = self.current();
        if opening.kind != Tok::LeftBracket {
            return self.fail(
                Code::MissingToken,
                opening.start,
                opening.end,
                "expected `[`".to_owned(),
            );
        }
        self.advance();
        let lower = self.interval_bound()?;
        let comma = self.current();
        if comma.kind != Tok::Comma {
            return self.fail(
                Code::MissingToken,
                comma.start,
                comma.end,
                "expected `,`".to_owned(),
            );
        }
        self.advance();
        let upper_token = self.current();
        if upper_token.kind == Tok::RightParenthesis {
            return self.fail(
                Code::UnsupportedOperator,
                opening.start,
                upper_token.end,
                "unbounded intervals are outside v5".to_owned(),
            );
        }
        let upper = self.interval_bound()?;
        let closing = self.current();
        if closing.kind != Tok::RightBracket {
            return self.fail(
                Code::MissingToken,
                closing.start,
                closing.end,
                "expected `]`".to_owned(),
            );
        }
        self.advance();
        match Interval::new(lower, upper) {
            Ok(interval) => Some((interval, closing.end)),
            // SPEC-GAP: FR-030's closed set has no `invalid_interval`.
            Err(_) => self.fail(
                Code::UnexpectedToken,
                opening.start,
                closing.end,
                "interval start exceeds its end".to_owned(),
            ),
        }
    }

    fn interval_bound(&mut self) -> Option<u32> {
        let token = self.current();
        if let Tok::Number {
            coefficient,
            scale: 0,
        } = token.kind
        {
            if let Ok(value) = u32::try_from(coefficient) {
                self.advance();
                return Some(value);
            }
            return self.fail(
                Code::IntegerOverflow,
                token.start,
                token.end,
                "interval bound is outside u32".to_owned(),
            );
        }
        self.fail(
            Code::UnexpectedToken,
            token.start,
            token.end,
            "expected an unsigned integer interval bound".to_owned(),
        )
    }

    // ------------------------------------------------------------- comparison

    fn parse_comparison(&mut self, depth: usize) -> Option<Parsed> {
        let opening = self.current();
        let opening_index = self.cursor;
        self.advance();
        if self.mixed_formula.get(opening_index).copied().unwrap_or(false) {
            // `(p1 & x > 0)`: a formula operator precedes the comparison at the
            // same depth, so the comparison itself lacks its parentheses.
            let first = self.current();
            let end = self
                .comparison_end
                .get(opening_index)
                .copied()
                .unwrap_or(first.end);
            return self.fail(
                Code::UnparenthesizedComparison,
                first.start,
                end,
                "a comparison inside a formula must have its own parentheses".to_owned(),
            );
        }
        let inner_depth = depth.saturating_add(1);
        let mut local: Vec<Expr> = Vec::new();
        let mut spans: Vec<(usize, usize)> = Vec::new();
        let left = self.parse_arith(&mut local, &mut spans, inner_depth)?;
        let operator_token = self.current();
        let operator = match operator_token.kind {
            Tok::Less => Cmp::Lt,
            Tok::LessEqual => Cmp::Le,
            Tok::Greater => Cmp::Gt,
            Tok::GreaterEqual => Cmp::Ge,
            Tok::EqualEqual => Cmp::Eq,
            Tok::NotEqual => Cmp::Ne,
            _ => {
                return self.fail(
                    Code::MissingToken,
                    operator_token.start,
                    operator_token.end,
                    "expected a comparison operator".to_owned(),
                )
            }
        };
        self.advance();
        let right = self.parse_arith(&mut local, &mut spans, inner_depth)?;
        let closing = self.current();
        if closing.kind.is_comparison() {
            return self.fail(
                Code::NonAssociativeComparison,
                operator_token.start,
                closing.end,
                "comparisons do not chain".to_owned(),
            );
        }
        if closing.kind != Tok::RightParenthesis {
            return self.fail(
                Code::MissingToken,
                closing.start,
                closing.end,
                "expected `)` closing the comparison".to_owned(),
            );
        }
        self.advance();
        let (start, end) = (opening.start, closing.end);
        // Typing per comparison gives each refusal the locus of its sub-expression.
        let types = match type_arena(&local, &self.view) {
            Ok(types) => types,
            Err((code, index)) => {
                let (node_start, node_end) = spans.get(index).copied().unwrap_or((start, end));
                return self.fail(code, node_start, node_end, format!("{code} in comparison"));
            }
        };
        let (left_scale, right_scale) = (types[left.index].scale, types[right.index].scale);
        if left_scale != right_scale {
            return self.fail(
                Code::ScaleMismatch,
                start,
                end,
                format!("comparison operands have scales {left_scale} and {right_scale}"),
            );
        }
        let candidate = Predicate {
            proposition: PropositionId(0),
            operator,
            left: left.index,
            right: right.index,
        };
        let Some(text) = render_predicate(&local, candidate, &self.view) else {
            return self.fail(
                Code::UnknownSignal,
                start,
                end,
                "comparison could not be rendered".to_owned(),
            );
        };
        let proposition = if let Some(existing) = self.by_text.get(&text) {
            *existing
        } else {
            let Some(fresh) = self.next_fresh else {
                return self.fail(
                    Code::PropositionOverflow,
                    start,
                    end,
                    "fresh proposition identity exceeds u32".to_owned(),
                );
            };
            // Only the arena of a new (table) predicate counts toward the node
            // limit; a repeated comparison reuses its identity and costs work only.
            let total = self
                .nodes
                .len()
                .saturating_add(self.expression_nodes)
                .saturating_add(local.len());
            if total > self.limits.max_nodes {
                return self.fail(
                    Code::NodeLimit,
                    start,
                    end,
                    format!("node count exceeds effective limit {}", self.limits.max_nodes),
                );
            }
            self.expression_nodes += local.len();
            self.next_fresh = fresh.checked_add(1);
            let proposition = PropositionId(fresh);
            let offset = self.arena.len();
            for node in &local {
                self.arena.push(shift(*node, offset));
            }
            self.predicates.push(Predicate {
                proposition,
                operator,
                left: left.index + offset,
                right: right.index + offset,
            });
            self.predicate_spans.push((start, end));
            self.by_text.insert(text, proposition);
            proposition
        };
        self.push_formula_node(NodeKind::Proposition { proposition }, start, end)
    }

    // ------------------------------------------------------------- arithmetic
    //
    // `arith`, `term` and `unary` share one depth; each parenthesis, call or
    // prefix minus adds one level, so expression nesting counts toward the
    // 256 limit one level per syntactic nesting.

    fn parse_arith(
        &mut self,
        local: &mut Vec<Expr>,
        spans: &mut Vec<(usize, usize)>,
        depth: usize,
    ) -> Option<Local> {
        if !self.enter(depth) {
            return None;
        }
        let mut left = self.parse_term(local, spans, depth)?;
        loop {
            let operator = self.current();
            if !matches!(operator.kind, Tok::Plus | Tok::Minus) {
                return Some(left);
            }
            self.advance();
            let right = self.parse_term(local, spans, depth)?;
            let node = if operator.kind == Tok::Plus {
                Expr::Add(left.index, right.index)
            } else {
                Expr::Sub(left.index, right.index)
            };
            left = self.push_expr(local, spans, node, left.start, right.end)?;
        }
    }

    fn parse_term(
        &mut self,
        local: &mut Vec<Expr>,
        spans: &mut Vec<(usize, usize)>,
        depth: usize,
    ) -> Option<Local> {
        if !self.enter(depth) {
            return None;
        }
        let mut left = self.parse_unary(local, spans, depth)?;
        while self.current().kind == Tok::Star {
            self.advance();
            let right = self.parse_unary(local, spans, depth)?;
            left = self.push_expr(
                local,
                spans,
                Expr::Mul(left.index, right.index),
                left.start,
                right.end,
            )?;
        }
        Some(left)
    }

    fn parse_unary(
        &mut self,
        local: &mut Vec<Expr>,
        spans: &mut Vec<(usize, usize)>,
        depth: usize,
    ) -> Option<Local> {
        if !self.enter(depth) {
            return None;
        }
        let token = self.current();
        if token.kind == Tok::Minus {
            self.advance();
            let operand = self.parse_unary(local, spans, depth.saturating_add(1))?;
            return self.push_expr(local, spans, Expr::Neg(operand.index), token.start, operand.end);
        }
        self.parse_primary(local, spans, depth)
    }

    fn parse_primary(
        &mut self,
        local: &mut Vec<Expr>,
        spans: &mut Vec<(usize, usize)>,
        depth: usize,
    ) -> Option<Local> {
        let token = self.current();
        match token.kind {
            Tok::Identifier => {
                self.advance();
                let name = &self.source[token.start..token.end];
                match self.view.lookup(name) {
                    Some((signal, _)) => {
                        self.push_expr(local, spans, Expr::Signal(signal), token.start, token.end)
                    }
                    None => self.fail(
                        Code::UnknownSignal,
                        token.start,
                        token.end,
                        format!("{name:?} names no catalog signal"),
                    ),
                }
            }
            Tok::Number { coefficient, scale } => {
                self.advance();
                let node = if scale == 0 {
                    Expr::Integer(coefficient)
                } else {
                    Expr::Decimal { coefficient, scale }
                };
                self.push_expr(local, spans, node, token.start, token.end)
            }
            Tok::Keyword(Keyword::Abs | Keyword::Scale)
                if self.peek(1).kind == Tok::LeftParenthesis =>
            {
                self.advance();
                self.advance();
                let operand = self.parse_arith(local, spans, depth.saturating_add(1))?;
                let node = if token.kind == Tok::Keyword(Keyword::Abs) {
                    Expr::Abs(operand.index)
                } else {
                    let comma = self.current();
                    if comma.kind != Tok::Comma {
                        return self.fail(
                            Code::MissingToken,
                            comma.start,
                            comma.end,
                            "expected `,` in scale(e, k)".to_owned(),
                        );
                    }
                    self.advance();
                    let target = self.current();
                    let Tok::Number {
                        coefficient,
                        scale: 0,
                    } = target.kind
                    else {
                        return self.fail(
                            Code::UnexpectedToken,
                            target.start,
                            target.end,
                            "scale target must be an unsigned integer".to_owned(),
                        );
                    };
                    if coefficient < 0 {
                        return self.fail(
                            Code::UnexpectedToken,
                            target.start,
                            target.end,
                            "scale target must be an unsigned integer".to_owned(),
                        );
                    }
                    let Some(target_scale) = u8::try_from(coefficient)
                        .ok()
                        .filter(|value| *value <= super::MAX_NUMERIC_SCALE)
                    else {
                        return self.fail(
                            Code::ScaleOverflow,
                            target.start,
                            target.end,
                            "scale target exceeds 18".to_owned(),
                        );
                    };
                    self.advance();
                    Expr::Rescale(operand.index, target_scale)
                };
                let closing = self.current();
                if closing.kind != Tok::RightParenthesis {
                    return self.fail(
                        Code::MissingToken,
                        closing.start,
                        closing.end,
                        "expected `)`".to_owned(),
                    );
                }
                self.advance();
                self.push_expr(local, spans, node, token.start, closing.end)
            }
            Tok::LeftParenthesis => {
                self.advance();
                let inner = self.parse_arith(local, spans, depth.saturating_add(1))?;
                let closing = self.current();
                if closing.kind.is_comparison() {
                    return self.fail(
                        Code::NonAssociativeComparison,
                        token.start,
                        closing.end,
                        "a comparison is not an arithmetic operand".to_owned(),
                    );
                }
                if closing.kind != Tok::RightParenthesis {
                    return self.fail(
                        Code::MissingToken,
                        closing.start,
                        closing.end,
                        "expected `)`".to_owned(),
                    );
                }
                self.advance();
                Some(Local {
                    index: inner.index,
                    start: token.start,
                    end: closing.end,
                })
            }
            Tok::Keyword(_) | Tok::Proposition(_) => self.fail(
                Code::ReservedIdentifier,
                token.start,
                token.end,
                "a reserved word or p<digits> cannot name a signal in v5".to_owned(),
            ),
            _ => self.fail(
                Code::UnexpectedToken,
                token.start,
                token.end,
                "expected an arithmetic operand".to_owned(),
            ),
        }
    }

    // ---------------------------------------------------------------- support

    fn push_expr(
        &mut self,
        local: &mut Vec<Expr>,
        spans: &mut Vec<(usize, usize)>,
        node: Expr,
        start: usize,
        end: usize,
    ) -> Option<Local> {
        if !self.charge_work(1, start, end) {
            return None;
        }
        local.push(node);
        spans.push((start, end));
        Some(Local {
            index: local.len() - 1,
            start,
            end,
        })
    }

    fn push_formula_node(&mut self, kind: NodeKind, start: usize, end: usize) -> Option<Parsed> {
        if !self.charge_node(start, end) {
            return None;
        }
        let Ok(id) = u32::try_from(self.nodes.len()) else {
            return self.fail(
                Code::NodeLimit,
                start,
                end,
                "node identity exceeds u32".to_owned(),
            );
        };
        self.nodes
            .push(Node::with_span(kind, checked_span(start, end)));
        Some(Parsed {
            node: NodeId(id),
            start,
            end,
        })
    }

    fn charge_node(&mut self, start: usize, end: usize) -> bool {
        if self.nodes.len().saturating_add(self.expression_nodes) >= self.limits.max_nodes {
            self.fail::<()>(
                Code::NodeLimit,
                start,
                end,
                format!("node count exceeds effective limit {}", self.limits.max_nodes),
            );
            return false;
        }
        self.charge_work(1, start, end)
    }

    /// Lowers `W`/`M` through the tl-syntax owner lowering, exactly as v2 does.
    fn lower(
        &mut self,
        kind: FutureKind,
        interval: Interval,
        (operator_start, operator_end): (usize, usize),
        left: Parsed,
        right: Parsed,
    ) -> Option<Parsed> {
        let (start, end) = (left.start, right.end);
        if self
            .nodes
            .len()
            .saturating_add(self.expression_nodes)
            .saturating_add(FUTURE_LOWERING_NODE_CHARGE)
            > self.limits.max_nodes
        {
            return self.fail(
                Code::NodeLimit,
                start,
                end,
                format!("node count exceeds effective limit {}", self.limits.max_nodes),
            );
        }
        if !self.charge_work(self.nodes.len(), start, end) {
            return None;
        }
        let bounds = (
            u32::try_from(self.nodes.len().saturating_sub(1)).ok(),
            raw_bounds(operator_start, operator_end),
            raw_bounds(start, end),
        );
        let (Some(root_index), Some(operator_span), Some(expression_span)) = bounds else {
            return self.fail(
                Code::NodeLimit,
                start,
                end,
                "node identity or source offset exceeds the tl-syntax wire range".to_owned(),
            );
        };
        let lowered = Formula::new(self.profile, NodeId(root_index), &self.nodes)
            .map_err(|error| error.to_string())
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
                    operator_span: Some(operator_span),
                    expression_span: Some(expression_span),
                }
                .lower()
                .map_err(|refusal| refusal.to_string())
            });
        match lowered {
            Ok(lowering) => {
                self.nodes.extend_from_slice(lowering.nodes());
                Some(Parsed {
                    node: lowering.root(),
                    start,
                    end,
                })
            }
            // SPEC-GAP: no `validation_failure` in the closed v5 set; an owner
            // lowering refusal after a valid parse can only be a resource cause.
            Err(message) => self.fail(Code::NodeLimit, start, end, message),
        }
    }

    fn enter(&mut self, depth: usize) -> bool {
        self.max_depth = self.max_depth.max(depth);
        if depth > self.limits.max_depth {
            let token = self.current();
            self.fail::<()>(
                Code::DepthLimit,
                token.start,
                token.end,
                format!("nesting exceeds effective limit {}", self.limits.max_depth),
            );
            return false;
        }
        let token = self.current();
        self.charge_work(1, token.start, token.end)
    }

    fn charge_work(&mut self, units: usize, start: usize, end: usize) -> bool {
        if self.stopped {
            return false;
        }
        if self.work.saturating_add(units) > self.limits.max_work {
            self.fail::<()>(
                Code::WorkLimit,
                start,
                end,
                format!("parser work exceeds effective limit {}", self.limits.max_work),
            );
            return false;
        }
        self.work += units;
        true
    }

    fn current(&self) -> Token {
        self.tokens[self.cursor.min(self.tokens.len().saturating_sub(1))]
    }

    fn peek(&self, ahead: usize) -> Token {
        self.tokens[self
            .cursor
            .saturating_add(ahead)
            .min(self.tokens.len().saturating_sub(1))]
    }

    fn advance(&mut self) {
        let token = self.current();
        if self.charge_work(1, token.start, token.end) && self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
    }

    /// Records one refusal and stops; returns `None` for `?` propagation.
    fn fail<T>(&mut self, code: Code, start: usize, end: usize, message: String) -> Option<T> {
        if !self.stopped {
            self.sink.push(code, start, end, message);
            self.stopped = true;
        }
        None
    }
}

const fn shift(node: Expr, offset: usize) -> Expr {
    match node {
        Expr::Signal(_) | Expr::Integer(_) | Expr::Decimal { .. } => node,
        Expr::Neg(operand) => Expr::Neg(operand + offset),
        Expr::Abs(operand) => Expr::Abs(operand + offset),
        Expr::Add(left, right) => Expr::Add(left + offset, right + offset),
        Expr::Sub(left, right) => Expr::Sub(left + offset, right + offset),
        Expr::Mul(left, right) => Expr::Mul(left + offset, right + offset),
        Expr::Rescale(operand, scale) => Expr::Rescale(operand + offset, scale),
    }
}

fn raw_bounds(start: usize, end: usize) -> Option<RawBounds> {
    Some(RawBounds::new(
        u64::try_from(start).ok()?,
        u64::try_from(end).ok()?,
    ))
}
