//! Lexer and bounded parser for `tl-parse.clean-ascii/v6` (FR-058, FR-059).
//!
//! Keywords `F G U R X Xs Xw` are case-sensitive and lexed by maximal munch,
//! so `Gp0` is `G p0` and `Xsp0` is `Xs p0`. Atoms, constants, Boolean
//! operators, grouping and precedence are those of v1.

use super::{
    model::{to_infinite, to_ltlf, Item, Kind},
    LtlDialectRevision, LtlDiagnosticCode as Code, LtlDocument, LtlParseReport,
    LtlParseSchemaVersion, LtlProfile, Sink,
};
use crate::{lexer::checked_span, ParseLimits, ParseStats};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tok {
    False,
    True,
    Proposition(u32),
    /// A bare digit run; meaningful only inside a refused interval.
    Number,
    Not,
    And,
    Or,
    Implies,
    Equivalent,
    Future,
    Globally,
    Until,
    Release,
    Next,
    StrongNext,
    WeakNext,
    /// `W M O H Y S T`.
    Unsupported,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    Comma,
    Invalid,
    Eof,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Token {
    pub(crate) kind: Tok,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) struct Lexed {
    pub(crate) tokens: Vec<Token>,
    pub(crate) work: usize,
}

const fn is_word(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub(crate) fn lex(source: &str, limits: ParseLimits, sink: &mut Sink) -> Lexed {
    let bytes = source.as_bytes();
    let mut tokens: Vec<Token> = Vec::new();
    let mut offset = 0_usize;
    let mut work = 0_usize;
    let mut stopped = false;
    if source.len() > limits.max_source_bytes {
        let start = limits.max_source_bytes.min(source.len());
        sink.push(
            Code::SourceLimit,
            start,
            next_char_end(source, start),
            format!(
                "source length {} exceeds effective limit {}",
                source.len(),
                limits.max_source_bytes
            ),
        );
        stopped = true;
    }
    while offset < bytes.len() && !stopped {
        if work >= limits.max_work {
            sink.push(
                Code::WorkLimit,
                offset,
                next_char_end(source, offset),
                format!("lexer work exceeds effective limit {}", limits.max_work),
            );
            break;
        }
        work += 1;
        let byte = bytes[offset];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            offset += 1;
            continue;
        }
        if tokens.len() >= limits.max_tokens {
            sink.push(
                Code::TokenLimit,
                offset,
                next_char_end(source, offset),
                format!("token count exceeds effective limit {}", limits.max_tokens),
            );
            break;
        }
        let start = offset;
        let rest = &source[start..];
        let push = |kind: Tok, end: usize, tokens: &mut Vec<Token>| {
            tokens.push(Token { kind, start, end });
            end
        };
        if rest.starts_with("<->") {
            offset = push(Tok::Equivalent, start + 3, &mut tokens);
            continue;
        }
        if rest.starts_with("->") {
            offset = push(Tok::Implies, start + 2, &mut tokens);
            continue;
        }
        let single = match byte {
            b'!' => Some(Tok::Not),
            b'&' => Some(Tok::And),
            b'|' => Some(Tok::Or),
            b'(' => Some(Tok::LeftParenthesis),
            b')' => Some(Tok::RightParenthesis),
            b'[' => Some(Tok::LeftBracket),
            b']' => Some(Tok::RightBracket),
            b',' => Some(Tok::Comma),
            b'F' => Some(Tok::Future),
            b'G' => Some(Tok::Globally),
            b'U' => Some(Tok::Until),
            b'R' => Some(Tok::Release),
            b'W' | b'M' | b'O' | b'H' | b'Y' | b'S' | b'T' => Some(Tok::Unsupported),
            _ => None,
        };
        if let Some(kind) = single {
            offset = push(kind, start + 1, &mut tokens);
            continue;
        }
        if byte == b'X' {
            let next = bytes.get(start + 1).copied();
            let after = bytes.get(start + 2).copied();
            match next {
                Some(b's') => offset = push(Tok::StrongNext, start + 2, &mut tokens),
                Some(b'w') => offset = push(Tok::WeakNext, start + 2, &mut tokens),
                Some(b'p') if after.is_some_and(|value| value.is_ascii_digit()) => {
                    offset = push(Tok::Next, start + 1, &mut tokens);
                }
                Some(b't' | b'f')
                    if ["true", "false"].into_iter().any(|word| {
                        rest[1..].starts_with(word)
                            && !bytes.get(start + 1 + word.len()).is_some_and(|b| is_word(*b))
                    }) =>
                {
                    // `Xtrue` is `X true` by maximal munch.
                    offset = push(Tok::Next, start + 1, &mut tokens);
                }
                Some(value) if value.is_ascii_lowercase() || value == b'_' => {
                    // `Xx`: an unknown prefix, refused as one word.
                    let mut end = start + 1;
                    while end < bytes.len() && is_word(bytes[end]) {
                        end += 1;
                    }
                    sink.push(
                        Code::UnknownIdentifier,
                        start,
                        end,
                        format!("{:?} is not a v6 keyword", &source[start..end]),
                    );
                    offset = push(Tok::Invalid, end, &mut tokens);
                }
                _ => offset = push(Tok::Next, start + 1, &mut tokens),
            }
            continue;
        }
        if byte == b'p' && bytes.get(start + 1).is_some_and(u8::is_ascii_digit) {
            let mut end = start + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            let digits = &source[start + 1..end];
            if digits.len() > 1 && digits.starts_with('0') {
                sink.push(
                    Code::NonCanonicalNumber,
                    start,
                    end,
                    format!("atom {:?} has a leading zero", &source[start..end]),
                );
                offset = push(Tok::Invalid, end, &mut tokens);
            } else if let Ok(value) = digits.parse::<u32>() {
                offset = push(Tok::Proposition(value), end, &mut tokens);
            } else {
                sink.push(
                    Code::IntegerOverflow,
                    start,
                    end,
                    format!("atom {:?} exceeds u32", &source[start..end]),
                );
                offset = push(Tok::Invalid, end, &mut tokens);
            }
            continue;
        }
        if byte.is_ascii_digit() {
            let mut end = start;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            offset = push(Tok::Number, end, &mut tokens);
            continue;
        }
        let constant = [("true", Tok::True), ("false", Tok::False)]
            .into_iter()
            .find(|(word, _)| {
                rest.starts_with(word)
                    && !bytes.get(start + word.len()).is_some_and(|b| is_word(*b))
            });
        if let Some((word, kind)) = constant {
            offset = push(kind, start + word.len(), &mut tokens);
            continue;
        }
        if is_word(byte) {
            let mut end = start;
            while end < bytes.len() && is_word(bytes[end]) {
                end += 1;
            }
            sink.push(
                Code::UnknownIdentifier,
                start,
                end,
                format!("{:?} is not a v6 keyword or atom", &source[start..end]),
            );
            offset = push(Tok::Invalid, end, &mut tokens);
            continue;
        }
        let end = next_char_end(source, start);
        sink.push(
            Code::UnexpectedCharacter,
            start,
            end,
            format!("character {:?} is outside the ASCII dialect", &source[start..end]),
        );
        offset = push(Tok::Invalid, end, &mut tokens);
    }
    let eof = offset.min(source.len());
    tokens.push(Token {
        kind: Tok::Eof,
        start: eof,
        end: eof,
    });
    Lexed { tokens, work }
}

fn next_char_end(source: &str, start: usize) -> usize {
    source
        .get(start..)
        .and_then(|rest| rest.chars().next())
        .map_or(start, |character| start + character.len_utf8())
}

pub(crate) fn parse(source: &str, profile: LtlProfile, limits: ParseLimits) -> LtlParseReport {
    let limits = limits.clamped();
    let mut report = LtlParseReport {
        schema_version: LtlParseSchemaVersion::V1,
        dialect_revision: LtlDialectRevision::V6,
        profile,
        limits,
        stats: ParseStats {
            source_bytes: source.len(),
            ..ParseStats::default()
        },
        document: None,
        diagnostics: Vec::new(),
        refusal: None,
    };
    let mut sink = Sink::new(limits.max_diagnostics);
    let lexed = lex(source, limits, &mut sink);
    report.stats.tokens = lexed.tokens.len().saturating_sub(1);
    report.stats.work = lexed.work;
    if sink.has_error() {
        return finish(report, sink);
    }
    let mut parser = Parser {
        profile,
        tokens: lexed.tokens,
        cursor: 0,
        items: Vec::new(),
        limits,
        work: lexed.work,
        max_depth: 0,
        stopped: false,
        sink,
    };
    let root = parser.parse_formula(1, 1);
    if !parser.stopped {
        let token = parser.current();
        if token.kind != Tok::Eof {
            parser.fail::<()>(
                Code::TrailingInput,
                token.start,
                token.end,
                "unexpected input after a complete formula".to_owned(),
            );
        }
    }
    if root.is_none() && !parser.sink.has_error() {
        parser.sink.push(
            Code::ValidationFailure,
            0,
            0,
            "v6 parser stopped without a typed refusal".to_owned(),
        );
    }
    report.stats.nodes = parser.items.len();
    report.stats.work = parser.work;
    report.stats.max_depth = parser.max_depth;
    let Some(root) = root.filter(|_| !parser.sink.has_error()) else {
        return finish(report, parser.sink);
    };
    let built = match profile {
        LtlProfile::LtlfFiniteTrace => to_ltlf(root.index, &parser.items).map(LtlDocument::Ltlf),
        LtlProfile::InfiniteTrace => {
            to_infinite(root.index, &parser.items).map(LtlDocument::Infinite)
        }
    };
    match built {
        Ok(document) => report.document = Some(document),
        Err(message) => parser.sink.push(Code::ValidationFailure, root.start, root.end, message),
    }
    finish(report, parser.sink)
}

fn finish(mut report: LtlParseReport, sink: Sink) -> LtlParseReport {
    report.stats.diagnostics_truncated = sink.truncated;
    report.refusal = sink.refusal;
    report.diagnostics = sink.diagnostics;
    report.stats.diagnostics = report.diagnostics.len();
    if report.refusal.is_some() {
        report.document = None;
    }
    report
}

#[derive(Clone, Copy)]
struct Parsed {
    index: usize,
    start: usize,
    end: usize,
}

struct Parser {
    profile: LtlProfile,
    tokens: Vec<Token>,
    cursor: usize,
    items: Vec<Item>,
    limits: ParseLimits,
    work: usize,
    max_depth: usize,
    stopped: bool,
    sink: Sink,
}

/// v1 binding powers: `<->` left, `->` right, `|`, `&`, then `U`/`R` left.
const fn binding_power(kind: Tok) -> Option<(u8, u8)> {
    match kind {
        Tok::Equivalent => Some((1, 2)),
        Tok::Implies => Some((2, 2)),
        Tok::Or => Some((3, 4)),
        Tok::And => Some((4, 5)),
        Tok::Until | Tok::Release => Some((5, 6)),
        _ => None,
    }
}

impl Parser {
    fn parse_formula(&mut self, minimum: u8, depth: usize) -> Option<Parsed> {
        if !self.enter(depth) {
            return None;
        }
        let mut left = self.parse_prefix(depth)?;
        loop {
            if self.stopped {
                return None;
            }
            let operator = self.current();
            if operator.kind == Tok::Unsupported {
                return self.unsupported(operator);
            }
            let Some((left_power, right_power)) = binding_power(operator.kind) else {
                break;
            };
            if left_power < minimum {
                break;
            }
            self.advance();
            if matches!(operator.kind, Tok::Until | Tok::Release) {
                self.refuse_interval()?;
            }
            let right = self.parse_formula(right_power, depth.saturating_add(1))?;
            let kind = match operator.kind {
                Tok::Equivalent => Kind::Equivalent(left.index, right.index),
                Tok::Implies => Kind::Implies(left.index, right.index),
                Tok::Or => Kind::Or(left.index, right.index),
                Tok::And => Kind::And(left.index, right.index),
                Tok::Until => Kind::Until(left.index, right.index),
                Tok::Release => Kind::Release(left.index, right.index),
                _ => return None,
            };
            left = self.push(kind, left.start, right.end)?;
        }
        Some(left)
    }

    fn parse_prefix(&mut self, depth: usize) -> Option<Parsed> {
        if !self.enter(depth) {
            return None;
        }
        let token = self.current();
        match token.kind {
            Tok::False => {
                self.advance();
                self.push(Kind::False, token.start, token.end)
            }
            Tok::True => {
                self.advance();
                self.push(Kind::True, token.start, token.end)
            }
            Tok::Proposition(value) => {
                self.advance();
                self.push(Kind::Proposition(value), token.start, token.end)
            }
            Tok::Not
            | Tok::Future
            | Tok::Globally
            | Tok::Next
            | Tok::StrongNext
            | Tok::WeakNext => {
                match (token.kind, self.profile) {
                    (Tok::Next, LtlProfile::LtlfFiniteTrace) => {
                        return self.fail(
                            Code::NextStrengthRequired,
                            token.start,
                            token.end,
                            "LTLf has no bare X; write Xs (strong next) or Xw (weak next)"
                                .to_owned(),
                        )
                    }
                    (Tok::StrongNext | Tok::WeakNext, LtlProfile::InfiniteTrace) => {
                        return self.fail(
                            Code::NextStrengthUndefined,
                            token.start,
                            token.end,
                            "Xs and Xw are LTLf-only; the infinite profile has no end of \
                             trace, write X"
                                .to_owned(),
                        )
                    }
                    _ => {}
                }
                self.advance();
                if token.kind != Tok::Not {
                    self.refuse_interval()?;
                }
                let operand = self.parse_prefix(depth.saturating_add(1))?;
                let kind = match token.kind {
                    Tok::Not => Kind::Not(operand.index),
                    Tok::Future => Kind::Future(operand.index),
                    Tok::Globally => Kind::Globally(operand.index),
                    Tok::Next => Kind::Next(operand.index),
                    Tok::StrongNext => Kind::StrongNext(operand.index),
                    _ => Kind::WeakNext(operand.index),
                };
                self.push(kind, token.start, operand.end)
            }
            Tok::LeftParenthesis => {
                self.advance();
                let inner = self.parse_formula(1, depth.saturating_add(1))?;
                let closing = self.current();
                if closing.kind != Tok::RightParenthesis {
                    return self.fail(
                        Code::MissingToken,
                        closing.start,
                        closing.end,
                        "expected `)`".to_owned(),
                    );
                }
                self.advance();
                Some(Parsed {
                    index: inner.index,
                    start: token.start,
                    end: closing.end,
                })
            }
            Tok::Unsupported => self.unsupported(token),
            Tok::LeftBracket => {
                self.refuse_interval()?;
                None
            }
            _ => self.fail(
                Code::UnexpectedToken,
                token.start,
                token.end,
                "expected a formula".to_owned(),
            ),
        }
    }

    fn unsupported<T>(&mut self, token: Token) -> Option<T> {
        self.fail(
            Code::UnsupportedOperator,
            token.start,
            token.end,
            "W, M, O, H, Y, S and T are outside tl-parse.clean-ascii/v6".to_owned(),
        )
    }

    /// Refuses an interval bracket at the cursor, spanning `[` through the
    /// next `]` or `)` (one forward scan, then the parse stops).
    fn refuse_interval(&mut self) -> Option<()> {
        let opening = self.current();
        if opening.kind != Tok::LeftBracket {
            return Some(());
        }
        let mut end = opening.end;
        for token in self.tokens.iter().skip(self.cursor + 1) {
            match token.kind {
                Tok::RightBracket | Tok::RightParenthesis => {
                    end = token.end;
                    break;
                }
                Tok::Number | Tok::Comma => end = token.end,
                _ => break,
            }
        }
        self.fail(
            Code::IntervalNotInDialect,
            opening.start,
            end,
            "intervals are not spelled in v6; select tl-parse.clean-ascii/v1 to v4 for intervals"
                .to_owned(),
        )
    }

    fn push(&mut self, kind: Kind, start: usize, end: usize) -> Option<Parsed> {
        if self.items.len() >= self.limits.max_nodes {
            return self.fail(
                Code::NodeLimit,
                start,
                end,
                format!("node count exceeds effective limit {}", self.limits.max_nodes),
            );
        }
        if !self.charge(start, end) {
            return None;
        }
        self.items.push(Item {
            kind,
            span: Some(checked_span(start, end)),
        });
        Some(Parsed {
            index: self.items.len() - 1,
            start,
            end,
        })
    }

    fn enter(&mut self, depth: usize) -> bool {
        self.max_depth = self.max_depth.max(depth);
        let token = self.current();
        if depth > self.limits.max_depth {
            self.fail::<()>(
                Code::DepthLimit,
                token.start,
                token.end,
                format!("nesting exceeds effective limit {}", self.limits.max_depth),
            );
            return false;
        }
        self.charge(token.start, token.end)
    }

    fn charge(&mut self, start: usize, end: usize) -> bool {
        if self.stopped {
            return false;
        }
        if self.work >= self.limits.max_work {
            self.fail::<()>(
                Code::WorkLimit,
                start,
                end,
                format!("parser work exceeds effective limit {}", self.limits.max_work),
            );
            return false;
        }
        self.work += 1;
        true
    }

    fn current(&self) -> Token {
        self.tokens[self.cursor.min(self.tokens.len().saturating_sub(1))]
    }

    fn advance(&mut self) {
        let token = self.current();
        if self.charge(token.start, token.end) && self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
    }

    fn fail<T>(&mut self, code: Code, start: usize, end: usize, message: String) -> Option<T> {
        if !self.stopped {
            self.sink.push(code, start, end, message);
            self.stopped = true;
        }
        None
    }
}
