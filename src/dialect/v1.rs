//! `tl-parse.clean-ascii/v1`: primitive future-time and Boolean syntax.

use tl_syntax::{FormulaDocument, FormulaError, Node, NodeId, NodeKind, SemanticProfile};

use crate::{lexer::TokenKind, parser::parse_dialect, ParseLimits, ParseReport};

use super::{BinarySpelling, Dialect, UnarySpelling};

pub(crate) fn parse(source: &str, profile: SemanticProfile, limits: ParseLimits) -> ParseReport {
    parse_dialect(source, profile, limits, Dialect::V1).report
}

pub(crate) fn classify_identifier(lexeme: &str) -> Option<TokenKind> {
    match lexeme {
        "false" => Some(TokenKind::False),
        "true" => Some(TokenKind::True),
        "F" => Some(TokenKind::Future),
        "G" => Some(TokenKind::Globally),
        "U" => Some(TokenKind::Until),
        "R" => Some(TokenKind::Release),
        _ => None,
    }
}

pub(crate) const fn unsupported_operator_profile(_lexeme: &str) -> Option<&'static str> {
    None
}

pub(crate) const fn atom_keyword_boundary(next: u8, followed_by_bracket: bool) -> bool {
    // Keep recognized binary spellings separate even when this dialect will
    // reject them, so a compact `p0S[0,1]p1` points at `S`, not at `p0S`.
    matches!(next, b'U' | b'R' | b'W' | b'M' | b'S' | b'T') && followed_by_bracket
}

pub(crate) const fn binary_binding_power(token: TokenKind) -> Option<(u8, u8)> {
    match token {
        TokenKind::Equivalent => Some((1, 2)),
        TokenKind::Implies => Some((2, 2)),
        TokenKind::Or => Some((3, 4)),
        TokenKind::And => Some((4, 5)),
        TokenKind::Until | TokenKind::Release => Some((5, 6)),
        _ => None,
    }
}

pub(crate) const fn permits_temporal_prefix(token: TokenKind) -> bool {
    matches!(token, TokenKind::Future | TokenKind::Globally)
}

pub(crate) fn build_document(
    profile: SemanticProfile,
    root: NodeId,
    nodes: Vec<Node>,
) -> Result<FormulaDocument, FormulaError> {
    FormulaDocument::new(profile, root, nodes)
}

pub(crate) const fn unary_spelling(kind: NodeKind) -> Option<UnarySpelling> {
    match kind {
        NodeKind::Not { .. } => Some(UnarySpelling {
            operator: "!",
            interval: None,
        }),
        NodeKind::Future { interval, .. } => Some(UnarySpelling {
            operator: "F",
            interval: Some(interval),
        }),
        NodeKind::Globally { interval, .. } => Some(UnarySpelling {
            operator: "G",
            interval: Some(interval),
        }),
        _ => None,
    }
}

pub(crate) const fn binary_spelling(kind: NodeKind) -> Option<BinarySpelling> {
    let (operator, interval, precedence, right_associative) = match kind {
        NodeKind::And { .. } => ("&", None, 4, false),
        NodeKind::Or { .. } => ("|", None, 3, false),
        NodeKind::Implies { .. } => ("->", None, 2, true),
        NodeKind::Equivalent { .. } => ("<->", None, 1, false),
        NodeKind::Until { interval, .. } => ("U", Some(interval), 5, false),
        NodeKind::Release { interval, .. } => ("R", Some(interval), 5, false),
        _ => return None,
    };
    Some(BinarySpelling {
        operator,
        interval,
        precedence,
        right_associative,
    })
}
