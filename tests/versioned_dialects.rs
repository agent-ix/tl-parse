use tl_parse::{
    format_clean_ascii_v3, format_document, parse, parse_clean_ascii_v2, parse_clean_ascii_v3,
    DiagnosticCode, FormatErrorCode, FormatLimits, ParseArtifactLimits, ParseLimits,
    StrictParseArtifactReadError,
};
use tl_syntax::{FormulaDocument, NodeKind, SemanticProfile, SyntaxArtifactLimits};

fn v1(source: &str) -> tl_parse::ParseReport {
    parse(
        source,
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
    )
}

// Trace: TC-046, FR-009-AC-1
#[test]
fn every_entry_point_selects_one_closed_dialect_without_fallback() {
    assert!(v1("F[0,1]p0").document.is_some());
    assert!(parse_clean_ascii_v2(
        "p0W[0,1]p1",
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
    )
    .document
    .is_some());
    assert!(parse_clean_ascii_v3("O[0,1]p0", ParseLimits::default())
        .document
        .is_some());

    for source in ["W[0,1]p0", "O[0,1]p0", "Yp0", "p0S[0,1]p1"] {
        assert!(v1(source).document.is_none(), "v1 accepted {source}");
    }
    for source in ["O[0,1]p0", "Yp0", "p0S[0,1]p1", "p0T[0,1]p1"] {
        let report = parse_clean_ascii_v2(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert!(report.document.is_none(), "v2 accepted {source}");
    }
    for source in [
        "F[0,1]p0",
        "G[0,1]p0",
        "p0U[0,1]p1",
        "p0R[0,1]p1",
        "p0W[0,1]p1",
        "p0M[0,1]p1",
    ] {
        let report = parse_clean_ascii_v3(source, ParseLimits::default());
        assert!(report.document.is_none(), "v3 accepted {source}");
        assert_eq!(
            report.diagnostics[0].code,
            DiagnosticCode::UnsupportedOperator
        );
    }
}

// Trace: TC-046, FR-009-AC-2, FR-009-AC-3
#[test]
fn past_policy_owns_exact_graph_precedence_spans_and_canonical_text() {
    let cases = [
        ("O[0,2]p0", "O[0,2]p0"),
        ("H[1,3]p1", "H[1,3]p1"),
        ("Yp2", "Yp2"),
        ("p0S[0,1]p1", "p0S[0,1]p1"),
        ("p0T[2,4]p1", "p0T[2,4]p1"),
        ("p0|p1S[0,2]p2&p3", "p0|p1S[0,2]p2&p3"),
    ];
    for (source, expected) in cases {
        let first = parse_clean_ascii_v3(source, ParseLimits::default());
        assert!(
            first.diagnostics.is_empty(),
            "{source}: {:?}",
            first.diagnostics
        );
        let document = first.document.expect("accepted v3 document");
        assert!(document.nodes().iter().all(|node| node.span.is_some()));
        let formatted = format_clean_ascii_v3(&document, FormatLimits::default());
        assert_eq!(formatted.text.as_deref(), Some(expected), "{source}");
        let reparsed = parse_clean_ascii_v3(expected, ParseLimits::default());
        assert_eq!(reparsed.document, Some(document));
    }

    let left = parse_clean_ascii_v3("p0S[0,1]p1T[0,1]p2", ParseLimits::default())
        .document
        .expect("left-associative temporal chain");
    let root = usize::try_from(left.root().0).unwrap();
    assert!(matches!(
        left.nodes()[root].kind,
        NodeKind::Triggered { .. }
    ));
}

// Trace: TC-046, FR-009-AC-3, FR-009-AC-4
#[test]
fn every_success_crosses_the_real_pinned_syntax_owner_reader() {
    let documents = [
        v1("G[0,2](p0->p1)").document.unwrap(),
        parse_clean_ascii_v2(
            "p0W[0,2]p1",
            SemanticProfile::OnlinePrefixV1,
            ParseLimits::default(),
        )
        .document
        .unwrap(),
        parse_clean_ascii_v3("H[0,2](p0S[1,2]Yp1)", ParseLimits::default())
            .document
            .unwrap(),
    ];

    for document in documents {
        let bytes = document.canonical_json_bytes().unwrap();
        let admitted =
            FormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default()).unwrap();
        assert_eq!(admitted, document);

        let encoded = String::from_utf8(bytes.clone()).unwrap();
        let mutation = if document.semantic_profile() == SemanticProfile::OriginCompleteHistoryV1 {
            encoded.replace("mltl.origin-complete-history/v1", "mltl.closed-trace/v1")
        } else {
            encoded.replace(
                document.semantic_profile().as_str(),
                "mltl.origin-complete-history/v1",
            )
        };
        assert!(FormulaDocument::from_json_bytes(
            mutation.as_bytes(),
            SyntaxArtifactLimits::default()
        )
        .is_err());

        let root_field = format!("\"root\":{}", document.root().0);
        let topology_mutation = encoded.replacen(&root_field, "\"root\":4294967295", 1);
        assert!(FormulaDocument::from_json_bytes(
            topology_mutation.as_bytes(),
            SyntaxArtifactLimits::default(),
        )
        .is_err());

        let operator_mutation =
            encoded.replacen("\"kind\":\"proposition\"", "\"kind\":\"once\"", 1);
        assert!(FormulaDocument::from_json_bytes(
            operator_mutation.as_bytes(),
            SyntaxArtifactLimits::default(),
        )
        .is_err());
    }

    let upgraded_future = v1("F[0,1]p0").document.unwrap().to_v2();
    assert_eq!(
        format_document(&upgraded_future, FormatLimits::default())
            .text
            .as_deref(),
        Some("F[0,1]p0")
    );
    assert!(
        format_clean_ascii_v3(&upgraded_future, FormatLimits::default())
            .text
            .is_none()
    );
}

// Trace: TC-046, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5
#[test]
fn past_report_reader_is_bounded_canonical_and_fail_closed() {
    let report = parse_clean_ascii_v3("O[0,1](p0S[0,2]Yp1)", ParseLimits::default());
    let bytes = report.canonical_json_bytes().unwrap();
    let exact = ParseArtifactLimits {
        document_bytes: bytes.len(),
        work: bytes.len() * 4,
        ..ParseArtifactLimits::default()
    };
    assert_eq!(
        tl_parse::PastParseReport::from_json_bytes(&bytes, exact).unwrap(),
        report
    );

    let byte_short = ParseArtifactLimits {
        document_bytes: bytes.len() - 1,
        ..ParseArtifactLimits::default()
    };
    assert!(matches!(
        tl_parse::PastParseReport::from_json_bytes(&bytes, byte_short),
        Err(StrictParseArtifactReadError::DocumentTooLarge { .. })
    ));
    let work_short = ParseArtifactLimits {
        work: bytes.len() * 4 - 1,
        ..ParseArtifactLimits::default()
    };
    assert!(matches!(
        tl_parse::PastParseReport::from_json_bytes(&bytes, work_short),
        Err(StrictParseArtifactReadError::WorkLimitExceeded { .. })
    ));
    assert!(matches!(
        tl_parse::PastParseReport::from_json_bytes(
            &bytes,
            ParseArtifactLimits {
                json_depth: 1,
                ..ParseArtifactLimits::default()
            },
        ),
        Err(StrictParseArtifactReadError::DepthLimitExceeded { .. })
    ));
    assert!(matches!(
        tl_parse::PastParseReport::from_json_bytes(
            &bytes,
            ParseArtifactLimits {
                string_bytes: 1,
                ..ParseArtifactLimits::default()
            },
        ),
        Err(StrictParseArtifactReadError::StringTooLarge { .. })
    ));

    let mut whitespace = bytes.clone();
    whitespace.push(b'\n');
    assert!(matches!(
        tl_parse::PastParseReport::from_json_bytes(&whitespace, ParseArtifactLimits::default()),
        Err(StrictParseArtifactReadError::NonCanonicalDocument)
    ));

    let reordered =
        serde_json::to_vec(&serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()).unwrap();
    assert_ne!(reordered, bytes);
    assert!(matches!(
        tl_parse::PastParseReport::from_json_bytes(&reordered, ParseArtifactLimits::default()),
        Err(StrictParseArtifactReadError::NonCanonicalDocument)
    ));

    let json = String::from_utf8(bytes.clone()).unwrap();
    let unknown = format!("{{\"unknown\":true,{}", &json[1..]);
    let duplicate = format!(
        "{{\"schema_version\":\"tl-parse.past-parse-report/v1\",{}",
        &json[1..]
    );
    let trailing = format!("{json}x");
    for refused in [unknown, duplicate, trailing] {
        assert!(matches!(
            tl_parse::PastParseReport::from_json_bytes(
                refused.as_bytes(),
                ParseArtifactLimits::default(),
            ),
            Err(StrictParseArtifactReadError::InvalidDocument(_))
        ));
    }
}

// Trace: TC-046, FR-009-AC-5
#[test]
fn parse_and_format_limits_are_exact_and_hostile_utf8_never_unwinds() {
    let baseline = v1("p0");
    let work = baseline.stats.work;
    for limits in [
        ParseLimits {
            max_source_bytes: 2,
            max_tokens: 1,
            max_nodes: 1,
            max_depth: 1,
            max_work: work,
            ..ParseLimits::default()
        },
        ParseLimits::default(),
    ] {
        assert!(parse("p0", SemanticProfile::ClosedTraceV1, limits)
            .document
            .is_some());
    }
    for (limits, code) in [
        (
            ParseLimits {
                max_source_bytes: 1,
                ..ParseLimits::default()
            },
            DiagnosticCode::SourceLimit,
        ),
        (
            ParseLimits {
                max_tokens: 0,
                ..ParseLimits::default()
            },
            DiagnosticCode::TokenLimit,
        ),
        (
            ParseLimits {
                max_nodes: 0,
                ..ParseLimits::default()
            },
            DiagnosticCode::NodeLimit,
        ),
        (
            ParseLimits {
                max_depth: 0,
                ..ParseLimits::default()
            },
            DiagnosticCode::DepthLimit,
        ),
        (
            ParseLimits {
                max_work: work - 1,
                ..ParseLimits::default()
            },
            DiagnosticCode::WorkLimit,
        ),
    ] {
        let report = parse("p0", SemanticProfile::ClosedTraceV1, limits);
        assert!(report.document.is_none());
        assert_eq!(report.diagnostics[0].code, code);
    }

    let exact_diagnostics = parse(
        "@@",
        SemanticProfile::ClosedTraceV1,
        ParseLimits {
            max_diagnostics: 2,
            ..ParseLimits::default()
        },
    );
    assert_eq!(exact_diagnostics.diagnostics.len(), 2);
    assert!(!exact_diagnostics.stats.diagnostics_truncated);
    let one_over_diagnostic_cap = parse(
        "@@",
        SemanticProfile::ClosedTraceV1,
        ParseLimits {
            max_diagnostics: 1,
            ..ParseLimits::default()
        },
    );
    assert_eq!(one_over_diagnostic_cap.diagnostics.len(), 1);
    assert!(one_over_diagnostic_cap.stats.diagnostics_truncated);

    let formatted = format_document(&baseline.document.unwrap(), FormatLimits::default());
    let output_bytes = formatted.text.as_ref().unwrap().len();
    assert!(format_document(
        &v1("p0").document.unwrap(),
        FormatLimits {
            max_output_bytes: output_bytes,
            max_work: formatted.stats.work,
        },
    )
    .text
    .is_some());
    for (limits, code) in [
        (
            FormatLimits {
                max_output_bytes: output_bytes - 1,
                ..FormatLimits::default()
            },
            FormatErrorCode::OutputLimit,
        ),
        (
            FormatLimits {
                max_work: formatted.stats.work - 1,
                ..FormatLimits::default()
            },
            FormatErrorCode::WorkLimit,
        ),
    ] {
        let refused = format_document(&v1("p0").document.unwrap(), limits);
        assert!(refused.text.is_none());
        assert_eq!(refused.error.unwrap().code, code);
    }

    for hostile in ["\0", "💥", "é", "Yester☃", "O[0,1]💣", "\u{10ffff}"] {
        let result =
            std::panic::catch_unwind(|| parse_clean_ascii_v3(hostile, ParseLimits::default()));
        assert!(result.is_ok(), "parser unwound for {hostile:?}");
        assert!(result.unwrap().document.is_none());
    }
}
