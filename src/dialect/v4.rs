//! Lexical and binding policy for the infinite-trace dialect.

use crate::lexer::TokenKind;

pub(crate) fn classify_identifier(lexeme: &str) -> Option<TokenKind> {
    Some(match lexeme {
        "false" => TokenKind::False,
        "true" => TokenKind::True,
        "fair" => TokenKind::Fair,
        "F" => TokenKind::Future,
        "G" => TokenKind::Globally,
        "U" => TokenKind::Until,
        "R" => TokenKind::Release,
        "W" => TokenKind::WeakUntil,
        "M" => TokenKind::StrongRelease,
        "O" => TokenKind::Once,
        "H" => TokenKind::Historically,
        "S" => TokenKind::Since,
        "T" => TokenKind::Triggered,
        _ => return None,
    })
}

pub(crate) const fn atom_keyword_boundary(next: u8, followed_by_bracket: bool) -> bool {
    matches!(next, b'U' | b'R' | b'W' | b'M' | b'S' | b'T') && followed_by_bracket
}

pub(crate) const fn binary_binding_power(token: TokenKind) -> Option<(u8, u8)> {
    match token {
        TokenKind::Equivalent => Some((1, 2)),
        TokenKind::Implies => Some((2, 2)),
        TokenKind::Or => Some((3, 4)),
        TokenKind::And => Some((4, 5)),
        TokenKind::Until
        | TokenKind::Release
        | TokenKind::WeakUntil
        | TokenKind::StrongRelease
        | TokenKind::Since
        | TokenKind::Triggered => Some((5, 6)),
        _ => None,
    }
}

pub(crate) const fn permits_temporal_prefix(token: TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Future
            | TokenKind::Globally
            | TokenKind::Once
            | TokenKind::Historically
            | TokenKind::StrongPrevious
    )
}
