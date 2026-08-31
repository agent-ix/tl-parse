use tl_syntax::{Formula, FormulaDocument, Interval, NodeId, NodeKind};

use crate::{FormatError, FormatErrorCode, FormatLimits, FormatReport, FormatStats};

/// Validates and canonically formats an owned formula document.
pub fn format_document(document: &FormulaDocument, limits: FormatLimits) -> FormatReport {
    match document.validate() {
        Ok(formula) => format_formula(formula, limits),
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
    let limits = limits.clamped();
    let nodes = formula.nodes();
    let mut reachable = vec![false; nodes.len()];
    let mut stack = vec![formula.root()];
    while let Some(id) = stack.pop() {
        let index = id.0 as usize;
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
                    let grouped = context.groups(precedence(node.kind));
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
                    push_node_actions(&mut actions, node.kind);
                    Ok(())
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

fn push_node_actions(actions: &mut Vec<Action>, kind: NodeKind) {
    match kind {
        NodeKind::False => actions.push(Action::Static("false")),
        NodeKind::True => actions.push(Action::Static("true")),
        NodeKind::Proposition { proposition } => {
            actions.push(Action::Owned(format!("p{}", proposition.0)));
        }
        NodeKind::Not { operand } => {
            actions.push(Action::Node(operand, Context::Prefix));
            actions.push(Action::Static("!"));
        }
        NodeKind::And { left, right } => binary_actions(actions, left, "&", right, 4, false),
        NodeKind::Or { left, right } => binary_actions(actions, left, "|", right, 3, false),
        NodeKind::Implies { left, right } => {
            binary_actions(actions, left, "->", right, 2, true);
        }
        NodeKind::Equivalent { left, right } => {
            binary_actions(actions, left, "<->", right, 1, false);
        }
        NodeKind::Future { interval, operand } => {
            actions.push(Action::Node(operand, Context::Prefix));
            actions.push(Action::Owned(interval_operator("F", interval)));
        }
        NodeKind::Globally { interval, operand } => {
            actions.push(Action::Node(operand, Context::Prefix));
            actions.push(Action::Owned(interval_operator("G", interval)));
        }
        NodeKind::Until {
            interval,
            left,
            right,
        } => {
            actions.push(Action::Node(
                right,
                Context::Binary {
                    precedence: 5,
                    group_equal: true,
                },
            ));
            actions.push(Action::Owned(interval_operator("U", interval)));
            actions.push(Action::Node(
                left,
                Context::Binary {
                    precedence: 5,
                    group_equal: false,
                },
            ));
        }
        NodeKind::Release {
            interval,
            left,
            right,
        } => {
            actions.push(Action::Node(
                right,
                Context::Binary {
                    precedence: 5,
                    group_equal: true,
                },
            ));
            actions.push(Action::Owned(interval_operator("R", interval)));
            actions.push(Action::Node(
                left,
                Context::Binary {
                    precedence: 5,
                    group_equal: false,
                },
            ));
        }
    }
}

fn precedence(kind: NodeKind) -> u8 {
    match kind {
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => 7,
        NodeKind::Not { .. } | NodeKind::Future { .. } | NodeKind::Globally { .. } => 6,
        NodeKind::Until { .. } | NodeKind::Release { .. } => 5,
        NodeKind::And { .. } => 4,
        NodeKind::Or { .. } => 3,
        NodeKind::Implies { .. } => 2,
        NodeKind::Equivalent { .. } => 1,
    }
}

fn operands(kind: NodeKind) -> [Option<NodeId>; 2] {
    match kind {
        NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => [None, None],
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. } => [Some(operand), None],
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. } => [Some(left), Some(right)],
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
