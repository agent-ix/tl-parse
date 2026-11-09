//! TL-364: one failing input per reject class of the strict parser-artifact
//! readers, the diagnostic code tables, the formatter dialect gate and the v4
//! dialect policy.
//!
//! Every table row is one concrete input and the exact typed error it must
//! produce. Reports are produced by the real parser and mutated through their
//! public fields or textually, so no canonical bytes are copied here. The
//! numeric ceilings of NFR-005 live in `robustness.rs`; this suite covers the
//! structural and encoding classes, the report invariants, and the preflight
//! scanner edge cases.

use serde_json::error::Category;
use tl_parse::tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, SemanticProfile, SourceSpan};
use tl_parse::{
    format_clean_ascii_v3, format_document, parse_clean_ascii_v3, parse_clean_ascii_v4,
    DiagnosticCode, FormatError, FormatErrorCode, FormatLimits, InfiniteParseReport,
    ParseArtifactLimits, ParseLimits, PastParseReport, StrictParseArtifactReadError,
    HARD_MAX_PARSE_ARTIFACT_DEPTH, HARD_MAX_TOKENS,
};

type ReadOutcome = Result<(), StrictParseArtifactReadError>;

struct Reader {
    name: &'static str,
    valid: Vec<u8>,
    read: fn(&[u8], ParseArtifactLimits) -> ReadOutcome,
}

fn past_report() -> PastParseReport {
    let report = parse_clean_ascii_v3("O[0,1](p0S[0,2]Yp1)", ParseLimits::default());
    assert!(report.document.is_some());
    report
}

fn infinite_report() -> InfiniteParseReport {
    let report = parse_v4("fair {F[0,)p0}: G[1,)p1");
    assert!(report.document.is_some() && report.fairness.is_some());
    report
}

fn parse_v4(source: &str) -> InfiniteParseReport {
    parse_clean_ascii_v4(
        source,
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        ParseLimits::default(),
    )
}

fn readers() -> Vec<Reader> {
    vec![
        Reader {
            name: "past-parse-report/v1",
            valid: past_report().canonical_json_bytes().unwrap(),
            read: |bytes, limits| PastParseReport::from_json_bytes(bytes, limits).map(drop),
        },
        Reader {
            name: "infinite-parse-report/v1",
            valid: serde_json::to_vec(&infinite_report()).unwrap(),
            read: |bytes, limits| InfiniteParseReport::from_json_bytes(bytes, limits).map(drop),
        },
    ]
}

#[derive(Clone, Copy, Debug)]
enum Expect {
    /// `InvalidDocument` whose serde category and message prefix are exact.
    Invalid(Category, &'static str),
    NonCanonical,
}

fn check(reader: &str, row: &str, outcome: ReadOutcome, expect: Expect) {
    let context = format!("{reader} / {row}");
    match (outcome, expect) {
        (Err(StrictParseArtifactReadError::NonCanonicalDocument), Expect::NonCanonical) => {}
        (
            Err(StrictParseArtifactReadError::InvalidDocument(error)),
            Expect::Invalid(class, prefix),
        ) => {
            assert_eq!(error.classify(), class, "{context}: {error}");
            assert!(
                error.to_string().starts_with(prefix),
                "{context}: expected message starting {prefix:?}, got {error}"
            );
        }
        (other, expected) => panic!("{context}: expected {expected:?}, got {other:?}"),
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

/// Replaces the first occurrence of `from`, failing loudly if it is absent.
fn mutate_first(valid: &[u8], from: &str, to: &str) -> Vec<u8> {
    let source = text(valid);
    assert!(source.contains(from), "{from:?} in {source}");
    source.replacen(from, to, 1).into_bytes()
}

/// The `"schema_version":"<id>"` member that every report opens with.
fn version_member(valid: &[u8]) -> String {
    let source = text(valid);
    let prefix = "{\"schema_version\":\"";
    assert!(source.starts_with(prefix), "{source}");
    let end = source[prefix.len()..].find('"').unwrap() + prefix.len();
    source[1..=end].to_string()
}

// Trace: TC-046, FR-009-AC-3, FR-009-AC-4
#[test]
fn every_report_reader_refuses_each_structural_and_encoding_class() {
    let limits = ParseArtifactLimits::default();
    for reader in readers() {
        let valid = reader.valid.as_slice();
        assert!(
            (reader.read)(valid, limits).is_ok(),
            "{}: canonical baseline must be accepted",
            reader.name
        );
        let member = version_member(valid);
        let with_comma = format!("{member},");
        let unknown_version = format!("{}9\"", &member[..member.len() - 1]);
        let moved_last = {
            let rest = text(valid).replacen(&with_comma, "", 1);
            format!("{},{member}}}", &rest[..rest.len() - 1])
        };
        let mut invalid_utf8 = mutate_first(valid, &member, "\"schema_version\":\"\u{1}");
        for byte in &mut invalid_utf8 {
            if *byte == 1 {
                *byte = 0xff;
            }
        }
        let rows: Vec<(&str, Vec<u8>, Expect)> = vec![
            (
                "empty input",
                Vec::new(),
                Expect::Invalid(Category::Eof, "EOF while parsing a value"),
            ),
            (
                "truncated document",
                valid[..valid.len() - 1].to_vec(),
                Expect::Invalid(Category::Eof, "EOF while parsing"),
            ),
            (
                "invalid UTF-8 inside a string",
                invalid_utf8,
                Expect::Invalid(Category::Syntax, "invalid unicode code point"),
            ),
            (
                "trailing data: stray continuation byte after the document",
                [valid, b"\x80"].concat(),
                Expect::Invalid(Category::Syntax, "trailing characters"),
            ),
            (
                "trailing garbage",
                [valid, b"x"].concat(),
                Expect::Invalid(Category::Syntax, "trailing characters"),
            ),
            (
                "second document appended",
                [valid, valid].concat(),
                Expect::Invalid(Category::Syntax, "trailing characters"),
            ),
            (
                "trailing whitespace is valid JSON but not canonical",
                [valid, b"\n"].concat(),
                Expect::NonCanonical,
            ),
            (
                "inserted whitespace is not canonical",
                mutate_first(valid, "{\"schema_version\"", "{ \"schema_version\""),
                Expect::NonCanonical,
            ),
            (
                "escaped member name decodes but is not canonical",
                mutate_first(valid, "\"schema_version\"", "\"\\u0073chema_version\""),
                Expect::NonCanonical,
            ),
            (
                "members out of canonical order",
                moved_last.into_bytes(),
                Expect::NonCanonical,
            ),
            (
                "duplicate member",
                mutate_first(valid, &with_comma, &format!("{with_comma}{with_comma}")),
                Expect::Invalid(Category::Data, "duplicate field `schema_version`"),
            ),
            (
                "unknown member",
                mutate_first(
                    valid,
                    "{\"schema_version\"",
                    "{\"unknown\":0,\"schema_version\"",
                ),
                Expect::Invalid(Category::Data, "unknown field `unknown`"),
            ),
            (
                "missing member",
                mutate_first(valid, &with_comma, ""),
                Expect::Invalid(Category::Data, "missing field `schema_version`"),
            ),
            (
                "unknown schema version",
                mutate_first(
                    valid,
                    &member,
                    &format!("\"schema_version\":{unknown_version}"),
                ),
                Expect::Invalid(Category::Data, "unknown variant"),
            ),
            (
                "schema version of the wrong JSON type",
                mutate_first(valid, &member, "\"schema_version\":7"),
                Expect::Invalid(Category::Syntax, "expected value"),
            ),
            (
                "document root of the wrong JSON type",
                b"7".to_vec(),
                Expect::Invalid(Category::Data, "invalid type: integer `7`"),
            ),
            (
                "null document",
                b"null".to_vec(),
                Expect::Invalid(Category::Data, "invalid type: null"),
            ),
        ];
        for (row, bytes, expect) in rows {
            check(reader.name, row, (reader.read)(&bytes, limits), expect);
        }
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn report_reader_preflight_scanner_edge_cases_are_exact() {
    // Each row is a JSON string literal, the lowered `string_bytes` limit, and
    // the decoded size the preflight must report for the first rejected byte.
    let escape_rows: [(&str, usize, usize); 8] = [
        (r#""\u0041""#, 0, 1),
        (r#""\u00e9""#, 1, 2),
        (r#""\u20ac""#, 2, 3),
        (r#""\ud83d\ude00""#, 3, 4),
        (r#""\n""#, 0, 1),
        // A high surrogate without its pair is charged as a three-byte scalar.
        (r#""\ud83dxx""#, 2, 3),
        // Malformed hex digits charge nothing for the escape, then one byte each.
        (r#""\uZZZZ""#, 3, 4),
        (r#""abc""#, 2, 3),
    ];
    for reader in readers() {
        for (literal, string_bytes, actual) in escape_rows {
            let limits = ParseArtifactLimits {
                string_bytes,
                ..ParseArtifactLimits::default()
            };
            let bytes = format!("{{\"schema_version\":{literal}}}");
            match (reader.read)(bytes.as_bytes(), limits) {
                Err(StrictParseArtifactReadError::StringTooLarge { actual: a, limit }) => {
                    assert_eq!(
                        (a, limit),
                        (actual, string_bytes),
                        "{} / {literal}",
                        reader.name
                    );
                }
                other => panic!("{} / {literal}: got {other:?}", reader.name),
            }
        }

        // Brackets inside a string are not nesting, however many there are.
        let brackets = format!("{{\"schema_version\":\"{}\"}}", "[".repeat(200));
        check(
            reader.name,
            "brackets inside a string",
            (reader.read)(brackets.as_bytes(), ParseArtifactLimits::default()),
            Expect::Invalid(Category::Data, "unknown variant"),
        );

        // An escaped quote does not end the string, so the brackets after it
        // stay inside the string and the document stays within depth 1.
        let escaped_quote = "{\"schema_version\":\"\\\"[[[[\"}";
        let depth_one = ParseArtifactLimits {
            json_depth: 1,
            ..ParseArtifactLimits::default()
        };
        check(
            reader.name,
            "escaped quote keeps brackets inside the string",
            (reader.read)(escaped_quote.as_bytes(), depth_one),
            Expect::Invalid(Category::Data, "unknown variant"),
        );

        // Closers without openers saturate at depth zero instead of underflowing.
        let closers = format!("{}{{}}", "]}".repeat(8));
        check(
            reader.name,
            "unbalanced closers",
            (reader.read)(closers.as_bytes(), depth_one),
            Expect::Invalid(Category::Syntax, "expected value"),
        );

        // Nesting past the owner ceiling is refused before any parsing.
        let nesting = HARD_MAX_PARSE_ARTIFACT_DEPTH + 1;
        let deep = format!("{}{}", "[".repeat(nesting), "]".repeat(nesting));
        match (reader.read)(deep.as_bytes(), ParseArtifactLimits::default()) {
            Err(StrictParseArtifactReadError::DepthLimitExceeded { actual, limit }) => {
                assert_eq!(
                    (actual, limit),
                    (nesting, HARD_MAX_PARSE_ARTIFACT_DEPTH),
                    "{}",
                    reader.name
                );
            }
            other => panic!("{} / over-depth: {other:?}", reader.name),
        }

        // A string left open at the end of input is refused by the JSON parser.
        check(
            reader.name,
            "unterminated string",
            (reader.read)(b"{\"schema_version\":\"abc", ParseArtifactLimits::default()),
            Expect::Invalid(Category::Eof, "EOF while parsing a string"),
        );
        check(
            reader.name,
            "dangling backslash at end of input",
            (reader.read)(
                b"{\"schema_version\":\"abc\\",
                ParseArtifactLimits::default(),
            ),
            Expect::Invalid(Category::Eof, "EOF while parsing a string"),
        );

        // The fixed admission charge is four work units per input byte (no exported constant).
        let length = reader.valid.len();
        let tight = ParseArtifactLimits {
            document_bytes: length - 1,
            ..ParseArtifactLimits::default()
        };
        match (reader.read)(&reader.valid, tight) {
            Err(StrictParseArtifactReadError::DocumentTooLarge { actual, limit }) => {
                assert_eq!((actual, limit), (length, length - 1), "{}", reader.name);
            }
            other => panic!("{} / over-bytes: {other:?}", reader.name),
        }
        let starved = ParseArtifactLimits {
            work: length * 4 - 1,
            ..ParseArtifactLimits::default()
        };
        match (reader.read)(&reader.valid, starved) {
            Err(StrictParseArtifactReadError::WorkLimitExceeded { actual, limit }) => {
                assert_eq!(
                    (actual, limit),
                    (length * 4, length * 4 - 1),
                    "{}",
                    reader.name
                );
            }
            other => panic!("{} / over-work: {other:?}", reader.name),
        }
    }
}

fn reread_past(report: &PastParseReport) -> ReadOutcome {
    let bytes = serde_json::to_vec(report).unwrap();
    PastParseReport::from_json_bytes(&bytes, ParseArtifactLimits::default()).map(drop)
}

fn reread_infinite(report: &InfiniteParseReport) -> ReadOutcome {
    let bytes = serde_json::to_vec(report).unwrap();
    InfiniteParseReport::from_json_bytes(&bytes, ParseArtifactLimits::default()).map(drop)
}

fn invariant(message: &'static str) -> Expect {
    Expect::Invalid(Category::Data, message)
}

// Trace: TC-046, FR-009-AC-3, FR-009-AC-4
#[test]
fn past_report_reader_refuses_each_invariant_violation_with_its_message() {
    let good = past_report();
    assert!(reread_past(&good).is_ok());
    let failed = parse_clean_ascii_v3("O[0,", ParseLimits::default());
    assert!(failed.document.is_none() && !failed.diagnostics.is_empty());
    assert!(reread_past(&failed).is_ok());

    let mut rows: Vec<(&str, PastParseReport, Expect)> = Vec::new();
    let mut row = |name: &'static str,
                   edit: &dyn Fn(&mut PastParseReport),
                   base: &PastParseReport,
                   expect| {
        let mut report = base.clone();
        edit(&mut report);
        rows.push((name, report, expect));
    };
    // Trips `limits != limits.clamped()`, which has one generic message for every field.
    row(
        "limits above the process-safe maximum",
        &|r| r.limits.max_tokens = HARD_MAX_TOKENS + 1,
        &good,
        invariant("parse limits exceed the process-safe maximum"),
    );
    row(
        "wrong semantic profile",
        &|r| r.semantic_profile = SemanticProfile::ClosedTraceV1,
        &good,
        invariant("clean-ascii/v3 requires the origin-complete history profile"),
    );
    row(
        "diagnostic count that matches no retained diagnostic",
        &|r| r.stats.diagnostics = 3,
        &good,
        invariant("diagnostic count does not match retained diagnostics"),
    );
    row(
        "token count above the effective limit",
        &|r| r.limits.max_tokens = 1,
        &good,
        invariant("parse statistics exceed the effective limits"),
    );
    row(
        "failed parse without a diagnostic",
        &|r| {
            r.document = None;
        },
        &good,
        invariant("a failed parse requires a retained or truncated diagnostic"),
    );
    row(
        "diagnostic span beyond the submitted source",
        &|r| r.diagnostics[0].span = SourceSpan::new(0, 10_000).unwrap(),
        &failed,
        invariant("diagnostic span exceeds the submitted source"),
    );
    row(
        "document next to a diagnostic",
        &|r| r.document = good.document.clone(),
        &failed,
        invariant("a document requires a diagnostic-free parse"),
    );
    row(
        "source bytes above the effective source limit",
        &|r| r.limits.max_source_bytes = 1,
        &good,
        invariant("a successful parse exceeds its effective source/depth limits"),
    );
    row(
        "formula-v1 document under the past dialect",
        &|r| {
            r.document = Some(
                FormulaDocument::new(
                    SemanticProfile::ClosedTraceV1,
                    NodeId(0),
                    vec![Node::new(NodeKind::True)],
                )
                .unwrap(),
            );
            r.stats.nodes = 1;
        },
        &good,
        invariant("clean-ascii/v3 documents require formula-v2 and the past profile"),
    );
    row(
        "node count that disagrees with the document",
        &|r| r.stats.nodes += 1,
        &good,
        invariant("document graph does not match the parse report"),
    );
    row(
        "document nodes without source spans",
        &|r| {
            let document = r.document.as_ref().unwrap();
            let bare = document
                .nodes()
                .iter()
                .map(|node| Node::new(node.kind))
                .collect();
            r.document = Some(
                FormulaDocument::new_v2(document.semantic_profile(), document.root(), bare)
                    .unwrap(),
            );
        },
        &good,
        invariant("document nodes require source-bounded spans"),
    );
    for (name, report, expect) in rows {
        check("past-parse-report/v1", name, reread_past(&report), expect);
    }
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_report_reader_refuses_each_invariant_violation_with_its_message() {
    let good = infinite_report();
    assert!(reread_infinite(&good).is_ok());
    let plain = parse_v4("p0&p1");
    assert!(plain.document.is_some() && plain.interval_spans.is_empty());
    let failed = parse_v4("p0 $ p1");
    assert!(failed.document.is_none() && failed.refusal.is_some());
    assert!(reread_infinite(&failed).is_ok());
    let other_fairness = parse_v4("fair {G[0,)p1}: G[1,)p1").fairness.unwrap();

    let mut rows: Vec<(&str, InfiniteParseReport, Expect)> = Vec::new();
    let mut row = |name: &'static str,
                   edit: &dyn Fn(&mut InfiniteParseReport),
                   base: &InfiniteParseReport,
                   expect| {
        let mut report = base.clone();
        edit(&mut report);
        rows.push((name, report, expect));
    };
    // Trips `limits != limits.clamped()`, which has one generic message for every field.
    row(
        "limits above the process-safe maximum",
        &|r| r.limits.max_tokens = HARD_MAX_TOKENS + 1,
        &good,
        invariant("v4 report counters exceed effective limits"),
    );
    row(
        "interval span beyond the source",
        &|r| r.interval_spans.push(SourceSpan::new(0, 10_000).unwrap()),
        &good,
        invariant("v4 report span exceeds source bytes"),
    );
    row(
        "graph and refusal together",
        &|r| r.refusal = failed.refusal,
        &good,
        invariant("v4 refusal code must occur exactly when the graph is absent"),
    );
    row(
        "neither graph nor refusal",
        &|r| r.refusal = None,
        &failed,
        invariant("v4 refusal code must occur exactly when the graph is absent"),
    );
    row(
        "refusal locus that differs from the retained diagnostic",
        &|r| {
            let mut refusal = r.refusal.unwrap();
            refusal.span = SourceSpan::new(0, 1).unwrap();
            r.refusal = Some(refusal);
        },
        &failed,
        invariant("v4 refusal conflicts with retained diagnostics"),
    );
    row(
        "failed report that retains no diagnostic",
        &|r| {
            r.diagnostics.clear();
            r.stats.diagnostics = 0;
        },
        &failed,
        invariant("failed v4 report requires a diagnostic or truncated diagnostics"),
    );
    row(
        "refusal locus beyond the source",
        &|r| {
            let mut refusal = r.refusal.unwrap();
            refusal.span = SourceSpan::new(0, 10_000).unwrap();
            r.refusal = Some(refusal);
            r.stats.diagnostics_truncated = true;
        },
        &failed,
        invariant("v4 refusal locus exceeds source bytes"),
    );
    row(
        "foreign clock on a successful report",
        &|r| r.clock = "wall_clock".to_owned(),
        &good,
        invariant("v4 successful report has conflicting identities or diagnostics"),
    );
    row(
        "node span beyond the source",
        &|r| r.stats.source_bytes = 1,
        &plain,
        invariant("v4 graph node span exceeds source bytes"),
    );
    row(
        "fairness bound to another graph",
        &|r| r.fairness = Some(other_fairness.clone()),
        &good,
        invariant("v4 fairness binding differs from owner graph"),
    );
    row(
        "premise spans without fairness roots",
        &|r| r.fairness = None,
        &good,
        invariant("v4 premise spans require fairness roots"),
    );
    row(
        "fairness on a failed report",
        &|r| r.fairness = good.fairness.clone(),
        &failed,
        invariant("v4 fairness requires a successful graph"),
    );
    for (name, report, expect) in rows {
        check(
            "infinite-parse-report/v1",
            name,
            reread_infinite(&report),
            expect,
        );
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn read_errors_render_their_exact_diagnostics() {
    let render = |error: StrictParseArtifactReadError| error.to_string();
    assert_eq!(
        render(StrictParseArtifactReadError::DocumentTooLarge {
            actual: 9,
            limit: 8
        }),
        "document has 9 bytes; limit is 8"
    );
    assert_eq!(
        render(StrictParseArtifactReadError::DepthLimitExceeded {
            actual: 65,
            limit: 64
        }),
        "document nesting depth is 65; limit is 64"
    );
    assert_eq!(
        render(StrictParseArtifactReadError::StringTooLarge {
            actual: 3,
            limit: 2
        }),
        "JSON string has 3 bytes; limit is 2"
    );
    assert_eq!(
        render(StrictParseArtifactReadError::WorkLimitExceeded {
            actual: 7,
            limit: 6
        }),
        "document admission costs 7 work units; limit is 6"
    );
    assert_eq!(
        render(StrictParseArtifactReadError::NonCanonicalDocument),
        "document is not canonical tl-parse JSON"
    );
    let invalid =
        PastParseReport::from_json_bytes(b"7", ParseArtifactLimits::default()).unwrap_err();
    assert!(render(invalid).starts_with("invalid document: invalid type: integer `7`"));
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn diagnostic_and_format_codes_keep_their_exact_wire_spellings() {
    let diagnostics = [
        (DiagnosticCode::SourceLimit, "source_limit"),
        (DiagnosticCode::TokenLimit, "token_limit"),
        (DiagnosticCode::NodeLimit, "node_limit"),
        (DiagnosticCode::DepthLimit, "depth_limit"),
        (DiagnosticCode::WorkLimit, "work_limit"),
        (DiagnosticCode::UnexpectedCharacter, "unexpected_character"),
        (DiagnosticCode::UnknownIdentifier, "unknown_identifier"),
        (DiagnosticCode::NonCanonicalNumber, "non_canonical_number"),
        (DiagnosticCode::IntegerOverflow, "integer_overflow"),
        (DiagnosticCode::UnexpectedToken, "unexpected_token"),
        (DiagnosticCode::MissingToken, "missing_token"),
        (DiagnosticCode::InvalidInterval, "invalid_interval"),
        (DiagnosticCode::TrailingInput, "trailing_input"),
        (DiagnosticCode::ValidationFailure, "validation_failure"),
        (DiagnosticCode::UnsupportedOperator, "unsupported_operator"),
        (
            DiagnosticCode::InfiniteProfileMismatch,
            "infinite_profile_mismatch",
        ),
        (
            DiagnosticCode::InfiniteClockMismatch,
            "infinite_clock_mismatch",
        ),
        (
            DiagnosticCode::DuplicateFairnessPremise,
            "duplicate_fairness_premise",
        ),
    ];
    for (code, spelling) in diagnostics {
        assert_eq!(code.as_str(), spelling);
        assert_eq!(code.to_string(), spelling);
        assert_eq!(
            serde_json::to_string(&code).unwrap(),
            format!("\"{spelling}\"")
        );
        assert_eq!(
            serde_json::from_str::<DiagnosticCode>(&format!("\"{spelling}\"")).unwrap(),
            code
        );
    }
    assert!(serde_json::from_str::<DiagnosticCode>("\"invented_code\"").is_err());

    let formats = [
        (FormatErrorCode::InvalidGraph, "invalid_graph"),
        (
            FormatErrorCode::UnrepresentableGraph,
            "unrepresentable_graph",
        ),
        (FormatErrorCode::OutputLimit, "output_limit"),
        (FormatErrorCode::WorkLimit, "work_limit"),
    ];
    for (code, spelling) in formats {
        assert_eq!(code.as_str(), spelling);
        assert_eq!(code.to_string(), spelling);
        let error = FormatError {
            code,
            message: "m".to_owned(),
        };
        assert_eq!(error.to_string(), format!("{spelling}: m"));
        assert_eq!(
            serde_json::from_str::<FormatError>(&serde_json::to_string(&error).unwrap()).unwrap(),
            error
        );
    }
    assert!(serde_json::from_str::<FormatErrorCode>("\"invented_code\"").is_err());
    assert!(serde_json::from_str::<FormatError>(
        "{\"code\":\"work_limit\",\"message\":\"m\",\"extra\":0}"
    )
    .is_err());
}

// Trace: TC-046, FR-009-AC-3, FR-009-AC-4
#[test]
fn formatter_refuses_documents_outside_the_selected_dialect_with_the_exact_error() {
    let limits = FormatLimits::default();
    let expected = FormatError {
        code: FormatErrorCode::InvalidGraph,
        message: "clean-ascii/v3 requires formula-v2 with mltl.origin-complete-history/v1"
            .to_owned(),
    };
    let node = || vec![Node::new(NodeKind::True)];
    let documents = [
        (
            "formula-v1 closed",
            FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(0), node()).unwrap(),
        ),
        (
            "formula-v2 closed",
            FormulaDocument::new_v2(SemanticProfile::ClosedTraceV1, NodeId(0), node()).unwrap(),
        ),
        (
            "formula-v2 online prefix",
            FormulaDocument::new_v2(SemanticProfile::OnlinePrefixV1, NodeId(0), node()).unwrap(),
        ),
    ];
    for (row, document) in &documents {
        let report = format_clean_ascii_v3(document, limits);
        assert_eq!(report.text, None, "{row}");
        assert_eq!(report.error, Some(expected.clone()), "{row}");
        assert_eq!(report.stats, Default::default(), "{row}");
    }
    // The generic entry point formats the same closed documents in the v1 dialect.
    let closed =
        FormulaDocument::new_v2(SemanticProfile::ClosedTraceV1, NodeId(0), node()).unwrap();
    assert_eq!(
        format_document(&closed, limits).text.as_deref(),
        Some("true")
    );

    // Limits above the hard maxima are clamped in the failure report too.
    let huge = FormatLimits {
        max_output_bytes: usize::MAX,
        max_work: usize::MAX,
    };
    let report = format_clean_ascii_v3(&documents[0].1, huge);
    assert_eq!(report.limits, FormatLimits::default());
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_dialect_policy_refuses_each_reject_class_with_its_exact_code() {
    // Atoms absorb a following operator letter unless a bracket follows it, so
    // the unbracketed spellings read as unknown identifiers.
    let rows: [(&str, DiagnosticCode); 15] = [
        ("p0U p1", DiagnosticCode::UnknownIdentifier),
        ("pU[0,1]", DiagnosticCode::UnknownIdentifier),
        ("pWp1", DiagnosticCode::UnknownIdentifier),
        ("X p0", DiagnosticCode::UnknownIdentifier),
        ("p0 W p1", DiagnosticCode::MissingToken),
        ("G p0", DiagnosticCode::MissingToken),
        ("fair {p0}", DiagnosticCode::MissingToken),
        ("F[0,)", DiagnosticCode::UnexpectedToken),
        ("W[0,)p0", DiagnosticCode::UnexpectedToken),
        ("!", DiagnosticCode::UnexpectedToken),
        ("p0 W[0,) p1 ) ", DiagnosticCode::TrailingInput),
        ("p0 W[2,1) p1", DiagnosticCode::InvalidInterval),
        ("p00", DiagnosticCode::NonCanonicalNumber),
        ("p99999999999", DiagnosticCode::IntegerOverflow),
        ("p0 $ p1", DiagnosticCode::UnexpectedCharacter),
    ];
    for (source, code) in rows {
        let report = parse_v4(source);
        assert!(report.document.is_none(), "{source}");
        assert_eq!(
            report.refusal.map(|refusal| refusal.code),
            Some(code),
            "{source}"
        );
        let codes: Vec<_> = report.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, vec![code], "{source}");
    }

    // Bracketed operator letters split the atom and are accepted.
    for source in [
        "p0U[0,1]p1",
        "p0W[0,)p1",
        "p0M[0,)G[0,)p1",
        "H[0,)p0",
        "Yp0",
        "O[0,)p0",
    ] {
        assert!(parse_v4(source).document.is_some(), "{source}");
    }

    // The dialect requires the exact profile and the event-position clock.
    let profile = parse_clean_ascii_v4(
        "p0",
        SemanticProfile::ClosedTraceV1,
        "event_position",
        ParseLimits::default(),
    );
    assert_eq!(
        profile.refusal.map(|refusal| refusal.code),
        Some(DiagnosticCode::InfiniteProfileMismatch)
    );
    let clock = parse_clean_ascii_v4(
        "p0",
        SemanticProfile::InfiniteTraceV1,
        "wall_clock",
        ParseLimits::default(),
    );
    assert_eq!(
        clock.refusal.map(|refusal| refusal.code),
        Some(DiagnosticCode::InfiniteClockMismatch)
    );
}
