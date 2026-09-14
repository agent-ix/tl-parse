use tl_syntax::{
    Formula, FormulaDocument, FormulaSchemaVersion, Interval, NodeId, NodeKind, SemanticProfile,
};

use crate::{
    dialect::Dialect, FormatError, FormatErrorCode, FormatLimits, FormatReport, FormatStats,
};

/// Validates and canonically formats an owned formula document.
pub fn format_document(document: &FormulaDocument, limits: FormatLimits) -> FormatReport {
    let dialect = match (document.schema_version(), document.semantic_profile()) {
        (_, SemanticProfile::ClosedTraceV1) | (_, SemanticProfile::OnlinePrefixV1) => Dialect::V1,
        (FormulaSchemaVersion::V2, SemanticProfile::OriginCompleteHistoryV1) => Dialect::V3,
        _ => {
            return failed_report(
                limits.clamped(),
                FormatStats::default(),
                FormatErrorCode::InvalidGraph,
                "formula schema/profile has no matching clean-ASCII dialect".to_owned(),
            )
        }
    };
    match document.validate() {
        Ok(formula) => format_formula_for_dialect(formula, limits, dialect),
        Err(error) => failed_report(
            limits.clamped(),
            FormatStats::default(),
            FormatErrorCode::InvalidGraph,
            format!("pinned tl-syntax validation failed: {error}"),
        ),
    }
}

/// Canonically formats an origin-complete formula-v2 document as clean-ascii/v3.
pub fn format_clean_ascii_v3(document: &FormulaDocument, limits: FormatLimits) -> FormatReport {
    if document.schema_version() != FormulaSchemaVersion::V2
        || document.semantic_profile() != SemanticProfile::OriginCompleteHistoryV1
    {
        return failed_report(
            limits.clamped(),
            FormatStats::default(),
            FormatErrorCode::InvalidGraph,
            "clean-ascii/v3 requires formula-v2 with mltl.origin-complete-history/v1".to_owned(),
        );
    }
    match document.validate() {
        Ok(formula) => format_formula_for_dialect(formula, limits, Dialect::V3),
        Err(error) => failed_report(
            limits.clamped(),
            FormatStats::default(),
            FormatErrorCode::InvalidGraph,
            format!("pinned tl-syntax validation failed: {error}"),
        ),
    }
}

/// Canonically formats a validated formula without recursive graph traversal.
pub fn format_formula(formula: Formula<'_>, limits: FormatLimits) -> FormatReport {
    let dialect = Dialect::for_formula_profile(formula.profile());
    format_formula_for_dialect(formula, limits, dialect)
}

fn format_formula_for_dialect(
    formula: Formula<'_>,
    limits: FormatLimits,
    dialect: Dialect,
) -> FormatReport {
    let limits = limits.clamped();
    let nodes = formula.nodes();
    let mut reachable = vec![false; nodes.len()];
    let mut stack = vec![formula.root()];
    while let Some(id) = stack.pop() {
        let Ok(index) = usize::try_from(id.0) else {
            return failed_report(
                limits.clamped(),
                FormatStats::default(),
                FormatErrorCode::InvalidGraph,
                format!("formula node {} cannot be indexed on this target", id.0),
            );
        };
        if reachable.get(index).copied().unwrap_or(false) {
            continue;
        }
        let Some(node) = formula.node(id) else {
            return failed_report(
                limits,
                FormatStats::default(),
                FormatErrorCode::InvalidGraph,
                format!("formula references absent node {}", id.0),
            );
        };
        reachable[index] = true;
        for operand in operands(node.kind).into_iter().flatten() {
            stack.push(operand);
        }
    }

    let mut stats = FormatStats {
        nodes: reachable.iter().filter(|item| **item).count(),
        ..FormatStats::default()
    };
    let mut text = String::new();
    let mut actions = vec![Action::Node(formula.root(), Context::Root)];
    while let Some(action) = actions.pop() {
        let result = match action {
            Action::Static(value) => append(&mut text, value, limits, &mut stats),
            Action::Owned(value) => append(&mut text, &value, limits, &mut stats),
            Action::Node(id, context) => {
                let Some(node) = formula.node(id) else {
                    return failed_report(
                        limits,
                        stats,
                        FormatErrorCode::InvalidGraph,
                        format!("formula references absent node {}", id.0),
                    );
                };
                if let Err(error) = charge(&mut stats, 1, limits) {
                    Err(error)
                } else {
                    let Some(node_precedence) = dialect.precedence(node.kind) else {
                        return failed_report(
                            limits,
                            stats,
                            FormatErrorCode::InvalidGraph,
                            "formula node is outside the selected clean-ASCII dialect".to_owned(),
                        );
                    };
                    let grouped = context.groups(node_precedence);
                    if grouped {
                        if let Err(error) = append(&mut text, "(", limits, &mut stats) {
                            return FormatReport {
                                limits,
                                stats,
                                text: None,
                                error: Some(error),
                            };
                        }
                        actions.push(Action::Static(")"));
                    }
                    if push_node_actions(&mut actions, dialect, node.kind) {
                        Ok(())
                    } else {
                        Err(FormatError {
                            code: FormatErrorCode::InvalidGraph,
                            message: "formula node is outside the selected clean-ASCII dialect"
                                .to_owned(),
                        })
                    }
                }
            }
        };
        if let Err(error) = result {
            return FormatReport {
                limits,
                stats,
                text: None,
                error: Some(error),
            };
        }
    }
    stats.output_bytes = text.len();
    FormatReport {
        limits,
        stats,
        text: Some(text),
        error: None,
    }
}

#[derive(Clone, Copy)]
enum Context {
    Root,
    Prefix,
    Binary { precedence: u8, group_equal: bool },
}

impl Context {
    fn groups(self, child_precedence: u8) -> bool {
        match self {
            Self::Root => false,
            Self::Prefix => child_precedence < 6,
            Self::Binary {
                precedence,
                group_equal,
            } => child_precedence < precedence || (group_equal && child_precedence == precedence),
        }
    }
}

enum Action {
    Node(NodeId, Context),
    Static(&'static str),
    Owned(String),
}

fn append(
    output: &mut String,
    value: &str,
    limits: FormatLimits,
    stats: &mut FormatStats,
) -> Result<(), FormatError> {
    let length = output
        .len()
        .checked_add(value.len())
        .ok_or_else(|| FormatError {
            code: FormatErrorCode::OutputLimit,
            message: "canonical text length overflowed".to_owned(),
        })?;
    if length > limits.max_output_bytes {
        return Err(FormatError {
            code: FormatErrorCode::OutputLimit,
            message: format!(
                "canonical text length {} exceeds effective limit {}",
                length, limits.max_output_bytes
            ),
        });
    }
    charge(stats, value.len(), limits)?;
    output.push_str(value);
    Ok(())
}

fn charge(stats: &mut FormatStats, amount: usize, limits: FormatLimits) -> Result<(), FormatError> {
    let Some(work) = stats.work.checked_add(amount) else {
        return Err(FormatError {
            code: FormatErrorCode::WorkLimit,
            message: "formatter work counter overflowed".to_owned(),
        });
    };
    if work > limits.max_work {
        return Err(FormatError {
            code: FormatErrorCode::WorkLimit,
            message: format!(
                "formatter work {} exceeds effective limit {}",
                work, limits.max_work
            ),
        });
    }
    stats.work = work;
    Ok(())
}

fn binary_actions(
    actions: &mut Vec<Action>,
    left: NodeId,
    operator: &'static str,
    right: NodeId,
    precedence: u8,
    right_associative: bool,
) {
    actions.push(Action::Node(
        right,
        Context::Binary {
            precedence,
            group_equal: !right_associative,
        },
    ));
    actions.push(Action::Static(operator));
    actions.push(Action::Node(
        left,
        Context::Binary {
            precedence,
            group_equal: right_associative,
        },
    ));
}

fn interval_operator(operator: &str, interval: Interval) -> String {
    format!("{operator}[{},{}]", interval.start(), interval.end())
}

fn push_node_actions(actions: &mut Vec<Action>, dialect: Dialect, kind: NodeKind) -> bool {
    match kind {
        NodeKind::False => {
            actions.push(Action::Static("false"));
            return true;
        }
        NodeKind::True => {
            actions.push(Action::Static("true"));
            return true;
        }
        NodeKind::Proposition { proposition } => {
            actions.push(Action::Owned(format!("p{}", proposition.0)));
            return true;
        }
        _ => {}
    }

    if let (Some(operator), Some(operand)) = (dialect.unary_spelling(kind), unary_operand(kind)) {
        actions.push(Action::Node(operand, Context::Prefix));
        if let Some(interval) = operator.interval {
            actions.push(Action::Owned(interval_operator(
                operator.operator,
                interval,
            )));
        } else {
            actions.push(Action::Static(operator.operator));
        }
        return true;
    }

    if let (Some(operator), Some((left, right))) =
        (dialect.binary_spelling(kind), binary_operands(kind))
    {
        if let Some(interval) = operator.interval {
            actions.push(Action::Node(
                right,
                Context::Binary {
                    precedence: operator.precedence,
                    group_equal: true,
                },
            ));
            actions.push(Action::Owned(interval_operator(
                operator.operator,
                interval,
            )));
            actions.push(Action::Node(
                left,
                Context::Binary {
                    precedence: operator.precedence,
                    group_equal: false,
                },
            ));
        } else {
            binary_actions(
                actions,
                left,
                operator.operator,
                right,
                operator.precedence,
                operator.right_associative,
            );
        }
        return true;
    }
    false
}

fn unary_operand(kind: NodeKind) -> Option<NodeId> {
    match kind {
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. }
        | NodeKind::Once { operand, .. }
        | NodeKind::Historically { operand, .. }
        | NodeKind::StrongPrevious { operand } => Some(operand),
        _ => None,
    }
}

fn binary_operands(kind: NodeKind) -> Option<(NodeId, NodeId)> {
    match kind {
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. }
        | NodeKind::Since { left, right, .. }
        | NodeKind::Triggered { left, right, .. } => Some((left, right)),
        _ => None,
    }
}

fn operands(kind: NodeKind) -> [Option<NodeId>; 2] {
    match kind {
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => [None, None],
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. }
        | NodeKind::Once { operand, .. }
        | NodeKind::Historically { operand, .. }
        | NodeKind::StrongPrevious { operand } => [Some(operand), None],
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. }
        | NodeKind::Since { left, right, .. }
        | NodeKind::Triggered { left, right, .. } => [Some(left), Some(right)],
    }
}

fn failed_report(
    limits: FormatLimits,
    stats: FormatStats,
    code: FormatErrorCode,
    message: String,
) -> FormatReport {
    FormatReport {
        limits,
        stats,
        text: None,
        error: Some(FormatError { code, message }),
    }
}
