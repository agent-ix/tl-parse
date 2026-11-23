//! Canonical `tl-parse.clean-ascii/v5` formatter (FR-031).
//!
//! The temporal skeleton prints exactly as v1/v2 (no spaces, minimum
//! parentheses). A proposition defined by the predicate table prints as its
//! comparison `(left op right)` with single spaces around the comparison and
//! binary arithmetic operators and nested binary arithmetic fully
//! parenthesized; every other proposition prints as `p<id>`.
//!
//! Fixed point: the formatter refuses (`unrepresentable_graph`) any table whose
//! identities would not be reproduced by re-parsing the emitted text, so that
//! `parse(format(g))` yields the same graph, table and identities.

use std::collections::{BTreeMap, BTreeSet};

use tl_syntax::{FormulaDocument, NodeId, NodeKind, SemanticProfile, SignalCatalogDocument, SignalDomain};

use super::{
    model::{read_table, CatalogView, Expr, Predicate, PredicateTableDocument},
    MAX_NUMERIC_SCALE,
};
use crate::{
    FormatError, FormatErrorCode, FormatLimits, FormatReport, FormatStats, HARD_MAX_DEPTH,
    HARD_MAX_NODES,
};

/// Renders one comparison as canonical text, or `None` if a signal is absent.
pub(crate) fn render_predicate(
    arena: &[Expr],
    predicate: Predicate,
    catalog: &CatalogView,
) -> Option<String> {
    let mut text = String::from("(");
    render_expression(arena, predicate.left, catalog, false, 0, &mut text)?;
    text.push(' ');
    text.push_str(predicate.operator.spelling());
    text.push(' ');
    render_expression(arena, predicate.right, catalog, false, 0, &mut text)?;
    text.push(')');
    Some(text)
}

fn render_expression(
    arena: &[Expr],
    index: usize,
    catalog: &CatalogView,
    nested: bool,
    depth: usize,
    out: &mut String,
) -> Option<()> {
    if depth > HARD_MAX_DEPTH {
        return None;
    }
    let next = depth + 1;
    match *arena.get(index)? {
        Expr::Signal(id) => {
            let (name, domain) = catalog.name(id)?;
            if matches!(domain, SignalDomain::Boolean) {
                return None;
            }
            out.push_str(name);
        }
        Expr::Integer(value) => out.push_str(&value.to_string()),
        Expr::Decimal { coefficient, scale } => {
            if scale == 0 || scale > MAX_NUMERIC_SCALE {
                // A scale-0 decimal would re-parse as an integer node.
                return None;
            }
            out.push_str(&decimal_text(coefficient, scale));
        }
        Expr::Neg(operand) => {
            out.push('-');
            // `-5` would re-parse as a literal, `--x` as nested negation of a
            // literal; parenthesize every operand that is not a plain name or call.
            let plain = matches!(
                arena.get(operand),
                Some(Expr::Signal(_) | Expr::Abs(_) | Expr::Rescale(..))
            );
            if !plain {
                out.push('(');
            }
            render_expression(arena, operand, catalog, false, next, out)?;
            if !plain {
                out.push(')');
            }
        }
        Expr::Abs(operand) => {
            out.push_str("abs(");
            render_expression(arena, operand, catalog, false, next, out)?;
            out.push(')');
        }
        Expr::Rescale(operand, scale) => {
            out.push_str("scale(");
            render_expression(arena, operand, catalog, false, next, out)?;
            out.push_str(", ");
            out.push_str(&scale.to_string());
            out.push(')');
        }
        Expr::Add(left, right) | Expr::Sub(left, right) | Expr::Mul(left, right) => {
            let operator = match arena.get(index)? {
                Expr::Add(..) => " + ",
                Expr::Sub(..) => " - ",
                _ => " * ",
            };
            if nested {
                out.push('(');
            }
            render_expression(arena, left, catalog, true, next, out)?;
            out.push_str(operator);
            render_expression(arena, right, catalog, true, next, out)?;
            if nested {
                out.push(')');
            }
        }
    }
    Some(())
}

/// Exactly `scale` fractional digits, trailing zeros kept.
fn decimal_text(coefficient: i64, scale: u8) -> String {
    let magnitude = coefficient.unsigned_abs().to_string();
    let width = usize::from(scale) + 1;
    let padded = if magnitude.len() < width {
        format!("{}{magnitude}", "0".repeat(width - magnitude.len()))
    } else {
        magnitude
    };
    let split = padded.len() - usize::from(scale);
    let sign = if coefficient < 0 { "-" } else { "" };
    format!("{sign}{}.{}", &padded[..split], &padded[split..])
}

#[derive(Clone, Copy)]
enum Context {
    Root,
    Prefix,
    Binary { precedence: u8, group_equal: bool },
}

impl Context {
    fn groups(self, child: u8) -> bool {
        match self {
            Self::Root => false,
            Self::Prefix => child < 6,
            Self::Binary {
                precedence,
                group_equal,
            } => child < precedence || (group_equal && child == precedence),
        }
    }
}

enum Action {
    Node(NodeId, Context),
    Text(String),
}

fn refuse(limits: FormatLimits, stats: FormatStats, code: FormatErrorCode, message: &str) -> FormatReport {
    FormatReport {
        limits,
        stats,
        text: None,
        error: Some(FormatError {
            code,
            message: message.to_owned(),
        }),
    }
}

/// Spelling, precedence and right-associativity of a v1 skeleton node.
fn spelling(kind: NodeKind) -> Option<(String, u8, bool)> {
    let interval = |name: &str, interval: tl_syntax::Interval| {
        format!("{name}[{},{}]", interval.start(), interval.end())
    };
    Some(match kind {
        NodeKind::Not { .. } => ("!".to_owned(), 6, false),
        NodeKind::Future { interval: i, .. } => (interval("F", i), 6, false),
        NodeKind::Globally { interval: i, .. } => (interval("G", i), 6, false),
        NodeKind::And { .. } => ("&".to_owned(), 4, false),
        NodeKind::Or { .. } => ("|".to_owned(), 3, false),
        NodeKind::Implies { .. } => ("->".to_owned(), 2, true),
        NodeKind::Equivalent { .. } => ("<->".to_owned(), 1, false),
        NodeKind::Until { interval: i, .. } => (interval("U", i), 5, false),
        NodeKind::Release { interval: i, .. } => (interval("R", i), 5, false),
        _ => return None,
    })
}

/// Formats a validated graph, predicate table and catalog as canonical v5 text.
///
/// Refuses with `unrepresentable_graph` and no text: past nodes; a predicate
/// whose signal is absent from (or Boolean in) `catalog`; a proposition both
/// bound by `catalog` and defined by `table`; two structurally identical
/// predicates under different identities; and identities that a re-parse of
/// the emitted text would allocate differently (FR-029 allocation rule).
pub fn format_clean_ascii_v5(
    document: &FormulaDocument,
    table: Option<&PredicateTableDocument>,
    catalog: &SignalCatalogDocument,
    limits: FormatLimits,
) -> FormatReport {
    let limits = limits.clamped();
    let mut stats = FormatStats::default();
    if !matches!(
        document.semantic_profile(),
        SemanticProfile::ClosedTraceV1 | SemanticProfile::OnlinePrefixV1
    ) {
        return refuse(
            limits,
            stats,
            FormatErrorCode::UnrepresentableGraph,
            "v5 text exists only for mltl.closed-trace/v1 and mltl.online-prefix/v1",
        );
    }
    let formula = match document.validate() {
        Ok(formula) => formula,
        Err(error) => {
            return refuse(
                limits,
                stats,
                FormatErrorCode::InvalidGraph,
                &format!("validation failed: {error}"),
            )
        }
    };
    let view = CatalogView::new(catalog);
    let (arena, predicates) = match table.map(read_table) {
        None => (Vec::new(), Vec::new()),
        Some(Ok(parts)) => parts,
        Some(Err(message)) => {
            return refuse(limits, stats, FormatErrorCode::InvalidGraph, &message);
        }
    };
    let mut defined: BTreeMap<u32, String> = BTreeMap::new();
    let mut seen_text: BTreeSet<String> = BTreeSet::new();
    for predicate in &predicates {
        if view.bound.contains_key(&predicate.proposition.0) {
            return refuse(
                limits,
                stats,
                FormatErrorCode::UnrepresentableGraph,
                "a proposition is both bound by the catalog and defined by the table",
            );
        }
        let Some(text) = render_predicate(&arena, *predicate, &view) else {
            return refuse(
                limits,
                stats,
                FormatErrorCode::UnrepresentableGraph,
                "a predicate reads a signal absent from, or Boolean in, the catalog",
            );
        };
        if !seen_text.insert(text.clone()) {
            return refuse(
                limits,
                stats,
                FormatErrorCode::UnrepresentableGraph,
                "two identities define structurally identical predicates",
            );
        }
        defined.insert(predicate.proposition.0, text);
    }

    let mut text = String::new();
    let mut explicit_max: Option<u32> = None;
    let mut predicate_order: Vec<u32> = Vec::new();
    let mut predicate_seen: BTreeSet<u32> = BTreeSet::new();
    // Shared operands (from `W`/`M` lowering) print once per use, so the text
    // can describe a tree far larger than the graph. `visits` counts the nodes
    // a re-parse of the text would build; `reachable` is the graph's own count.
    let mut visits = arena.len();
    let mut reachable: BTreeSet<u32> = BTreeSet::new();
    let mut actions = vec![Action::Node(formula.root(), Context::Root)];
    while let Some(action) = actions.pop() {
        let piece = match action {
            Action::Text(piece) => piece,
            Action::Node(id, context) => {
                let Some(node) = formula.node(id) else {
                    return refuse(limits, stats, FormatErrorCode::InvalidGraph, "absent node");
                };
                reachable.insert(id.0);
                stats.nodes = reachable.len();
                visits = visits.saturating_add(1);
                if visits > HARD_MAX_NODES {
                    // SPEC-GAP (FR-031): a graph whose shared operands expand
                    // past the node limit has no text that re-parses to it.
                    return refuse(
                        limits,
                        stats,
                        FormatErrorCode::UnrepresentableGraph,
                        "the expanded text would exceed the parser node limit",
                    );
                }
                if let Err(error) = charge(&mut stats, 1, limits) {
                    return FormatReport {
                        limits,
                        stats,
                        text: None,
                        error: Some(error),
                    };
                }
                match node.kind {
                    NodeKind::False => "false".to_owned(),
                    NodeKind::True => "true".to_owned(),
                    NodeKind::Proposition { proposition } => {
                        if let Some(predicate) = defined.get(&proposition.0) {
                            if predicate_seen.insert(proposition.0) {
                                predicate_order.push(proposition.0);
                            }
                            predicate.clone()
                        } else {
                            explicit_max = Some(explicit_max.map_or(proposition.0, |max| {
                                max.max(proposition.0)
                            }));
                            format!("p{}", proposition.0)
                        }
                    }
                    kind => {
                        let Some((operator, precedence, right_associative)) = spelling(kind) else {
                            return refuse(
                                limits,
                                stats,
                                FormatErrorCode::UnrepresentableGraph,
                                "past nodes have no v5 text",
                            );
                        };
                        let grouped = context.groups(precedence);
                        if grouped {
                            actions.push(Action::Text(")".to_owned()));
                        }
                        match kind {
                            NodeKind::Not { operand }
                            | NodeKind::Future { operand, .. }
                            | NodeKind::Globally { operand, .. } => {
                                actions.push(Action::Node(operand, Context::Prefix));
                                if grouped {
                                    format!("({operator}")
                                } else {
                                    operator
                                }
                            }
                            NodeKind::And { left, right }
                            | NodeKind::Or { left, right }
                            | NodeKind::Implies { left, right }
                            | NodeKind::Equivalent { left, right }
                            | NodeKind::Until { left, right, .. }
                            | NodeKind::Release { left, right, .. } => {
                                actions.push(Action::Node(
                                    right,
                                    Context::Binary {
                                        precedence,
                                        group_equal: !right_associative,
                                    },
                                ));
                                actions.push(Action::Text(operator));
                                actions.push(Action::Node(
                                    left,
                                    Context::Binary {
                                        precedence,
                                        group_equal: right_associative,
                                    },
                                ));
                                if grouped {
                                    "(".to_owned()
                                } else {
                                    String::new()
                                }
                            }
                            _ => String::new(),
                        }
                    }
                }
            }
        };
        if let Err(error) = append(&mut text, &piece, limits, &mut stats) {
            return FormatReport {
                limits,
                stats,
                text: None,
                error: Some(error),
            };
        }
    }

    // Fixed-point check of the FR-029 allocation rule.
    let mut next = match (explicit_max, view.max_binding) {
        (None, None) => Some(0_u32),
        (left, right) => left.max(right).and_then(|max| max.checked_add(1)),
    };
    for identity in &predicate_order {
        if next != Some(*identity) {
            return refuse(
                limits,
                stats,
                FormatErrorCode::UnrepresentableGraph,
                "predicate identities differ from the v5 first-occurrence allocation",
            );
        }
        next = identity.checked_add(1);
    }
    if predicate_order.len() != predicates.len() {
        return refuse(
            limits,
            stats,
            FormatErrorCode::UnrepresentableGraph,
            "the table defines predicates the graph does not reference",
        );
    }
    stats.output_bytes = text.len();
    FormatReport {
        limits,
        stats,
        text: Some(text),
        error: None,
    }
}

fn append(
    output: &mut String,
    value: &str,
    limits: FormatLimits,
    stats: &mut FormatStats,
) -> Result<(), FormatError> {
    let length = output.len().saturating_add(value.len());
    if length > limits.max_output_bytes {
        return Err(FormatError {
            code: FormatErrorCode::OutputLimit,
            message: format!(
                "canonical text length {length} exceeds effective limit {}",
                limits.max_output_bytes
            ),
        });
    }
    charge(stats, value.len(), limits)?;
    output.push_str(value);
    Ok(())
}

fn charge(stats: &mut FormatStats, amount: usize, limits: FormatLimits) -> Result<(), FormatError> {
    let work = stats.work.saturating_add(amount);
    if work > limits.max_work {
        return Err(FormatError {
            code: FormatErrorCode::WorkLimit,
            message: format!("formatter work {work} exceeds effective limit {}", limits.max_work),
        });
    }
    stats.work = work;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::decimal_text;

    // Trace: TC-1549, FR-031-AC-1
    #[test]
    fn decimals_keep_exactly_their_scale() {
        assert_eq!(decimal_text(500, 2), "5.00");
        assert_eq!(decimal_text(-5, 2), "-0.05");
        assert_eq!(decimal_text(5, 1), "0.5");
        assert_eq!(decimal_text(0, 18), "0.000000000000000000");
        assert_eq!(decimal_text(i64::MIN, 18), "-9.223372036854775808");
    }
}
