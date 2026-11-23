//! Internal v6 graph and the single boundary to the tl-syntax editions.
//!
//! The infinite edition (`InfiniteFormulaDocument`) is on the pinned tl-syntax
//! rev. UNBUILT-PIN: the LTLf edition `tl-syntax.formula-ltlf/v1` (tl-syntax
//! FR-332/FR-333, TL-428) is written concurrently in TL-code-tl-syntax; the
//! names below are guesses recorded in PLAN-045 "Deferred verification":
//!
//! - `tl_syntax::LtlfFormulaDocument` (serde, `Clone + Debug + Eq`):
//!   `new(SemanticProfile, NodeId, Vec<LtlfNode>) -> Result<Self, E: Display>`,
//!   `root() -> NodeId`, `nodes() -> &[LtlfNode]`, `semantic_profile()`.
//! - `tl_syntax::LtlfNode { pub kind: LtlfNodeKind, pub span: Option<SourceSpan> }`
//!   with `LtlfNode::with_span(kind, span)`.
//! - `tl_syntax::LtlfNodeKind::{False, True, Proposition { proposition },
//!   Not { operand }, And/Or/Implies/Equivalent { left, right },
//!   Future { operand }, Globally { operand }, Until { left, right },
//!   Release { left, right }, StrongNext { operand }, WeakNext { operand }}`.
//! - `tl_syntax::SemanticProfile::LtlfFiniteTraceV1`.

use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind, Interval, NodeId,
    PropositionId, SemanticProfile, SourceSpan, TemporalInterval, UnboundedInterval,
};
// UNBUILT-PIN: tl-syntax FR-332/FR-333 LTLf edition (see module docs).
pub use tl_syntax::LtlfFormulaDocument;
use tl_syntax::{LtlfNode, LtlfNodeKind};

/// One v6 node; operands are indices of earlier nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    False,
    True,
    Proposition(u32),
    Not(usize),
    And(usize, usize),
    Or(usize, usize),
    Implies(usize, usize),
    Equivalent(usize, usize),
    Future(usize),
    Globally(usize),
    Until(usize, usize),
    Release(usize, usize),
    /// Infinite profile only: lowers to `F[1,1]`.
    Next(usize),
    /// LTLf only.
    StrongNext(usize),
    /// LTLf only.
    WeakNext(usize),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Item {
    pub(crate) kind: Kind,
    pub(crate) span: Option<SourceSpan>,
}

fn id(index: usize) -> Result<NodeId, String> {
    u32::try_from(index)
        .map(NodeId)
        .map_err(|_| "node identity exceeds u32".to_owned())
}

fn index(node: NodeId) -> Result<usize, String> {
    usize::try_from(node.0).map_err(|_| "node identity exceeds usize".to_owned())
}

const fn from_zero() -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(0))
}

pub(crate) fn to_infinite(root: usize, items: &[Item]) -> Result<InfiniteFormulaDocument, String> {
    let mut nodes = Vec::with_capacity(items.len());
    for item in items {
        let kind = match item.kind {
            Kind::False => InfiniteNodeKind::False,
            Kind::True => InfiniteNodeKind::True,
            Kind::Proposition(value) => InfiniteNodeKind::Proposition {
                proposition: PropositionId(value),
            },
            Kind::Not(operand) => InfiniteNodeKind::Not { operand: id(operand)? },
            Kind::And(left, right) => InfiniteNodeKind::And {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Or(left, right) => InfiniteNodeKind::Or {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Implies(left, right) => InfiniteNodeKind::Implies {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Equivalent(left, right) => InfiniteNodeKind::Equivalent {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Future(operand) => InfiniteNodeKind::Future {
                interval: from_zero(),
                operand: id(operand)?,
            },
            Kind::Globally(operand) => InfiniteNodeKind::Globally {
                interval: from_zero(),
                operand: id(operand)?,
            },
            Kind::Until(left, right) => InfiniteNodeKind::Until {
                interval: from_zero(),
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Release(left, right) => InfiniteNodeKind::Release {
                interval: from_zero(),
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Next(operand) => InfiniteNodeKind::Future {
                interval: TemporalInterval::Closed(
                    Interval::new(1, 1).map_err(|error| error.to_string())?,
                ),
                operand: id(operand)?,
            },
            Kind::StrongNext(_) | Kind::WeakNext(_) => {
                return Err("Xs and Xw have no infinite-trace node".to_owned())
            }
        };
        nodes.push(match item.span {
            Some(span) => InfiniteNode::with_span(kind, span),
            None => InfiniteNode::new(kind),
        });
    }
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        id(root)?,
        nodes,
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn to_ltlf(root: usize, items: &[Item]) -> Result<LtlfFormulaDocument, String> {
    let mut nodes = Vec::with_capacity(items.len());
    for item in items {
        let kind = match item.kind {
            Kind::False => LtlfNodeKind::False,
            Kind::True => LtlfNodeKind::True,
            Kind::Proposition(value) => LtlfNodeKind::Proposition {
                proposition: PropositionId(value),
            },
            Kind::Not(operand) => LtlfNodeKind::Not { operand: id(operand)? },
            Kind::And(left, right) => LtlfNodeKind::And {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Or(left, right) => LtlfNodeKind::Or {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Implies(left, right) => LtlfNodeKind::Implies {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Equivalent(left, right) => LtlfNodeKind::Equivalent {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Future(operand) => LtlfNodeKind::Future { operand: id(operand)? },
            Kind::Globally(operand) => LtlfNodeKind::Globally { operand: id(operand)? },
            Kind::Until(left, right) => LtlfNodeKind::Until {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::Release(left, right) => LtlfNodeKind::Release {
                left: id(left)?,
                right: id(right)?,
            },
            Kind::StrongNext(operand) => LtlfNodeKind::StrongNext { operand: id(operand)? },
            Kind::WeakNext(operand) => LtlfNodeKind::WeakNext { operand: id(operand)? },
            Kind::Next(_) => return Err("bare X has no LTLf node".to_owned()),
        };
        let node = match item.span {
            Some(span) => LtlfNode::with_span(kind, span),
            None => LtlfNode::new(kind),
        };
        nodes.push(node);
    }
    LtlfFormulaDocument::new(SemanticProfile::LtlfFiniteTraceV1, id(root)?, nodes)
        .map_err(|error| error.to_string())
}

/// Reads an LTLf document into the internal graph. LTLf is wholly in the image.
pub(crate) fn from_ltlf(document: &LtlfFormulaDocument) -> Result<(usize, Vec<Kind>), String> {
    let mut kinds = Vec::with_capacity(document.nodes().len());
    for node in document.nodes() {
        kinds.push(match node.kind {
            LtlfNodeKind::False => Kind::False,
            LtlfNodeKind::True => Kind::True,
            LtlfNodeKind::Proposition { proposition } => Kind::Proposition(proposition.0),
            LtlfNodeKind::Not { operand } => Kind::Not(index(operand)?),
            LtlfNodeKind::And { left, right } => Kind::And(index(left)?, index(right)?),
            LtlfNodeKind::Or { left, right } => Kind::Or(index(left)?, index(right)?),
            LtlfNodeKind::Implies { left, right } => Kind::Implies(index(left)?, index(right)?),
            LtlfNodeKind::Equivalent { left, right } => {
                Kind::Equivalent(index(left)?, index(right)?)
            }
            LtlfNodeKind::Future { operand } => Kind::Future(index(operand)?),
            LtlfNodeKind::Globally { operand } => Kind::Globally(index(operand)?),
            LtlfNodeKind::Until { left, right } => Kind::Until(index(left)?, index(right)?),
            LtlfNodeKind::Release { left, right } => Kind::Release(index(left)?, index(right)?),
            LtlfNodeKind::StrongNext { operand } => Kind::StrongNext(index(operand)?),
            LtlfNodeKind::WeakNext { operand } => Kind::WeakNext(index(operand)?),
            #[allow(unreachable_patterns)]
            _ => return Err("LTLf node outside tl-parse.clean-ascii/v6".to_owned()),
        });
    }
    Ok((index(document.root())?, kinds))
}

/// Outcome of reading an infinite document into the v6 image (FR-060).
pub(crate) enum Image {
    Expressible(usize, Vec<Kind>),
    NotExpressible(&'static str),
}

const fn is_from_zero(interval: TemporalInterval) -> bool {
    matches!(interval, TemporalInterval::Unbounded(bound) if bound.start() == 0)
}

const fn is_next(interval: TemporalInterval) -> bool {
    matches!(interval, TemporalInterval::Closed(bound) if bound.start() == 1 && bound.end() == 1)
}

/// The infinite image is `F/G/U/R` over `[0,)` and `F[1,1]` (spelled `X`);
/// `G[1,1]`, any other interval and past nodes are refused, never rewritten.
pub(crate) fn from_infinite(document: &InfiniteFormulaDocument) -> Result<Image, String> {
    let mut kinds = Vec::with_capacity(document.nodes().len());
    for node in document.nodes() {
        let kind = match node.kind {
            InfiniteNodeKind::False => Kind::False,
            InfiniteNodeKind::True => Kind::True,
            InfiniteNodeKind::Proposition { proposition } => Kind::Proposition(proposition.0),
            InfiniteNodeKind::Not { operand } => Kind::Not(index(operand)?),
            InfiniteNodeKind::And { left, right } => Kind::And(index(left)?, index(right)?),
            InfiniteNodeKind::Or { left, right } => Kind::Or(index(left)?, index(right)?),
            InfiniteNodeKind::Implies { left, right } => Kind::Implies(index(left)?, index(right)?),
            InfiniteNodeKind::Equivalent { left, right } => {
                Kind::Equivalent(index(left)?, index(right)?)
            }
            InfiniteNodeKind::Future { interval, operand } if is_from_zero(interval) => {
                Kind::Future(index(operand)?)
            }
            InfiniteNodeKind::Future { interval, operand } if is_next(interval) => {
                Kind::Next(index(operand)?)
            }
            InfiniteNodeKind::Globally { interval, operand } if is_from_zero(interval) => {
                Kind::Globally(index(operand)?)
            }
            InfiniteNodeKind::Until {
                interval,
                left,
                right,
            } if is_from_zero(interval) => Kind::Until(index(left)?, index(right)?),
            InfiniteNodeKind::Release {
                interval,
                left,
                right,
            } if is_from_zero(interval) => Kind::Release(index(left)?, index(right)?),
            InfiniteNodeKind::Future { .. }
            | InfiniteNodeKind::Globally { .. }
            | InfiniteNodeKind::Until { .. }
            | InfiniteNodeKind::Release { .. } => {
                return Ok(Image::NotExpressible(
                    "only [0,) intervals and F[1,1] (X) have v6 text",
                ))
            }
            InfiniteNodeKind::Once { .. }
            | InfiniteNodeKind::Historically { .. }
            | InfiniteNodeKind::StrongPrevious { .. }
            | InfiniteNodeKind::Since { .. }
            | InfiniteNodeKind::Triggered { .. } => {
                return Ok(Image::NotExpressible("past nodes have no v6 text"))
            }
        };
        kinds.push(kind);
    }
    Ok(Image::Expressible(index(document.root())?, kinds))
}
