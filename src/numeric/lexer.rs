//! Bounded lexer for `tl-parse.clean-ascii/v5` (FR-029 lexical rules).
//!
//! Minus is lexed by position (ADR-020 decision 3): `-` begins a negative
//! literal only where an operand is expected and a digit follows immediately.
//! After an operand (identifier, number, proposition, constant or `)`) it is
//! always binary subtraction.

use super::{NumericDiagnosticCode as Code, Sink, MAX_NUMERIC_IDENTIFIER_BYTES, MAX_NUMERIC_SCALE};
use crate::ParseLimits;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Keyword {
    True,
    False,
    Future,
    Globally,
    Until,
    Release,
    WeakUntil,
    StrongRelease,
    Abs,
    Scale,
    /// `each some atleast in set contract assume guarantee` (TL-424/TL-425 forms).
    Aggregation,
    /// Past, next and fairness spellings refused in v5.
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tok {
    Identifier,
    Keyword(Keyword),
    Proposition(u32),
    /// Exact literal: coefficient and scale (`5` is (5, 0); `5.00` is (500, 2)).
    Number { coefficient: i64, scale: u8 },
    Not,
    And,
    Or,
    Implies,
    Equivalent,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    Comma,
    Plus,
    Minus,
    Star,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,
    /// A lexical error already reported; the parser never runs after one.
    Invalid,
    Eof,
}

impl Tok {
    pub(crate) const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Less
                | Self::LessEqual
                | Self::Greater
                | Self::GreaterEqual
                | Self::EqualEqual
                | Self::NotEqual
        )
    }

    /// Tokens after which `-` is binary subtraction.
    const fn ends_operand(self) -> bool {
        matches!(
            self,
            Self::Identifier
                | Self::Proposition(_)
                | Self::Number { .. }
                | Self::RightParenthesis
                | Self::Keyword(Keyword::True | Keyword::False)
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Token {
    pub(crate) kind: Tok,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) struct Lexed {
    pub(crate) tokens: Vec<Token>,
    /// For each `(` token index: the matching `)` exists and a comparison occurs
    /// at that parenthesis's own depth before it. One pass, no backtracking.
    pub(crate) opens_comparison: Vec<bool>,
    /// For each `(` token index: a formula operator occurs at its own depth
    /// before the first comparison there (`(p1 & x > 0)`).
    pub(crate) mixed_formula: Vec<bool>,
    /// For each `(` token index: end offset of the first comparison operator
    /// at its own depth (0 when there is none).
    pub(crate) comparison_end: Vec<usize>,
    /// Largest explicit `p<digits>` identity in the source.
    pub(crate) max_explicit_proposition: Option<u32>,
    pub(crate) work: usize,
    pub(crate) stopped: bool,
}

const RESERVED_AGGREGATION: [&str; 8] = [
    "each",
    "some",
    "atleast",
    "in",
    "set",
    "contract",
    "assume",
    "guarantee",
];

// SPEC-GAP: FR-029 lists the reserved words but not the single-letter operator
// keywords. Since `F`, `G`, `U`, `R`, `W`, `M` lex as operators and `X`, `O`,
// `H`, `Y`, `S`, `T` as refused operators, none of them can name a signal in v5.
const UNSUPPORTED_WORDS: [&str; 9] = ["X", "Xs", "Xw", "O", "H", "Y", "S", "T", "fair"];

fn classify(lexeme: &str) -> Option<Keyword> {
    let keyword = match lexeme {
        "true" => Keyword::True,
        "false" => Keyword::False,
        "F" => Keyword::Future,
        "G" => Keyword::Globally,
        "U" => Keyword::Until,
        "R" => Keyword::Release,
        "W" => Keyword::WeakUntil,
        "M" => Keyword::StrongRelease,
        "abs" => Keyword::Abs,
        "scale" => Keyword::Scale,
        _ if RESERVED_AGGREGATION.contains(&lexeme) => Keyword::Aggregation,
        _ if UNSUPPORTED_WORDS.contains(&lexeme) => Keyword::Unsupported,
        _ => return None,
    };
    Some(keyword)
}

struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    offset: usize,
    tokens: Vec<Token>,
    work: usize,
    stopped: bool,
    max_explicit_proposition: Option<u32>,
}

pub(crate) fn lex(source: &str, limits: ParseLimits, sink: &mut Sink) -> Lexed {
    let mut lexer = Lexer {
        source,
        bytes: source.as_bytes(),
        offset: 0,
        tokens: Vec::new(),
        work: 0,
        stopped: false,
        max_explicit_proposition: None,
    };
    if source.len() > limits.max_source_bytes {
        let start = limits.max_source_bytes.min(source.len());
        let end = next_char_end(source, start);
        sink.push(
            Code::SourceLimit,
            start,
            end,
            format!(
                "source length {} exceeds effective limit {}",
                source.len(),
                limits.max_source_bytes
            ),
        );
        lexer.stopped = true;
    }
    while lexer.offset < lexer.bytes.len() && !lexer.stopped {
        if lexer.work >= limits.max_work {
            let start = lexer.offset;
            sink.push(
                Code::WorkLimit,
                start,
                next_char_end(source, start),
                format!("lexer work exceeds effective limit {}", limits.max_work),
            );
            lexer.stopped = true;
            break;
        }
        lexer.work += 1;
        let byte = lexer.bytes[lexer.offset];
        if matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
            lexer.offset += 1;
            continue;
        }
        if lexer.tokens.len() >= limits.max_tokens {
            let start = lexer.offset;
            sink.push(
                Code::TokenLimit,
                start,
                next_char_end(source, start),
                format!("token count exceeds effective limit {}", limits.max_tokens),
            );
            lexer.stopped = true;
            break;
        }
        lexer.scan(sink);
    }
    let eof = lexer.offset.min(source.len());
    lexer.tokens.push(Token {
        kind: Tok::Eof,
        start: eof,
        end: eof,
    });
    let marks = comparison_parentheses(&lexer.tokens);
    Lexed {
        tokens: lexer.tokens,
        opens_comparison: marks.opens_comparison,
        mixed_formula: marks.mixed_formula,
        comparison_end: marks.comparison_end,
        max_explicit_proposition: lexer.max_explicit_proposition,
        work: lexer.work,
        stopped: lexer.stopped,
    }
}

struct ParenthesisMarks {
    opens_comparison: Vec<bool>,
    mixed_formula: Vec<bool>,
    comparison_end: Vec<usize>,
}

const fn is_formula_operator(kind: Tok) -> bool {
    matches!(
        kind,
        Tok::Not
            | Tok::And
            | Tok::Or
            | Tok::Implies
            | Tok::Equivalent
            | Tok::Keyword(
                Keyword::Future
                    | Keyword::Globally
                    | Keyword::Until
                    | Keyword::Release
                    | Keyword::WeakUntil
                    | Keyword::StrongRelease
            )
    )
}

/// One stack pass: marks each `(` whose own depth holds a comparison operator,
/// whether a formula operator precedes that comparison at the same depth, and
/// where the first such comparison ends. No backtracking.
fn comparison_parentheses(tokens: &[Token]) -> ParenthesisMarks {
    let mut marks = ParenthesisMarks {
        opens_comparison: vec![false; tokens.len()],
        mixed_formula: vec![false; tokens.len()],
        comparison_end: vec![0; tokens.len()],
    };
    let mut formula_seen = vec![false; tokens.len()];
    let mut open: Vec<usize> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            Tok::LeftParenthesis => open.push(index),
            Tok::RightParenthesis => {
                open.pop();
            }
            kind if kind.is_comparison() => {
                if let Some(top) = open.last().copied() {
                    if !marks.opens_comparison[top] {
                        marks.opens_comparison[top] = true;
                        marks.comparison_end[top] = token.end;
                        marks.mixed_formula[top] = formula_seen[top];
                    }
                }
            }
            kind if is_formula_operator(kind) => {
                if let Some(top) = open.last().copied() {
                    if !marks.opens_comparison[top] {
                        formula_seen[top] = true;
                    }
                }
            }
            _ => {}
        }
    }
    marks
}

impl Lexer<'_> {
    fn previous_ends_operand(&self) -> bool {
        self.tokens.last().is_some_and(|token| token.kind.ends_operand())
    }

    fn push(&mut self, kind: Tok, start: usize) {
        self.tokens.push(Token {
            kind,
            start,
            end: self.offset,
        });
    }

    fn scan(&mut self, sink: &mut Sink) {
        let start = self.offset;
        let rest = &self.source[start..];
        let two: Option<(&str, Tok)> = [
            ("<->", Tok::Equivalent),
            ("->", Tok::Implies),
            ("<=", Tok::LessEqual),
            (">=", Tok::GreaterEqual),
            ("==", Tok::EqualEqual),
            ("!=", Tok::NotEqual),
        ]
        .into_iter()
        .find(|(spelling, _)| rest.starts_with(spelling));
        if let Some((spelling, kind)) = two {
            self.offset += spelling.len();
            self.push(kind, start);
            return;
        }
        let byte = self.bytes[start];
        let single = match byte {
            b'!' => Some(Tok::Not),
            b'&' => Some(Tok::And),
            b'|' => Some(Tok::Or),
            b'(' => Some(Tok::LeftParenthesis),
            b')' => Some(Tok::RightParenthesis),
            b'[' => Some(Tok::LeftBracket),
            b']' => Some(Tok::RightBracket),
            b',' => Some(Tok::Comma),
            b'+' => Some(Tok::Plus),
            b'*' => Some(Tok::Star),
            b'<' => Some(Tok::Less),
            b'>' => Some(Tok::Greater),
            _ => None,
        };
        if let Some(kind) = single {
            self.offset += 1;
            self.push(kind, start);
            return;
        }
        if byte == b'-' {
            let digit_follows = self
                .bytes
                .get(start + 1)
                .is_some_and(u8::is_ascii_digit);
            if digit_follows && !self.previous_ends_operand() {
                self.offset += 1;
                self.scan_number(start, true, sink);
            } else {
                self.offset += 1;
                self.push(Tok::Minus, start);
            }
            return;
        }
        if matches!(byte, b'/' | b'%' | b'^' | b'{' | b'}' | b';' | b':') {
            self.offset += 1;
            sink.push(
                Code::UnsupportedOperator,
                start,
                self.offset,
                format!("{:?} is outside tl-parse.clean-ascii/v5", byte as char),
            );
            self.push(Tok::Invalid, start);
            return;
        }
        if byte.is_ascii_digit() {
            self.scan_number(start, false, sink);
            return;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            self.scan_word(start, sink);
            return;
        }
        // `=` alone, `.`, NUL, other ASCII punctuation and every non-ASCII
        // character (U+2212 minus, fullwidth digits) are unexpected here.
        self.offset = next_char_end(self.source, start);
        sink.push(
            Code::UnexpectedCharacter,
            start,
            self.offset,
            format!("character {:?} is outside the ASCII dialect", &self.source[start..self.offset]),
        );
        self.push(Tok::Invalid, start);
    }

    fn scan_word(&mut self, start: usize, sink: &mut Sink) {
        while self.offset < self.bytes.len()
            && (self.bytes[self.offset].is_ascii_alphanumeric() || self.bytes[self.offset] == b'_')
        {
            self.offset += 1;
        }
        let lexeme = &self.source[start..self.offset];
        if let Some(keyword) = classify(lexeme) {
            self.push(Tok::Keyword(keyword), start);
            return;
        }
        if let Some(digits) = lexeme.strip_prefix('p') {
            if !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) {
                if digits.len() > 1 && digits.starts_with('0') {
                    sink.push(
                        Code::NonCanonicalNumber,
                        start,
                        self.offset,
                        format!("proposition {lexeme} has a leading zero"),
                    );
                    self.push(Tok::Invalid, start);
                } else if let Ok(value) = digits.parse::<u32>() {
                    self.max_explicit_proposition =
                        Some(self.max_explicit_proposition.map_or(value, |max| max.max(value)));
                    self.push(Tok::Proposition(value), start);
                } else {
                    sink.push(
                        Code::IntegerOverflow,
                        start,
                        self.offset,
                        format!("proposition {lexeme} exceeds u32"),
                    );
                    self.push(Tok::Invalid, start);
                }
                return;
            }
        }
        if lexeme.len() > MAX_NUMERIC_IDENTIFIER_BYTES {
            // SPEC-GAP: FR-029 bounds identifiers at 255 bytes but names no code
            // for a longer one. No catalog name can exceed 255 bytes, so the
            // simplest true statement is `unknown_signal`.
            sink.push(
                Code::UnknownSignal,
                start,
                self.offset,
                format!(
                    "identifier of {} bytes exceeds {MAX_NUMERIC_IDENTIFIER_BYTES}",
                    lexeme.len()
                ),
            );
            self.push(Tok::Invalid, start);
            return;
        }
        self.push(Tok::Identifier, start);
    }

    /// Scans `[-]digits[.digits]`; `self.offset` is already past a leading `-`.
    fn scan_number(&mut self, start: usize, negative: bool, sink: &mut Sink) {
        let integer_start = self.offset;
        while self.offset < self.bytes.len() && self.bytes[self.offset].is_ascii_digit() {
            self.offset += 1;
        }
        let integer_end = self.offset;
        let mut fraction: Option<(usize, usize)> = None;
        if self.bytes.get(self.offset) == Some(&b'.') {
            let fraction_start = self.offset + 1;
            let mut cursor = fraction_start;
            while cursor < self.bytes.len() && self.bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            self.offset = cursor;
            fraction = Some((fraction_start, cursor));
        }
        // Exponents, hexadecimal, digit separators and alphabetic suffixes
        // belong to the same malformed token and are refused as one.
        let suffix_start = self.offset;
        while self.offset < self.bytes.len()
            && (self.bytes[self.offset].is_ascii_alphanumeric()
                || self.bytes[self.offset] == b'_'
                || self.bytes[self.offset] == b'.')
        {
            self.offset += 1;
        }
        let lexeme = &self.source[start..self.offset];
        if self.offset > suffix_start {
            sink.push(
                Code::UnsupportedOperator,
                start,
                self.offset,
                format!("numeric spelling {lexeme:?} (exponent, radix or separator) is outside v5"),
            );
            self.push(Tok::Invalid, start);
            return;
        }
        let integer_digits = &self.source[integer_start..integer_end];
        let fraction_digits = fraction.map(|(from, to)| &self.source[from..to]);
        let all_zero = integer_digits.bytes().all(|byte| byte == b'0')
            && fraction_digits.is_none_or(|digits| digits.bytes().all(|byte| byte == b'0'));
        let non_canonical = (integer_digits.len() > 1 && integer_digits.starts_with('0'))
            || fraction_digits.is_some_and(str::is_empty)
            || (negative && all_zero);
        if non_canonical {
            sink.push(
                Code::NonCanonicalNumber,
                start,
                self.offset,
                format!("literal {lexeme:?} is not canonical"),
            );
            self.push(Tok::Invalid, start);
            return;
        }
        let fraction_digits = fraction_digits.unwrap_or("");
        let Ok(scale) = u8::try_from(fraction_digits.len()) else {
            sink.push(
                Code::ScaleOverflow,
                start,
                self.offset,
                format!("literal {lexeme:?} has more than {MAX_NUMERIC_SCALE} fractional digits"),
            );
            self.push(Tok::Invalid, start);
            return;
        };
        if scale > MAX_NUMERIC_SCALE {
            sink.push(
                Code::ScaleOverflow,
                start,
                self.offset,
                format!("literal {lexeme:?} has more than {MAX_NUMERIC_SCALE} fractional digits"),
            );
            self.push(Tok::Invalid, start);
            return;
        }
        match coefficient(integer_digits, fraction_digits, negative) {
            Some(value) => self.push(
                Tok::Number {
                    coefficient: value,
                    scale,
                },
                start,
            ),
            None => {
                sink.push(
                    Code::IntegerOverflow,
                    start,
                    self.offset,
                    format!("literal {lexeme:?} does not fit a signed 64-bit coefficient"),
                );
                self.push(Tok::Invalid, start);
            }
        }
    }
}

/// Exact coefficient of `integer.fraction`, checked against `i64`. Linear in
/// the digit count and never allocates, so a 4,096-digit literal is bounded.
fn coefficient(integer: &str, fraction: &str, negative: bool) -> Option<i64> {
    let mut value: i128 = 0;
    for byte in integer.bytes().chain(fraction.bytes()) {
        value = value.checked_mul(10)?.checked_add(i128::from(byte - b'0'))?;
        if value > i128::from(i64::MAX) + 1 {
            return None;
        }
    }
    let signed = if negative { -value } else { value };
    i64::try_from(signed).ok()
}

pub(crate) fn next_char_end(source: &str, start: usize) -> usize {
    source
        .get(start..)
        .and_then(|rest| rest.chars().next())
        .map_or(start, |character| start + character.len_utf8())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::numeric::Sink;

    fn kinds(source: &str) -> Vec<Tok> {
        let mut sink = Sink::new(64);
        let lexed = lex(source, ParseLimits::default(), &mut sink);
        lexed.tokens.iter().map(|token| token.kind).collect()
    }

    // Trace: TC-1544, FR-029-AC-2
    #[test]
    fn minus_is_lexed_by_position_not_adjacency() {
        assert_eq!(
            kinds("x -5"),
            vec![
                Tok::Identifier,
                Tok::Minus,
                Tok::Number {
                    coefficient: 5,
                    scale: 0
                },
                Tok::Eof
            ]
        );
        assert_eq!(
            kinds("x - -5"),
            vec![
                Tok::Identifier,
                Tok::Minus,
                Tok::Number {
                    coefficient: -5,
                    scale: 0
                },
                Tok::Eof
            ]
        );
        assert_eq!(kinds("-x"), vec![Tok::Minus, Tok::Identifier, Tok::Eof]);
        assert_eq!(
            kinds("(-5.00"),
            vec![
                Tok::LeftParenthesis,
                Tok::Number {
                    coefficient: -500,
                    scale: 2
                },
                Tok::Eof
            ]
        );
    }

    // Trace: TC-1544, FR-029-AC-2
    #[test]
    fn parenthesis_marks_follow_the_comparison_at_their_own_depth() {
        let mut sink = Sink::new(64);
        let lexed = lex("((x) > 1) & (p1 & p2)", ParseLimits::default(), &mut sink);
        assert!(lexed.opens_comparison[0]);
        assert!(!lexed.opens_comparison[1]);
        let inner = lexed
            .tokens
            .iter()
            .rposition(|token| token.kind == Tok::LeftParenthesis)
            .unwrap();
        assert!(!lexed.opens_comparison[inner]);
    }

    // Trace: TC-1546, FR-030-AC-1
    #[test]
    fn literal_refusals_have_their_own_codes() {
        for (source, code) in [
            ("-0", Code::NonCanonicalNumber),
            ("007", Code::NonCanonicalNumber),
            ("1.", Code::NonCanonicalNumber),
            (".5", Code::UnexpectedCharacter),
            ("1e5", Code::UnsupportedOperator),
            ("0x10", Code::UnsupportedOperator),
            ("9223372036854775808", Code::IntegerOverflow),
            ("0.1234567890123456789", Code::ScaleOverflow),
            ("\u{2212}5", Code::UnexpectedCharacter),
            ("=", Code::UnexpectedCharacter),
            ("/", Code::UnsupportedOperator),
            ("%", Code::UnsupportedOperator),
        ] {
            let mut sink = Sink::new(64);
            lex(source, ParseLimits::default(), &mut sink);
            assert_eq!(sink.refusal.map(|refusal| refusal.code), Some(code), "{source}");
        }
    }

    #[test]
    fn negative_extreme_literal_is_exact() {
        assert_eq!(coefficient("9223372036854775808", "", true), Some(i64::MIN));
        assert_eq!(coefficient("9223372036854775808", "", false), None);
    }
}
