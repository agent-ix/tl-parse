//! Canonical `tl-parse.clean-ascii/v6` formatter (FR-060).
//!
//! Canonical text uses the minimum parentheses of the v1 precedence rules and
//! single spaces: `!p0`, `G p0`, `Xs p0`, `p0 U p1`, `p0 & (p1 | p2)`.
//! A graph outside the dialect image is refused as `not_expressible`, never
//! rewritten.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{
    model::{from_infinite, from_ltlf, Image, Kind},
    LtlDocument, LtlProfile,
};
use crate::{FormatLimits, FormatStats};

/// Stable v6 format refusal codes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LtlFormatErrorCode {
    /// The graph is valid but outside the v6 image under the selected profile.
    NotExpressible,
    /// The graph is malformed (operand order, identities).
    InvalidGraph,
    /// Output exceeds the effective byte limit.
    OutputLimit,
    /// Formatter work exceeds the effective limit.
    WorkLimit,
}

impl LtlFormatErrorCode {
    /// Stable wire spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotExpressible => "not_expressible",
            Self::InvalidGraph => "invalid_graph",
            Self::OutputLimit => "output_limit",
            Self::WorkLimit => "work_limit",
        }
    }
}

impl fmt::Display for LtlFormatErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Typed v6 format refusal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LtlFormatError {
    /// Stable code.
    pub code: LtlFormatErrorCode,
    /// Human-readable context.
    pub message: String,
}

/// Result of bounded v6 formatting: text xor error.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LtlFormatReport {
    /// Effective limits.
    pub limits: FormatLimits,
    /// Deterministic counters.
    pub stats: FormatStats,
    /// Canonical text on success only.
    pub text: Option<String>,
    /// Typed refusal on failure only.
    pub error: Option<LtlFormatError>,
}

fn refused(
    limits: FormatLimits,
    stats: FormatStats,
    code: LtlFormatErrorCode,
    message: &str,
) -> LtlFormatReport {
    LtlFormatReport {
        limits,
        stats,
        text: None,
        error: Some(LtlFormatError {
            code,
            message: message.to_owned(),
        }),
    }
}

#[derive(Clone, Copy)]
enum Context {
    Root,
    Prefix,
    Binary { precedence: u8, group_equal: bool },
}

impl Context {
    const fn groups(self, child: u8) -> bool {
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
    Node(usize, Context),
    Text(&'static str),
}

const fn precedence(kind: Kind) -> u8 {
    match kind {
        Kind::False | Kind::True | Kind::Proposition(_) => 7,
        Kind::Not(_)
        | Kind::Future(_)
        | Kind::Globally(_)
        | Kind::Next(_)
        | Kind::StrongNext(_)
        | Kind::WeakNext(_) => 6,
        Kind::Until(..) | Kind::Release(..) => 5,
        Kind::And(..) => 4,
        Kind::Or(..) => 3,
        Kind::Implies(..) => 2,
        Kind::Equivalent(..) => 1,
    }
}

fn operands_precede(kinds: &[Kind]) -> bool {
    kinds.iter().enumerate().all(|(index, kind)| {
        let operands: [Option<usize>; 2] = match *kind {
            Kind::False | Kind::True | Kind::Proposition(_) => [None, None],
            Kind::Not(a)
            | Kind::Future(a)
            | Kind::Globally(a)
            | Kind::Next(a)
            | Kind::StrongNext(a)
            | Kind::WeakNext(a) => [Some(a), None],
            Kind::And(a, b)
            | Kind::Or(a, b)
            | Kind::Implies(a, b)
            | Kind::Equivalent(a, b)
            | Kind::Until(a, b)
            | Kind::Release(a, b) => [Some(a), Some(b)],
        };
        operands.into_iter().flatten().all(|operand| operand < index)
    })
}

/// Formats a v6 graph under the caller's profile.
///
/// Refused as `not_expressible`: a document of the other profile's edition
/// (for example an LTLf graph with `Xs` under `InfiniteTrace`), and under
/// `InfiniteTrace` every closed interval except `F[1,1]` (spelled `X`; so
/// `G[1,1]` is refused), any `[a,)` with `a > 0`, and past nodes.
pub fn format_clean_ascii_v6(
    document: &LtlDocument,
    profile: LtlProfile,
    limits: FormatLimits,
) -> LtlFormatReport {
    let limits = limits.clamped();
    let mut stats = FormatStats::default();
    if document.profile() != profile {
        return refused(
            limits,
            stats,
            LtlFormatErrorCode::NotExpressible,
            "the document's edition belongs to the other v6 profile",
        );
    }
    let read = match document {
        LtlDocument::Ltlf(ltlf) => from_ltlf(ltlf).map(|(root, kinds)| Image::Expressible(root, kinds)),
        LtlDocument::Infinite(infinite) => from_infinite(infinite),
    };
    let (root, kinds) = match read {
        Ok(Image::Expressible(root, kinds)) => (root, kinds),
        Ok(Image::NotExpressible(message)) => {
            return refused(limits, stats, LtlFormatErrorCode::NotExpressible, message)
        }
        Err(message) => return refused(limits, stats, LtlFormatErrorCode::InvalidGraph, &message),
    };
    if root >= kinds.len() || !operands_precede(&kinds) {
        return refused(
            limits,
            stats,
            LtlFormatErrorCode::InvalidGraph,
            "operands must precede their users and the root must exist",
        );
    }
    let mut text = String::new();
    let mut actions = vec![Action::Node(root, Context::Root)];
    while let Some(action) = actions.pop() {
        let piece: String = match action {
            Action::Text(value) => value.to_owned(),
            Action::Node(index, context) => {
                stats.nodes = stats.nodes.saturating_add(1);
                stats.work = stats.work.saturating_add(1);
                if stats.work > limits.max_work {
                    return refused(
                        limits,
                        stats,
                        LtlFormatErrorCode::WorkLimit,
                        "formatter work exceeds the effective limit",
                    );
                }
                let kind = kinds[index];
                let own = precedence(kind);
                let grouped = context.groups(own);
                if grouped {
                    actions.push(Action::Text(")"));
                }
                let open = if grouped { "(" } else { "" };
                match kind {
                    Kind::False => format!("{open}false"),
                    Kind::True => format!("{open}true"),
                    Kind::Proposition(value) => format!("{open}p{value}"),
                    Kind::Not(operand) => {
                        actions.push(Action::Node(operand, Context::Prefix));
                        format!("{open}!")
                    }
                    Kind::Future(operand)
                    | Kind::Globally(operand)
                    | Kind::Next(operand)
                    | Kind::StrongNext(operand)
                    | Kind::WeakNext(operand) => {
                        let keyword = match kind {
                            Kind::Future(_) => "F",
                            Kind::Globally(_) => "G",
                            Kind::Next(_) => "X",
                            Kind::StrongNext(_) => "Xs",
                            _ => "Xw",
                        };
                        actions.push(Action::Node(operand, Context::Prefix));
                        format!("{open}{keyword} ")
                    }
                    Kind::And(left, right)
                    | Kind::Or(left, right)
                    | Kind::Implies(left, right)
                    | Kind::Equivalent(left, right)
                    | Kind::Until(left, right)
                    | Kind::Release(left, right) => {
                        let (operator, right_associative) = match kind {
                            Kind::And(..) => (" & ", false),
                            Kind::Or(..) => (" | ", false),
                            Kind::Implies(..) => (" -> ", true),
                            Kind::Equivalent(..) => (" <-> ", false),
                            Kind::Until(..) => (" U ", false),
                            _ => (" R ", false),
                        };
                        actions.push(Action::Node(
                            right,
                            Context::Binary {
                                precedence: own,
                                group_equal: !right_associative,
                            },
                        ));
                        actions.push(Action::Text(operator));
                        actions.push(Action::Node(
                            left,
                            Context::Binary {
                                precedence: own,
                                group_equal: right_associative,
                            },
                        ));
                        open.to_owned()
                    }
                }
            }
        };
        let length = text.len().saturating_add(piece.len());
        if length > limits.max_output_bytes {
            return refused(
                limits,
                stats,
                LtlFormatErrorCode::OutputLimit,
                "canonical text exceeds the effective output limit",
            );
        }
        stats.work = stats.work.saturating_add(piece.len());
        if stats.work > limits.max_work {
            return refused(
                limits,
                stats,
                LtlFormatErrorCode::WorkLimit,
                "formatter work exceeds the effective limit",
            );
        }
        text.push_str(&piece);
    }
    stats.output_bytes = text.len();
    LtlFormatReport {
        limits,
        stats,
        text: Some(text),
        error: None,
    }
}
