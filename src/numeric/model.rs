//! Internal expression arena and the single boundary to the tl-syntax numeric
//! model (FR-302/FR-303).
//!
//! UNBUILT-PIN: written against tl-syntax `TL-code-unbuilt` (TL-420), reached
//! through the `[patch]` in `Cargo.toml`. Owner items used here:
//! `PredicateTableDocument::{new(SignalCatalog, Vec<ExprNode>, Vec<Predicate>),
//! expressions(), predicates()}`, `ExprNode` (non-exhaustive; `SlotSignal` is
//! template-only), `ExprId(pub u32)`, `Predicate { proposition, operator, left,
//! right }`, `ComparisonOperator`, `PredicateTableError`, `ExpressionError`,
//! `ExpressionLimit` and `MAX_EXPRESSION_DEPTH`. Only this file must change if
//! those names move before the merged rev is pinned.

use std::collections::BTreeMap;

use tl_syntax::{PropositionId, SignalCatalogDocument, SignalDomain, SignalId};
// UNBUILT-PIN: tl-syntax FR-302/FR-303 owner types (see module docs).
pub use tl_syntax::PredicateTableDocument;
use tl_syntax::{
    ComparisonOperator, ExprId, ExprNode, ExpressionError, ExpressionLimit,
    Predicate as OwnerPredicate, PredicateTableError, MAX_EXPRESSION_DEPTH,
};

use super::{NumericDiagnosticCode as Code, MAX_NUMERIC_SCALE};

/// One expression node; operands are indices of earlier nodes in the same arena.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Expr {
    Signal(SignalId),
    Integer(i64),
    Decimal { coefficient: i64, scale: u8 },
    Neg(usize),
    Abs(usize),
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Rescale(usize, u8),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Cmp {
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Eq => "==",
            Self::Ne => "!=",
        }
    }
}

/// One table entry over a shared arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Predicate {
    pub(crate) proposition: PropositionId,
    pub(crate) operator: Cmp,
    pub(crate) left: usize,
    pub(crate) right: usize,
}

/// FR-302 derived type: scale and inclusive coefficient range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExprType {
    pub(crate) scale: u8,
    pub(crate) minimum: i64,
    pub(crate) maximum: i64,
}

/// Catalog lookups for v5 name resolution.
pub(crate) struct CatalogView {
    by_name: BTreeMap<String, (SignalId, SignalDomain)>,
    names: BTreeMap<SignalId, (String, SignalDomain)>,
    pub(crate) max_binding: Option<u32>,
    pub(crate) bound: BTreeMap<u32, SignalId>,
}

impl CatalogView {
    pub(crate) fn new(catalog: &SignalCatalogDocument) -> Self {
        let mut by_name = BTreeMap::new();
        let mut names = BTreeMap::new();
        for owned in catalog.signals() {
            let declaration = owned.as_borrowed();
            by_name.insert(
                declaration.name().to_owned(),
                (declaration.id(), declaration.domain()),
            );
            names.insert(
                declaration.id(),
                (declaration.name().to_owned(), declaration.domain()),
            );
        }
        let mut bound = BTreeMap::new();
        for binding in catalog.bindings() {
            bound.insert(binding.proposition().0, binding.signal());
        }
        Self {
            by_name,
            names,
            max_binding: bound.keys().next_back().copied(),
            bound,
        }
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<(SignalId, SignalDomain)> {
        self.by_name.get(name).copied()
    }

    pub(crate) fn name(&self, id: SignalId) -> Option<(&str, SignalDomain)> {
        self.names.get(&id).map(|(name, domain)| (name.as_str(), *domain))
    }
}

fn pow10(exponent: u8) -> i128 {
    let mut value: i128 = 1;
    for _ in 0..exponent {
        value *= 10;
    }
    value
}

fn fit(minimum: i128, maximum: i128, scale: u8) -> Result<ExprType, Code> {
    match (i64::try_from(minimum), i64::try_from(maximum)) {
        (Ok(minimum), Ok(maximum)) => Ok(ExprType {
            scale,
            minimum,
            maximum,
        }),
        _ => Err(Code::RangeOverflow),
    }
}

/// Types every node in arena order; returns the first refusal and its node.
///
/// This mirrors tl-syntax FR-302 so that each refusal has the byte locus of the
/// offending sub-expression; the tl-syntax constructor in [`build_table`]
/// remains the authority and is always called afterwards. As in the owner,
/// depth is checked before typing: a leaf has depth 1 and a node one more than
/// its deepest operand, so a long left-associative chain such as
/// `1 + 1 + ... + 1` is refused as `depth_limit` at the first node past
/// `MAX_EXPRESSION_DEPTH` (FR-029: expression nesting counts toward 256).
pub(crate) fn type_arena(
    arena: &[Expr],
    catalog: &CatalogView,
) -> Result<Vec<ExprType>, (Code, usize)> {
    let mut types: Vec<ExprType> = Vec::with_capacity(arena.len());
    let mut depths: Vec<usize> = Vec::with_capacity(arena.len());
    for (index, node) in arena.iter().enumerate() {
        let depth = operands(*node)
            .into_iter()
            .flatten()
            .map(|operand| depths.get(operand).copied().unwrap_or(0))
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        if depth > MAX_EXPRESSION_DEPTH {
            return Err((Code::DepthLimit, index));
        }
        depths.push(depth);
        let typed = type_node(*node, &types, catalog).map_err(|code| (code, index))?;
        types.push(typed);
    }
    Ok(types)
}

const fn operands(node: Expr) -> [Option<usize>; 2] {
    match node {
        Expr::Signal(_) | Expr::Integer(_) | Expr::Decimal { .. } => [None, None],
        Expr::Neg(operand) | Expr::Abs(operand) | Expr::Rescale(operand, _) => {
            [Some(operand), None]
        }
        Expr::Add(left, right) | Expr::Sub(left, right) | Expr::Mul(left, right) => {
            [Some(left), Some(right)]
        }
    }
}

fn type_node(node: Expr, types: &[ExprType], catalog: &CatalogView) -> Result<ExprType, Code> {
    let operand = |index: usize| types.get(index).copied().ok_or(Code::UnexpectedToken);
    match node {
        Expr::Signal(id) => match catalog.name(id).map(|(_, domain)| domain) {
            None => Err(Code::UnknownSignal),
            Some(SignalDomain::Boolean) => Err(Code::NonNumericSignal),
            Some(SignalDomain::Integer(domain)) => Ok(ExprType {
                scale: 0,
                minimum: domain.minimum(),
                maximum: domain.maximum(),
            }),
            Some(SignalDomain::FixedDecimal(domain)) => Ok(ExprType {
                scale: domain.scale(),
                minimum: domain.minimum_coefficient(),
                maximum: domain.maximum_coefficient(),
            }),
        },
        Expr::Integer(value) => Ok(ExprType {
            scale: 0,
            minimum: value,
            maximum: value,
        }),
        Expr::Decimal { coefficient, scale } => {
            if scale > MAX_NUMERIC_SCALE {
                return Err(Code::ScaleOverflow);
            }
            Ok(ExprType {
                scale,
                minimum: coefficient,
                maximum: coefficient,
            })
        }
        Expr::Neg(index) => {
            let inner = operand(index)?;
            fit(
                -i128::from(inner.maximum),
                -i128::from(inner.minimum),
                inner.scale,
            )
        }
        Expr::Abs(index) => {
            let inner = operand(index)?;
            let low = i128::from(inner.minimum);
            let high = i128::from(inner.maximum);
            let top = low.abs().max(high.abs());
            let bottom = if low <= 0 && 0 <= high {
                0
            } else {
                low.abs().min(high.abs())
            };
            fit(bottom, top, inner.scale)
        }
        Expr::Add(left, right) | Expr::Sub(left, right) => {
            let (left_type, right_type) = (operand(left)?, operand(right)?);
            if left_type.scale != right_type.scale {
                return Err(Code::ScaleMismatch);
            }
            let (a, b) = (i128::from(left_type.minimum), i128::from(left_type.maximum));
            let (c, d) = (i128::from(right_type.minimum), i128::from(right_type.maximum));
            if matches!(node, Expr::Add(..)) {
                fit(a + c, b + d, left_type.scale)
            } else {
                fit(a - d, b - c, left_type.scale)
            }
        }
        Expr::Mul(left, right) => {
            let (left_type, right_type) = (operand(left)?, operand(right)?);
            let scale = left_type.scale.saturating_add(right_type.scale);
            if scale > MAX_NUMERIC_SCALE {
                return Err(Code::ScaleOverflow);
            }
            let (a, b) = (i128::from(left_type.minimum), i128::from(left_type.maximum));
            let (c, d) = (i128::from(right_type.minimum), i128::from(right_type.maximum));
            // |corner| <= 2^126, so each product fits i128.
            let corners = [a * c, a * d, b * c, b * d];
            let minimum = corners.iter().copied().min().unwrap_or(0);
            let maximum = corners.iter().copied().max().unwrap_or(0);
            fit(minimum, maximum, scale)
        }
        Expr::Rescale(index, target) => {
            let inner = operand(index)?;
            if target > MAX_NUMERIC_SCALE {
                return Err(Code::ScaleOverflow);
            }
            if target < inner.scale {
                return Err(Code::RescaleNarrowing);
            }
            let factor = pow10(target - inner.scale);
            fit(
                i128::from(inner.minimum) * factor,
                i128::from(inner.maximum) * factor,
                target,
            )
        }
    }
}

fn owner_id(index: usize) -> Result<ExprId, Code> {
    u32::try_from(index).map(ExprId).map_err(|_| Code::NodeLimit)
}

fn to_owner(node: Expr) -> Result<ExprNode, Code> {
    Ok(match node {
        Expr::Signal(signal) => ExprNode::Signal { signal },
        Expr::Integer(value) => ExprNode::Integer { value },
        Expr::Decimal { coefficient, scale } => ExprNode::Decimal { coefficient, scale },
        Expr::Neg(operand) => ExprNode::Neg {
            operand: owner_id(operand)?,
        },
        Expr::Abs(operand) => ExprNode::Abs {
            operand: owner_id(operand)?,
        },
        Expr::Add(left, right) => ExprNode::Add {
            left: owner_id(left)?,
            right: owner_id(right)?,
        },
        Expr::Sub(left, right) => ExprNode::Sub {
            left: owner_id(left)?,
            right: owner_id(right)?,
        },
        Expr::Mul(left, right) => ExprNode::Mul {
            left: owner_id(left)?,
            right: owner_id(right)?,
        },
        Expr::Rescale(operand, scale) => ExprNode::Rescale {
            operand: owner_id(operand)?,
            scale,
        },
    })
}

const fn owner_operator(operator: Cmp) -> ComparisonOperator {
    match operator {
        Cmp::Lt => ComparisonOperator::Lt,
        Cmp::Le => ComparisonOperator::Le,
        Cmp::Gt => ComparisonOperator::Gt,
        Cmp::Ge => ComparisonOperator::Ge,
        Cmp::Eq => ComparisonOperator::Eq,
        Cmp::Ne => ComparisonOperator::Ne,
    }
}

const fn from_owner_operator(operator: ComparisonOperator) -> Cmp {
    match operator {
        ComparisonOperator::Lt => Cmp::Lt,
        ComparisonOperator::Le => Cmp::Le,
        ComparisonOperator::Gt => Cmp::Gt,
        ComparisonOperator::Ge => Cmp::Ge,
        ComparisonOperator::Eq => Cmp::Eq,
        ComparisonOperator::Ne => Cmp::Ne,
    }
}

/// Owner refusal of a table the local mirror admitted.
///
/// SPEC-GAP: the closed v5 code set has no `validation_failure`.
/// `OperandNotPreceding` and `ScratchTooSmall` are internal faults of this
/// parser; they map to `node_limit` (the nearest resource class) rather than
/// to a typing code that would misdescribe the source.
fn expression_code(error: &ExpressionError) -> Code {
    match error {
        ExpressionError::UnknownSignal { .. } => Code::UnknownSignal,
        ExpressionError::NonNumericSignal { .. } => Code::NonNumericSignal,
        ExpressionError::ScaleMismatch { .. } => Code::ScaleMismatch,
        ExpressionError::ScaleOverflow { .. } => Code::ScaleOverflow,
        ExpressionError::RescaleNarrowing { .. } => Code::RescaleNarrowing,
        ExpressionError::RangeOverflow { .. } => Code::RangeOverflow,
        ExpressionError::ExpressionLimitExceeded {
            resource: ExpressionLimit::Depth,
            ..
        } => Code::DepthLimit,
        ExpressionError::ExpressionLimitExceeded { .. } => Code::NodeLimit,
        ExpressionError::SlotOutsideTemplate { .. } => Code::SlotOutsideTemplate,
        // `OperandNotPreceding`, `ScratchTooSmall`, and any later variant of
        // the non-exhaustive owner enum.
        _ => Code::NodeLimit,
    }
}

/// Builds the owner table through the tl-syntax constructor (the typing authority).
///
/// SPEC-GAP: a catalog that fails `validate()` here was already admitted by
/// tl-syntax as a document; the failure is an internal fault and maps to
/// `unknown_signal` at the root span.
pub(crate) fn build_table(
    catalog: &SignalCatalogDocument,
    arena: &[Expr],
    predicates: &[Predicate],
) -> Result<PredicateTableDocument, Code> {
    let view = catalog.validate().map_err(|_| Code::UnknownSignal)?;
    let expressions = arena
        .iter()
        .map(|node| to_owner(*node))
        .collect::<Result<Vec<_>, Code>>()?;
    let definitions = predicates
        .iter()
        .map(|predicate| {
            Ok(OwnerPredicate {
                proposition: predicate.proposition,
                operator: owner_operator(predicate.operator),
                left: owner_id(predicate.left)?,
                right: owner_id(predicate.right)?,
            })
        })
        .collect::<Result<Vec<_>, Code>>()?;
    PredicateTableDocument::new(view, expressions, definitions).map_err(|error| match error {
        PredicateTableError::Expression(inner) => expression_code(&inner),
        PredicateTableError::ScaleMismatch { .. } => Code::ScaleMismatch,
        PredicateTableError::PredicateLimitExceeded { .. } => Code::NodeLimit,
        // `PropositionNotIncreasing`, `RootOutOfRange` and later variants are
        // faults of this parser, not properties of the source (SPEC-GAP above).
        _ => Code::NodeLimit,
    })
}

/// Reads an owner table back into the internal arena.
pub(crate) fn read_table(
    table: &PredicateTableDocument,
) -> Result<(Vec<Expr>, Vec<Predicate>), String> {
    let index = |id: ExprId| usize::try_from(id.0).map_err(|_| "expression id".to_owned());
    let mut arena = Vec::with_capacity(table.expressions().len());
    for node in table.expressions() {
        let converted = match *node {
            ExprNode::Signal { signal } => Expr::Signal(signal),
            ExprNode::Integer { value } => Expr::Integer(value),
            ExprNode::Decimal { coefficient, scale } => Expr::Decimal { coefficient, scale },
            ExprNode::Neg { operand } => Expr::Neg(index(operand)?),
            ExprNode::Abs { operand } => Expr::Abs(index(operand)?),
            ExprNode::Add { left, right } => Expr::Add(index(left)?, index(right)?),
            ExprNode::Sub { left, right } => Expr::Sub(index(left)?, index(right)?),
            ExprNode::Mul { left, right } => Expr::Mul(index(left)?, index(right)?),
            ExprNode::Rescale { operand, scale } => Expr::Rescale(index(operand)?, scale),
            // `SlotSignal` is template-only and any later variant has no v5 text.
            _ => return Err("expression node outside tl-parse.clean-ascii/v5".to_owned()),
        };
        arena.push(converted);
    }
    let mut predicates = Vec::with_capacity(table.predicates().len());
    for definition in table.predicates() {
        predicates.push(Predicate {
            proposition: definition.proposition,
            operator: from_owner_operator(definition.operator),
            left: index(definition.left)?,
            right: index(definition.right)?,
        });
    }
    Ok((arena, predicates))
}
