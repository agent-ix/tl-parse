use std::fmt::Write as _;

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

    let mut rendered: Vec<Option<Rendered>> = vec![None; nodes.len()];
    let mut stats = FormatStats::default();
    for (index, node) in nodes.iter().enumerate() {
        if !reachable[index] {
            continue;
        }
        let text = match render_node(node.kind, &rendered, limits, &mut stats) {
            Ok(text) => text,
            Err(error) => {
                return FormatReport {
                    limits,
                    stats,
                    text: None,
                    error: Some(error),
                };
            }
        };
        stats.nodes += 1;
        rendered[index] = Some(text);
    }

    let root = formula.root().0 as usize;
    let Some(rendered_root) = rendered.get_mut(root).and_then(Option::take) else {
        return failed_report(
            limits,
            stats,
            FormatErrorCode::InvalidGraph,
            "formula root was not rendered".to_owned(),
        );
    };
    stats.output_bytes = rendered_root.text.len();
    FormatReport {
        limits,
        stats,
        text: Some(rendered_root.text),
        error: None,
    }
}

fn render_node(
    kind: NodeKind,
    rendered: &[Option<Rendered>],
    limits: FormatLimits,
    stats: &mut FormatStats,
) -> Result<Rendered, FormatError> {
    let rendered_node = match kind {
        NodeKind::False => Rendered::atom("false".to_owned()),
        NodeKind::True => Rendered::atom("true".to_owned()),
        NodeKind::Proposition { proposition } => Rendered::atom(format!("p{}", proposition.0)),
        NodeKind::Not { operand } => {
            let operand = child(rendered, operand)?;
            Rendered::prefix(assemble(&["!", &prefix_operand(operand)]))
        }
        NodeKind::And { left, right } => binary(rendered, left, "&", right, 4, false)?,
        NodeKind::Or { left, right } => binary(rendered, left, "|", right, 3, false)?,
        NodeKind::Implies { left, right } => binary(rendered, left, "->", right, 2, true)?,
        NodeKind::Equivalent { left, right } => binary(rendered, left, "<->", right, 1, false)?,
        NodeKind::Future { interval, operand } => unary_temporal(rendered, "F", interval, operand)?,
        NodeKind::Globally { interval, operand } => {
            unary_temporal(rendered, "G", interval, operand)?
        }
        NodeKind::Until {
            interval,
            left,
            right,
        } => binary_temporal(rendered, left, "U", interval, right)?,
        NodeKind::Release {
            interval,
            left,
            right,
        } => binary_temporal(rendered, left, "R", interval, right)?,
    };
    if rendered_node.text.len() > limits.max_output_bytes {
        return Err(FormatError {
            code: FormatErrorCode::OutputLimit,
            message: format!(
                "canonical text length {} exceeds effective limit {}",
                rendered_node.text.len(),
                limits.max_output_bytes
            ),
        });
    }
    let Some(work) = stats.work.checked_add(1) else {
        return Err(FormatError {
            code: FormatErrorCode::WorkLimit,
            message: "formatter work counter overflowed".to_owned(),
        });
    };
    if work > limits.max_work {
        return Err(FormatError {
            code: FormatErrorCode::WorkLimit,
            message: format!(
                "formatter node work {} exceeds effective limit {}",
                work, limits.max_work
            ),
        });
    }
    stats.work = work;
    Ok(rendered_node)
}

#[derive(Clone, Debug)]
struct Rendered {
    text: String,
    precedence: u8,
}

impl Rendered {
    fn atom(text: String) -> Self {
        Self {
            text,
            precedence: 7,
        }
    }

    fn prefix(text: String) -> Self {
        Self {
            text,
            precedence: 6,
        }
    }
}

fn child(rendered: &[Option<Rendered>], id: NodeId) -> Result<&Rendered, FormatError> {
    rendered
        .get(id.0 as usize)
        .and_then(Option::as_ref)
        .ok_or_else(|| FormatError {
            code: FormatErrorCode::InvalidGraph,
            message: format!("operand {} was not rendered before its owner", id.0),
        })
}

fn binary(
    rendered: &[Option<Rendered>],
    left: NodeId,
    operator: &str,
    right: NodeId,
    precedence: u8,
    right_associative: bool,
) -> Result<Rendered, FormatError> {
    let left = child(rendered, left)?;
    let right = child(rendered, right)?;
    let left = binary_operand(left, precedence, right_associative);
    let right = binary_operand(right, precedence, !right_associative);
    Ok(Rendered {
        text: assemble(&[&left, operator, &right]),
        precedence,
    })
}

fn unary_temporal(
    rendered: &[Option<Rendered>],
    operator: &str,
    interval: Interval,
    operand: NodeId,
) -> Result<Rendered, FormatError> {
    let mut prefix = String::new();
    let _ = write!(
        prefix,
        "{operator}[{},{}]",
        interval.start(),
        interval.end()
    );
    Ok(Rendered::prefix(assemble(&[
        &prefix,
        &prefix_operand(child(rendered, operand)?),
    ])))
}

fn binary_temporal(
    rendered: &[Option<Rendered>],
    left: NodeId,
    operator: &str,
    interval: Interval,
    right: NodeId,
) -> Result<Rendered, FormatError> {
    let mut middle = String::new();
    let _ = write!(
        middle,
        "{operator}[{},{}]",
        interval.start(),
        interval.end()
    );
    binary(rendered, left, &middle, right, 5, false)
}

fn prefix_operand(operand: &Rendered) -> String {
    if operand.precedence < 6 {
        assemble(&["(", &operand.text, ")"])
    } else {
        operand.text.clone()
    }
}

fn binary_operand(operand: &Rendered, parent_precedence: u8, parent_side_groups: bool) -> String {
    if operand.precedence < parent_precedence
        || (operand.precedence == parent_precedence && parent_side_groups)
    {
        assemble(&["(", &operand.text, ")"])
    } else {
        operand.text.clone()
    }
}

fn assemble(parts: &[&str]) -> String {
    let capacity = parts.iter().map(|part| part.len()).sum();
    let mut output = String::with_capacity(capacity);
    for part in parts {
        output.push_str(part);
    }
    output
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
