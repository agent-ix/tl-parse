use tl_parse::{
    format_document, parse, report_json, source_limit_report, DiagnosticCode, ExpectedToken,
    FormatLimits, ParseLimits, RecoveryAction,
};
use tl_syntax::{NodeKind, SemanticProfile};

fn parse_closed(source: &str) -> tl_parse::ParseReport {
    parse(
        source,
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
    )
}

fn canonical(source: &str) -> String {
    let report = parse_closed(source);
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    format_document(
        report.document.as_ref().expect("valid fixture"),
        FormatLimits::default(),
    )
    .text
    .expect("format succeeds")
}

// Trace: TC-001, TC-005, FR-001-AC-1, FR-002-AC-1
#[test]
fn complete_dialect_vocabulary_maps_to_exact_node_kinds() {
    let report = parse_closed("false <-> true -> p0 | !p1 & F[0,1]p2 U[1,2] G[2,3]p3 R[0,0] p4");
    let document = report.document.expect("complete vocabulary parses");
    let kinds: Vec<_> = document.nodes().iter().map(|node| node.kind).collect();
    assert!(kinds.contains(&NodeKind::False));
    assert!(kinds.contains(&NodeKind::True));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Proposition { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Not { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::And { .. })));
    assert!(kinds.iter().any(|kind| matches!(kind, NodeKind::Or { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Implies { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Equivalent { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Future { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Globally { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Until { .. })));
    assert!(kinds
        .iter()
        .any(|kind| matches!(kind, NodeKind::Release { .. })));
}

// Trace: TC-002, FR-001-AC-1
#[test]
fn precedence_and_associativity_are_unambiguous() {
    assert_eq!(
        canonical("p0 <-> p1 -> p2 | p3 & p4 U[1,2] p5"),
        "p0<->p1->p2|p3&p4U[1,2]p5"
    );
    assert_eq!(canonical("p0 -> p1 -> p2"), "p0->p1->p2");
    assert_eq!(canonical("p0 <-> p1 <-> p2"), "p0<->p1<->p2");
    assert_eq!(canonical("p0 U[0,1] p1 U[2,3] p2"), "p0U[0,1]p1U[2,3]p2");
}

// Trace: TC-003, FR-001-AC-2
#[test]
fn invalid_characters_and_identifiers_have_utf8_byte_spans() {
    let report = parse_closed("p0 λ");
    assert!(report.document.is_none());
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|item| item.code == DiagnosticCode::UnexpectedCharacter)
        .expect("unexpected character diagnostic");
    assert_eq!((diagnostic.span.start(), diagnostic.span.end()), (3, 5));
    assert_eq!(diagnostic.recovery, RecoveryAction::SkippedToken);

    let report = parse_closed("request");
    assert_eq!(
        report.diagnostics[0].code,
        DiagnosticCode::UnknownIdentifier
    );
    assert_eq!(
        (
            report.diagnostics[0].span.start(),
            report.diagnostics[0].span.end()
        ),
        (0, 7)
    );
}

// Trace: TC-004, FR-001-AC-2
#[test]
fn noncanonical_overflowing_and_inverted_numbers_are_rejected() {
    assert_eq!(
        parse_closed("p01").diagnostics[0].code,
        DiagnosticCode::NonCanonicalNumber
    );
    assert_eq!(
        parse_closed("p4294967296").diagnostics[0].code,
        DiagnosticCode::IntegerOverflow
    );
    assert!(parse_closed("F[01,2]p0")
        .diagnostics
        .iter()
        .any(|item| item.code == DiagnosticCode::NonCanonicalNumber));
    assert!(parse_closed("F[3,2]p0")
        .diagnostics
        .iter()
        .any(|item| item.code == DiagnosticCode::InvalidInterval));
}

// Trace: TC-006, FR-002-AC-1
#[test]
fn graph_is_topological_and_nodes_retain_full_source_spans() {
    let report = parse_closed("F[1,2](p7 & !p8)");
    let document = report.document.expect("valid formula");
    document.validate().expect("pinned validator accepts graph");
    for (index, node) in document.nodes().iter().enumerate() {
        match node.kind {
            NodeKind::Not { operand }
            | NodeKind::Future { operand, .. }
            | NodeKind::Globally { operand, .. } => assert!((operand.0 as usize) < index),
            NodeKind::And { left, right }
            | NodeKind::Or { left, right }
            | NodeKind::Implies { left, right }
            | NodeKind::Equivalent { left, right }
            | NodeKind::Until { left, right, .. }
            | NodeKind::Release { left, right, .. } => {
                assert!((left.0 as usize) < index);
                assert!((right.0 as usize) < index);
            }
            NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => {}
        }
        assert!(node.span.is_some());
    }
    let root = &document.nodes()[document.root().0 as usize];
    let root_span = root.span.expect("root span");
    assert_eq!((root_span.start(), root_span.end()), (0, 16));
}

// Trace: TC-007, FR-002-AC-2, StR-001-VC-2
#[test]
fn every_success_validates_and_preserves_the_selected_profile() {
    for profile in [
        SemanticProfile::ClosedTraceV1,
        SemanticProfile::OnlinePrefixV1,
    ] {
        let report = parse("G[0,4](p0 -> p1)", profile, ParseLimits::default());
        assert_eq!(report.semantic_profile, profile);
        let document = report.document.expect("valid document");
        assert_eq!(document.semantic_profile(), profile);
        assert_eq!(document.validate().expect("valid graph").profile(), profile);
    }
}

// Trace: TC-008, FR-002-AC-3
#[test]
fn recovery_diagnostics_never_expose_partial_documents() {
    let missing = parse_closed("F[1,2 p0");
    assert!(missing.document.is_none());
    let diagnostic = missing
        .diagnostics
        .iter()
        .find(|item| item.expected == [ExpectedToken::RightBracket])
        .expect("right-bracket recovery");
    assert_eq!(diagnostic.recovery, RecoveryAction::InsertedToken);

    let trailing = parse_closed("p0 p1");
    assert!(trailing.document.is_none());
    assert!(trailing
        .diagnostics
        .iter()
        .any(|item| item.code == DiagnosticCode::TrailingInput));

    let absent = parse_closed("");
    assert!(absent.document.is_none());
    assert_eq!(absent.diagnostics[0].expected, [ExpectedToken::Expression]);
}

// Trace: TC-008, TC-012, FR-003-AC-1, FR-003-AC-2
#[test]
fn malformed_interval_recovery_has_exact_structured_diagnostics() {
    let cases = [
        (
            "F p0",
            DiagnosticCode::MissingToken,
            vec![ExpectedToken::LeftBracket],
            RecoveryAction::InsertedToken,
        ),
        (
            "F[,1]p0",
            DiagnosticCode::UnexpectedToken,
            vec![ExpectedToken::Integer],
            RecoveryAction::SkippedToken,
        ),
        (
            "F[0 p0",
            DiagnosticCode::MissingToken,
            vec![ExpectedToken::Comma],
            RecoveryAction::InsertedToken,
        ),
        (
            "F[0,]p0",
            DiagnosticCode::UnexpectedToken,
            vec![ExpectedToken::Integer],
            RecoveryAction::SkippedToken,
        ),
        (
            "F[0,1p0",
            DiagnosticCode::MissingToken,
            vec![ExpectedToken::RightBracket],
            RecoveryAction::InsertedToken,
        ),
        (
            "F[2,1]p0",
            DiagnosticCode::InvalidInterval,
            vec![],
            RecoveryAction::None,
        ),
    ];
    for (source, code, expected, recovery) in cases {
        let report = parse_closed(source);
        assert!(
            report.document.is_none(),
            "malformed interval {source:?} succeeded"
        );
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == code)
            .unwrap_or_else(|| panic!("missing {code:?} for {source:?}: {:?}", report.diagnostics));
        assert_eq!(
            diagnostic.expected, expected,
            "unexpected token set for {source:?}"
        );
        assert_eq!(
            diagnostic.recovery, recovery,
            "unexpected recovery for {source:?}"
        );
    }
}

// Trace: TC-009, TC-010, FR-003-AC-1
#[test]
fn diagnostics_have_stable_golden_fields_and_versioned_json() {
    let report = parse_closed("F[2,1]p0");
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|item| item.code == DiagnosticCode::InvalidInterval)
        .expect("interval diagnostic");
    assert_eq!((diagnostic.span.start(), diagnostic.span.end()), (1, 6));
    assert_eq!(diagnostic.found, "\"[2,1]\"");
    assert!(diagnostic.expected.is_empty());
    assert_eq!(diagnostic.recovery, RecoveryAction::None);
    assert_eq!(report.schema_version, "tl-parse.diagnostics/v1");
    let json = report_json(&report).expect("report serializes");
    assert!(json.starts_with("{\"schema_version\":\"tl-parse.diagnostics/v1\""));
    assert!(json.contains("\"code\":\"invalid_interval\""));
    assert!(json.contains("\"semantic_profile\":\"mltl.closed-trace/v1\""));
    let decoded: tl_parse::ParseReport = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, report);
    let with_unknown = json.replacen('{', "{\"fabricated\":true,", 1);
    assert!(serde_json::from_str::<tl_parse::ParseReport>(&with_unknown).is_err());
    assert!(diagnostic.to_string().contains("invalid_interval"));
}

// Trace: TC-011, FR-003-AC-2, NFR-001-AC-2
#[test]
fn source_token_node_and_depth_limits_fail_closed() {
    let limits = ParseLimits {
        max_source_bytes: 1,
        ..ParseLimits::default()
    };
    assert_eq!(
        parse("p0", SemanticProfile::ClosedTraceV1, limits).diagnostics[0].code,
        DiagnosticCode::SourceLimit
    );

    let limits = ParseLimits {
        max_tokens: 1,
        ..ParseLimits::default()
    };
    assert_eq!(
        parse("p0 & p1", SemanticProfile::ClosedTraceV1, limits).diagnostics[0].code,
        DiagnosticCode::TokenLimit
    );

    let limits = ParseLimits {
        max_nodes: 1,
        ..ParseLimits::default()
    };
    assert!(parse("!p0", SemanticProfile::ClosedTraceV1, limits)
        .diagnostics
        .iter()
        .any(|item| item.code == DiagnosticCode::NodeLimit));

    let limits = ParseLimits {
        max_depth: 1,
        ..ParseLimits::default()
    };
    assert!(parse("!p0", SemanticProfile::ClosedTraceV1, limits)
        .diagnostics
        .iter()
        .any(|item| item.code == DiagnosticCode::DepthLimit));
}

// Trace: TC-021, FR-005-AC-3, NFR-001-AC-1
#[test]
fn source_limit_report_is_total_at_every_limit_boundary() {
    let limits = ParseLimits {
        max_source_bytes: 4_096,
        ..ParseLimits::default()
    };
    for source_bytes in [0, 1, 4_095, 4_096, 4_097] {
        let report = source_limit_report(source_bytes, SemanticProfile::ClosedTraceV1, limits);
        assert_eq!(report.stats.source_bytes, source_bytes);
        assert_eq!(report.diagnostics[0].code, DiagnosticCode::SourceLimit);
        let span = report.diagnostics[0].span;
        assert!(span.start() <= span.end());
        assert!(span.end() as usize <= source_bytes);
    }
}

// Trace: TC-012, FR-003-AC-2, NFR-001-AC-2
#[test]
fn diagnostic_and_work_limits_fail_closed_without_growth() {
    let limits = ParseLimits {
        max_work: 1,
        ..ParseLimits::default()
    };
    let report = parse("p0", SemanticProfile::ClosedTraceV1, limits);
    assert!(report.document.is_none());
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::WorkLimit);
    assert!(report.stats.work <= 1);

    let limits = ParseLimits {
        max_diagnostics: 1,
        ..ParseLimits::default()
    };
    let report = parse("bad worse worst", SemanticProfile::ClosedTraceV1, limits);
    assert!(report.document.is_none());
    assert_eq!(report.diagnostics.len(), 1);
    assert!(report.stats.diagnostics_truncated);

    let limits = ParseLimits {
        max_diagnostics: 1,
        ..ParseLimits::default()
    };
    let report = parse("p0 p1 p2", SemanticProfile::ClosedTraceV1, limits);
    assert!(report.document.is_none());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::TrailingInput);
    assert_eq!(report.diagnostics[0].recovery, RecoveryAction::SkippedToken);
    assert!(report.stats.diagnostics_truncated);

    let limits = ParseLimits {
        max_diagnostics: 0,
        ..ParseLimits::default()
    };
    let report = parse("@", SemanticProfile::ClosedTraceV1, limits);
    assert!(report.document.is_none());
    assert!(report.diagnostics.is_empty());
    assert!(report.stats.diagnostics_truncated);
}

// Trace: TC-013, FR-003-AC-3, NFR-001-AC-1
#[test]
fn identical_requests_produce_byte_identical_reports() {
    let source = "G[0,3](p0 -> p1) extra";
    let first = parse_closed(source);
    let second = parse_closed(source);
    assert_eq!(first, second);
    assert_eq!(
        report_json(&first).expect("first JSON"),
        report_json(&second).expect("second JSON")
    );
}
