//! Closed, versioned clean-ASCII dialect policies.
//!
//! Traversal lives in the shared lexer/parser/formatter modules. Every choice
//! that can change accepted spelling, precedence, profile, schema, or lowering
//! is selected here and owned by the corresponding version module.

pub(crate) mod v1;
pub(crate) mod v2;
pub(crate) mod v3;
pub(crate) mod v4;

use tl_syntax::{FormulaDocument, FormulaError, Interval, Node, NodeId, NodeKind, SemanticProfile};

use crate::lexer::TokenKind;

/// One explicitly selected, closed input dialect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Dialect {
    V1,
    V2,
    V3,
    V4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnarySpelling {
    pub(crate) operator: &'static str,
    pub(crate) interval: Option<Interval>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BinarySpelling {
    pub(crate) operator: &'static str,
    pub(crate) interval: Option<Interval>,
    pub(crate) precedence: u8,
    pub(crate) right_associative: bool,
}

impl Dialect {
    pub(crate) fn classify_identifier(self, lexeme: &str) -> Option<TokenKind> {
        match self {
            Self::V1 => v1::classify_identifier(lexeme),
            Self::V2 => v2::classify_identifier(lexeme),
            Self::V3 => v3::classify_identifier(lexeme),
            Self::V4 => v4::classify_identifier(lexeme),
        }
    }

    pub(crate) fn unsupported_operator_profile(self, lexeme: &str) -> Option<&'static str> {
        match self {
            Self::V1 => v1::unsupported_operator_profile(lexeme),
            Self::V2 => v2::unsupported_operator_profile(lexeme),
            Self::V3 => v3::unsupported_operator_profile(lexeme),
            Self::V4 => None,
        }
    }

    pub(crate) fn permits_strong_previous(self) -> bool {
        matches!(self, Self::V3 | Self::V4)
    }

    pub(crate) fn atom_keyword_boundary(self, next: u8, followed_by_bracket: bool) -> bool {
        match self {
            Self::V1 => v1::atom_keyword_boundary(next, followed_by_bracket),
            Self::V2 => v2::atom_keyword_boundary(next, followed_by_bracket),
            Self::V3 => v3::atom_keyword_boundary(next, followed_by_bracket),
            Self::V4 => v4::atom_keyword_boundary(next, followed_by_bracket),
        }
    }

    pub(crate) fn binary_binding_power(self, token: TokenKind) -> Option<(u8, u8)> {
        match self {
            Self::V1 => v1::binary_binding_power(token),
            Self::V2 => v2::binary_binding_power(token),
            Self::V3 => v3::binary_binding_power(token),
            Self::V4 => v4::binary_binding_power(token),
        }
    }

    pub(crate) fn permits_temporal_prefix(self, token: TokenKind) -> bool {
        match self {
            Self::V1 => v1::permits_temporal_prefix(token),
            Self::V2 => v2::permits_temporal_prefix(token),
            Self::V3 => v3::permits_temporal_prefix(token),
            Self::V4 => v4::permits_temporal_prefix(token),
        }
    }

    pub(crate) fn build_document(
        self,
        profile: SemanticProfile,
        root: NodeId,
        nodes: Vec<Node>,
    ) -> Result<FormulaDocument, FormulaError> {
        match self {
            Self::V1 => v1::build_document(profile, root, nodes),
            Self::V2 => v2::build_document(profile, root, nodes),
            Self::V3 => v3::build_document(profile, root, nodes),
            Self::V4 => v1::build_document(profile, root, nodes),
        }
    }

    pub(crate) fn permits_lowering(self) -> bool {
        matches!(self, Self::V2)
    }

    pub(crate) const fn for_formula_profile(profile: SemanticProfile) -> Self {
        if matches!(profile, SemanticProfile::OriginCompleteHistoryV1) {
            Self::V3
        } else {
            Self::V1
        }
    }

    pub(crate) fn unary_spelling(self, kind: NodeKind) -> Option<UnarySpelling> {
        match self {
            Self::V1 => v1::unary_spelling(kind),
            Self::V2 => v2::unary_spelling(kind),
            Self::V3 => v3::unary_spelling(kind),
            Self::V4 => None,
        }
    }

    pub(crate) fn binary_spelling(self, kind: NodeKind) -> Option<BinarySpelling> {
        match self {
            Self::V1 => v1::binary_spelling(kind),
            Self::V2 => v2::binary_spelling(kind),
            Self::V3 => v3::binary_spelling(kind),
            Self::V4 => None,
        }
    }

    pub(crate) fn precedence(self, kind: NodeKind) -> Option<u8> {
        match kind {
            NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => Some(7),
            _ => self.unary_spelling(kind).map(|_| 6).or_else(|| {
                self.binary_spelling(kind)
                    .map(|operator| operator.precedence)
            }),
        }
    }
}
