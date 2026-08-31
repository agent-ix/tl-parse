use tl_syntax::SourceSpan;

use crate::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, ExpectedToken, ParseLimits, RecoveryAction,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    False,
    True,
    Proposition(u32),
    Number(u32),
    Not,
    And,
    Or,
    Implies,
    Equivalent,
    Future,
    Globally,
    Until,
    Release,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    Comma,
    Invalid,
    Eof,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Token {
    pub(crate) fn span(self) -> SourceSpan {
        checked_span(self.start, self.end)
    }

    pub(crate) fn found(self, source: &str) -> String {
        if self.kind == TokenKind::Eof {
            return "<eof>".to_owned();
        }
        let lexeme = &source[self.start..self.end];
        let mut clipped = String::new();
        for character in lexeme.chars().take(32) {
            clipped.push(character);
        }
        if lexeme.chars().count() > 32 {
            clipped.push('…');
        }
        format!("{clipped:?}")
    }
}

pub(crate) struct LexResult {
    pub(crate) tokens: Vec<Token>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) diagnostics_truncated: bool,
    pub(crate) had_error: bool,
    pub(crate) work: usize,
}

struct Lexer<'a> {
    source: &'a str,
    limits: ParseLimits,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    diagnostics_truncated: bool,
    had_error: bool,
    work: usize,
    stopped: bool,
}

pub(crate) fn lex(source: &str, limits: ParseLimits) -> LexResult {
    let mut lexer = Lexer {
        source,
        limits,
        offset: 0,
        tokens: Vec::new(),
        diagnostics: Vec::new(),
        diagnostics_truncated: false,
        had_error: false,
        work: 0,
        stopped: false,
    };

    if source.len() > limits.max_source_bytes {
        let start = limits.max_source_bytes.min(source.len());
        let end = start.saturating_add(1).min(source.len());
        lexer.push_diagnostic(
            DiagnosticCode::SourceLimit,
            start,
            end,
            "<source>",
            Vec::new(),
            RecoveryAction::Stopped,
            format!(
                "source length {} exceeds effective limit {}",
                source.len(),
                limits.max_source_bytes
            ),
        );
        lexer.stopped = true;
    }

    while lexer.offset < source.len() && !lexer.stopped {
        if !lexer.charge_work() {
            break;
        }
        let byte = source.as_bytes()[lexer.offset];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            lexer.offset += 1;
            continue;
        }
        if lexer.tokens.len() >= limits.max_tokens {
            let start = lexer.offset;
            let end = next_char_end(source, start);
            lexer.push_diagnostic(
                DiagnosticCode::TokenLimit,
                start,
                end,
                "<token>",
                Vec::new(),
                RecoveryAction::Stopped,
                format!("token count exceeds effective limit {}", limits.max_tokens),
            );
            lexer.stopped = true;
            break;
        }
        lexer.scan_token();
    }

    let eof = lexer.offset.min(source.len());
    lexer.tokens.push(Token {
        kind: TokenKind::Eof,
        start: eof,
        end: eof,
    });
    LexResult {
        tokens: lexer.tokens,
        diagnostics: lexer.diagnostics,
        diagnostics_truncated: lexer.diagnostics_truncated,
        had_error: lexer.had_error,
        work: lexer.work,
    }
}

impl Lexer<'_> {
    fn charge_work(&mut self) -> bool {
        if self.work >= self.limits.max_work {
            let start = self.offset.min(self.source.len());
            let end = next_char_end(self.source, start);
            self.push_diagnostic(
                DiagnosticCode::WorkLimit,
                start,
                end,
                "<work>",
                Vec::new(),
                RecoveryAction::Stopped,
                format!(
                    "lexer/parser work exceeds effective limit {}",
                    self.limits.max_work
                ),
            );
            self.stopped = true;
            false
        } else {
            self.work += 1;
            true
        }
    }

    fn scan_token(&mut self) {
        let start = self.offset;
        let rest = &self.source[start..];
        if rest.starts_with("<->") {
            self.offset += 3;
            self.push_token(TokenKind::Equivalent, start);
            return;
        }
        if rest.starts_with("->") {
            self.offset += 2;
            self.push_token(TokenKind::Implies, start);
            return;
        }

        let byte = self.source.as_bytes()[start];
        let single = match byte {
            b'!' => Some(TokenKind::Not),
            b'&' => Some(TokenKind::And),
            b'|' => Some(TokenKind::Or),
            b'(' => Some(TokenKind::LeftParenthesis),
            b')' => Some(TokenKind::RightParenthesis),
            b'[' => Some(TokenKind::LeftBracket),
            b']' => Some(TokenKind::RightBracket),
            b',' => Some(TokenKind::Comma),
            _ => None,
        };
        if let Some(kind) = single {
            self.offset += 1;
            self.push_token(kind, start);
            return;
        }

        if byte == b'p'
            && self
                .source
                .as_bytes()
                .get(start + 1)
                .is_some_and(u8::is_ascii_digit)
        {
            self.offset += 1;
            let digit_start = self.offset;
            while self.offset < self.source.len()
                && self.source.as_bytes()[self.offset].is_ascii_digit()
            {
                self.offset += 1;
            }
            let digits = &self.source[digit_start..self.offset];
            self.finish_numeric_token(start, digits, true);
            return;
        }
        for (keyword, kind) in [("false", TokenKind::False), ("true", TokenKind::True)] {
            if rest.starts_with(keyword) && self.atom_keyword_boundary(start + keyword.len()) {
                self.offset += keyword.len();
                self.push_token(kind, start);
                return;
            }
        }

        if byte.is_ascii_alphabetic() || byte == b'_' {
            self.scan_identifier(start);
        } else if byte.is_ascii_digit() {
            self.scan_number(start, false);
        } else {
            self.offset = next_char_end(self.source, start);
            let found = Token {
                kind: TokenKind::Invalid,
                start,
                end: self.offset,
            }
            .found(self.source);
            self.push_diagnostic(
                DiagnosticCode::UnexpectedCharacter,
                start,
                self.offset,
                &found,
                Vec::new(),
                RecoveryAction::SkippedToken,
                format!("character {found} is outside the ASCII dialect"),
            );
            self.push_token(TokenKind::Invalid, start);
        }
    }

    fn atom_keyword_boundary(&self, end: usize) -> bool {
        match self.source.as_bytes().get(end) {
            None => true,
            Some(byte) if !byte.is_ascii_alphanumeric() && *byte != b'_' => true,
            Some(b'U' | b'R') => self.source.as_bytes().get(end + 1) == Some(&b'['),
            Some(_) => false,
        }
    }

    fn scan_identifier(&mut self, start: usize) {
        while self.offset < self.source.len() {
            let byte = self.source.as_bytes()[self.offset];
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                self.offset += 1;
            } else {
                break;
            }
        }
        let lexeme = &self.source[start..self.offset];
        let kind = match lexeme {
            "false" => Some(TokenKind::False),
            "true" => Some(TokenKind::True),
            "F" => Some(TokenKind::Future),
            "G" => Some(TokenKind::Globally),
            "U" => Some(TokenKind::Until),
            "R" => Some(TokenKind::Release),
            _ => None,
        };
        if let Some(kind) = kind {
            self.push_token(kind, start);
            return;
        }
        if let Some(digits) = lexeme.strip_prefix('p') {
            if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
                self.finish_numeric_token(start, digits, true);
                return;
            }
        }
        let found = Token {
            kind: TokenKind::Invalid,
            start,
            end: self.offset,
        }
        .found(self.source);
        self.push_diagnostic(
            DiagnosticCode::UnknownIdentifier,
            start,
            self.offset,
            &found,
            vec![ExpectedToken::Expression],
            RecoveryAction::SkippedToken,
            format!("identifier {found} is outside the closed dialect"),
        );
        self.push_token(TokenKind::Invalid, start);
    }

    fn scan_number(&mut self, start: usize, proposition: bool) {
        while self.offset < self.source.len()
            && self.source.as_bytes()[self.offset].is_ascii_digit()
        {
            self.offset += 1;
        }
        let digits = &self.source[start..self.offset];
        self.finish_numeric_token(start, digits, proposition);
    }

    fn finish_numeric_token(&mut self, start: usize, digits: &str, proposition: bool) {
        let kind_for = |value| {
            if proposition {
                TokenKind::Proposition(value)
            } else {
                TokenKind::Number(value)
            }
        };
        if digits.len() > 1 && digits.starts_with('0') {
            let found = Token {
                kind: TokenKind::Invalid,
                start,
                end: self.offset,
            }
            .found(self.source);
            self.push_diagnostic(
                DiagnosticCode::NonCanonicalNumber,
                start,
                self.offset,
                &found,
                vec![ExpectedToken::Integer],
                RecoveryAction::SkippedToken,
                format!("number {found} has a forbidden leading zero"),
            );
            self.push_token(TokenKind::Invalid, start);
        } else if let Ok(value) = digits.parse::<u32>() {
            self.push_token(kind_for(value), start);
        } else {
            let found = Token {
                kind: TokenKind::Invalid,
                start,
                end: self.offset,
            }
            .found(self.source);
            self.push_diagnostic(
                DiagnosticCode::IntegerOverflow,
                start,
                self.offset,
                &found,
                vec![ExpectedToken::Integer],
                RecoveryAction::SkippedToken,
                format!("number {found} exceeds unsigned 32-bit range"),
            );
            self.push_token(TokenKind::Invalid, start);
        }
    }

    fn push_token(&mut self, kind: TokenKind, start: usize) {
        self.tokens.push(Token {
            kind,
            start,
            end: self.offset,
        });
    }

    // Keeping every diagnostic field explicit here makes call-site recovery
    // behavior reviewable; a builder would obscure this closed internal path.
    #[allow(clippy::too_many_arguments)]
    fn push_diagnostic(
        &mut self,
        code: DiagnosticCode,
        start: usize,
        end: usize,
        found: &str,
        expected: Vec<ExpectedToken>,
        recovery: RecoveryAction,
        message: String,
    ) {
        self.had_error = true;
        if self.diagnostics.len() >= self.limits.max_diagnostics {
            self.diagnostics_truncated = true;
            return;
        }
        self.diagnostics.push(Diagnostic {
            code,
            severity: DiagnosticSeverity::Error,
            span: checked_span(start, end),
            found: found.to_owned(),
            expected,
            recovery,
            message,
        });
    }
}

pub(crate) fn checked_span(start: usize, end: usize) -> SourceSpan {
    match SourceSpan::new(start as u32, end as u32) {
        Ok(span) => span,
        Err(_) => match SourceSpan::new(0, 0) {
            Ok(span) => span,
            Err(_) => unreachable!("zero source span is valid"),
        },
    }
}

fn next_char_end(source: &str, start: usize) -> usize {
    source
        .get(start..)
        .and_then(|rest| rest.chars().next())
        .map_or(start, |character| start + character.len_utf8())
}
