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

    let mut rendered: Vec<Option<String>> = vec![None; nodes.len()];
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
    let Some(text) = rendered.get_mut(root).and_then(Option::take) else {
        return failed_report(
            limits,
            stats,
            FormatErrorCode::InvalidGraph,
            "formula root was not rendered".to_owned(),
        );
    };
    stats.output_bytes = text.len();
    FormatReport {
        limits,
        stats,
        text: Some(text),
        error: None,
    }
}

fn render_node(
    kind: NodeKind,
    rendered: &[Option<String>],
    limits: FormatLimits,
    stats: &mut FormatStats,
) -> Result<String, FormatError> {
    let text = match kind {
        NodeKind::False => "false".to_owned(),
        NodeKind::True => "true".to_owned(),
        NodeKind::Proposition { proposition } => format!("p{}", proposition.0),
        NodeKind::Not { operand } => {
            let operand = child(rendered, operand)?;
            assemble(&["!(", operand, ")"])
        }
        NodeKind::And { left, right } => binary(rendered, left, "&", right)?,
        NodeKind::Or { left, right } => binary(rendered, left, "|", right)?,
        NodeKind::Implies { left, right } => binary(rendered, left, "->", right)?,
        NodeKind::Equivalent { left, right } => binary(rendered, left, "<->", right)?,
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
    if text.len() > limits.max_output_bytes {
        return Err(FormatError {
            code: FormatErrorCode::OutputLimit,
            message: format!(
                "canonical text length {} exceeds effective limit {}",
                text.len(),
                limits.max_output_bytes
            ),
        });
    }
    let Some(work) = stats.work.checked_add(text.len()) else {
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
    Ok(text)
}

fn child(rendered: &[Option<String>], id: NodeId) -> Result<&str, FormatError> {
    rendered
        .get(id.0 as usize)
        .and_then(Option::as_deref)
        .ok_or_else(|| FormatError {
            code: FormatErrorCode::InvalidGraph,
            message: format!("operand {} was not rendered before its owner", id.0),
        })
}

fn binary(
    rendered: &[Option<String>],
    left: NodeId,
    operator: &str,
    right: NodeId,
) -> Result<String, FormatError> {
    Ok(assemble(&[
        "(",
        child(rendered, left)?,
        operator,
        child(rendered, right)?,
        ")",
    ]))
}

fn unary_temporal(
    rendered: &[Option<String>],
    operator: &str,
    interval: Interval,
    operand: NodeId,
) -> Result<String, FormatError> {
    let mut prefix = String::new();
    let _ = write!(
        prefix,
        "{operator}[{},{}](",
        interval.start(),
        interval.end()
    );
    Ok(assemble(&[&prefix, child(rendered, operand)?, ")"]))
}

fn binary_temporal(
    rendered: &[Option<String>],
    left: NodeId,
    operator: &str,
    interval: Interval,
    right: NodeId,
) -> Result<String, FormatError> {
    let mut middle = String::new();
    let _ = write!(
        middle,
        "{operator}[{},{}]",
        interval.start(),
        interval.end()
    );
    Ok(assemble(&[
        "(",
        child(rendered, left)?,
        &middle,
        child(rendered, right)?,
        ")",
    ]))
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
