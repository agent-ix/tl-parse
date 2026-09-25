use proptest::prelude::*;
use tl_parse::{
    dialect_v3_digest, dialect_v3_document_digest, format_clean_ascii_v3, parse,
    parse_clean_ascii_v2, parse_clean_ascii_v3, DiagnosticCode, FormatErrorCode, FormatLimits,
    ParseLimits, PastDialectRevision, PastOperatorProfile, PastParseReport, PastParseSchemaVersion,
    RecoveryAction, DIALECT_REVISION, DIALECT_V2_REVISION, DIALECT_V3_DOCUMENT,
    DIALECT_V3_REVISION, PAST_PARSE_REPORT_SCHEMA_VERSION, TL_SYNTAX_REVISION,
};
use tl_syntax::{
    FormulaDocument, FormulaSchemaVersion, Interval, Node, NodeId, NodeKind, PropositionId,
    SemanticProfile, SourceSpan, PAST_OPERATORS_V1,
};

fn span(start: u32, end: u32) -> SourceSpan {
    SourceSpan::new(start, end).unwrap()
}

fn interval(start: u32, end: u32) -> Interval {
    Interval::new(start, end).unwrap()
}

fn v3(source: &str) -> PastParseReport {
    parse_clean_ascii_v3(source, ParseLimits::default())
}

fn document(source: &str) -> FormulaDocument {
    let report = v3(source);
    assert!(
        report.diagnostics.is_empty(),
        "{source}: {:?}",
        report.diagnostics
    );
    report.document.unwrap()
}

fn canonical(source: &str) -> String {
    format_clean_ascii_v3(&document(source), FormatLimits::default())
        .text
        .unwrap()
}

// Trace: TC-046, TC-055, FR-009-AC-1, FR-009-AC-3, FR-013-AC-2
#[test]
fn v3_identity_is_closed_and_prior_dialects_are_unchanged() {
    assert_eq!(DIALECT_V3_REVISION, "tl-parse.clean-ascii/v3");
    assert_eq!(
        PAST_PARSE_REPORT_SCHEMA_VERSION,
        "tl-parse.past-parse-report/v1"
    );
    assert!(DIALECT_V3_DOCUMENT.contains(DIALECT_V3_REVISION));
    assert_eq!(
        dialect_v3_digest(),
        "e5909f5c086e7f182f15d90ac1c2e8d93a0177a1b2cbea5cc2cee84405220591"
    );
    assert_eq!(
        dialect_v3_document_digest(),
        "feaa05df2816cbd84682bb0af261a0715cce4a8ab04de75520eb481ecbe50e50"
    );
    assert_eq!(PAST_OPERATORS_V1, "tl-syntax.past-operators/v1");

    let report = v3("Yp0");
    assert_eq!(report.schema_version, PastParseSchemaVersion::V1);
    assert_eq!(report.dialect_revision, PastDialectRevision::V3);
    assert_eq!(
        report.operator_profile,
        PastOperatorProfile::PastOperatorsV1
    );
    assert_eq!(report.tl_syntax_revision, TL_SYNTAX_REVISION);
    assert_eq!(
        report.semantic_profile,
        SemanticProfile::OriginCompleteHistoryV1
    );
    assert_eq!(
        report.document.as_ref().unwrap().schema_version(),
        FormulaSchemaVersion::V2
    );

    for source in [
        "O[0,1] p0",
        "H[0,1] p0",
        "Y p0",
        "p0 S[0,1] p1",
        "p0 T[0,1] p1",
    ] {
        let v1 = parse(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert_eq!(v1.dialect_revision, DIALECT_REVISION);
        assert!(v1.document.is_none());
        assert!(v1
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnknownIdentifier));

        let v2 = parse_clean_ascii_v2(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert_eq!(DIALECT_V2_REVISION, "tl-parse.clean-ascii/v2");
        assert!(v2.document.is_none());
        assert!(v2
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnsupportedOperator));
    }
}

// Trace: TC-046, TC-055, FR-009-AC-2, FR-013-AC-2
#[test]
fn every_past_operator_builds_the_exact_v2_node_and_span() {
    let cases = [
        (
            "O[0,2]p0",
            NodeKind::Once {
                interval: interval(0, 2),
                operand: NodeId(0),
            },
            span(0, 8),
        ),
        (
            "H[1,3](p0)",
            NodeKind::Historically {
                interval: interval(1, 3),
                operand: NodeId(0),
            },
            span(0, 10),
        ),
        (
            "Yp0",
            NodeKind::StrongPrevious { operand: NodeId(0) },
            span(0, 3),
        ),
        (
            "p0S[0,1]p1",
            NodeKind::Since {
                interval: interval(0, 1),
                left: NodeId(0),
                right: NodeId(1),
            },
            span(0, 10),
        ),
        (
            "p0T[2,3]p1",
            NodeKind::Triggered {
                interval: interval(2, 3),
                left: NodeId(0),
                right: NodeId(1),
            },
            span(0, 10),
        ),
    ];
    for (source, expected, expected_span) in cases {
        let document = document(source);
        let root = document.root();
        assert_eq!(
            document.nodes()[root.0 as usize],
            Node::with_span(expected, expected_span)
        );
    }
}

// Trace: TC-046, TC-055, FR-009-AC-2, FR-013-AC-2
#[test]
fn v3_precedence_associativity_and_canonical_format_are_exact() {
    assert_eq!(canonical("O[ 0 , 2 ] (p0 & Y p1)"), "O[0,2](p0&Yp1)");
    assert_eq!(canonical("p0 S[0,1] p1 T[2,3] p2"), "p0S[0,1]p1T[2,3]p2");
    assert_eq!(
        canonical("p0 S[0,1] (p1 T[2,3] p2)"),
        "p0S[0,1](p1T[2,3]p2)"
    );
    assert_eq!(canonical("p0 | p1 S[0,1] p2 & p3"), "p0|p1S[0,1]p2&p3");
    assert_eq!(canonical("p0 -> p1 T[0,1] p2"), "p0->p1T[0,1]p2");

    let left_associative = document("p0S[0,1]p1T[2,3]p2");
    let NodeKind::Triggered { left, .. } =
        left_associative.nodes()[left_associative.root().0 as usize].kind
    else {
        panic!("root is not Triggered");
    };
    assert!(matches!(
        left_associative.nodes()[left.0 as usize].kind,
        NodeKind::Since { .. }
    ));
}

// Trace: TC-046, TC-055, TC-057, FR-009-AC-4, FR-009-AC-5, FR-013-AC-2
#[test]
fn v3_refuses_future_weak_previous_long_names_and_malformed_intervals() {
    let cases = [
        ("F[0,1]p0", DiagnosticCode::UnsupportedOperator, span(0, 1)),
        ("G[0,1]p0", DiagnosticCode::UnsupportedOperator, span(0, 1)),
        (
            "p0U[0,1]p1",
            DiagnosticCode::UnsupportedOperator,
            span(2, 3),
        ),
        (
            "p0R[0,1]p1",
            DiagnosticCode::UnsupportedOperator,
            span(2, 3),
        ),
        (
            "p0W[0,1]p1",
            DiagnosticCode::UnsupportedOperator,
            span(2, 3),
        ),
        (
            "p0M[0,1]p1",
            DiagnosticCode::UnsupportedOperator,
            span(2, 3),
        ),
        ("X p0", DiagnosticCode::UnsupportedOperator, span(0, 1)),
        ("Previous p0", DiagnosticCode::UnknownIdentifier, span(0, 8)),
        ("Yesterday", DiagnosticCode::UnknownIdentifier, span(0, 9)),
        ("YF[0,1]p0", DiagnosticCode::UnknownIdentifier, span(0, 2)),
        ("YG[0,1]p0", DiagnosticCode::UnknownIdentifier, span(0, 2)),
        (
            "Once[0,1] p0",
            DiagnosticCode::UnknownIdentifier,
            span(0, 4),
        ),
        ("o[0,1]p0", DiagnosticCode::UnknownIdentifier, span(0, 1)),
        ("O[2,1]p0", DiagnosticCode::InvalidInterval, span(1, 6)),
        (
            "p0S[01,2]p1",
            DiagnosticCode::NonCanonicalNumber,
            span(4, 6),
        ),
        (
            "p0T[0,4294967296]p1",
            DiagnosticCode::IntegerOverflow,
            span(6, 16),
        ),
    ];
    for (source, code, expected_span) in cases {
        let report = v3(source);
        assert!(report.document.is_none(), "{source}");
        let first = report.diagnostics.first().unwrap();
        assert_eq!((first.code, first.span), (code, expected_span), "{source}");
    }
}

// Trace: TC-055, TC-057, FR-013-AC-2
#[test]
fn v3_resource_limits_and_formatter_profile_gate_fail_closed() {
    let report = parse_clean_ascii_v3(
        "O[0,1]p0",
        ParseLimits {
            max_nodes: 1,
            ..ParseLimits::default()
        },
    );
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::NodeLimit);
    assert!(report.document.is_none());

    let future = FormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(0),
        vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        })],
    )
    .unwrap();
    let formatted = format_clean_ascii_v3(&future, FormatLimits::default());
    assert_eq!(formatted.error.unwrap().code, FormatErrorCode::InvalidGraph);
}

// Trace: TC-046, TC-055, FR-009-AC-3, FR-009-AC-4, FR-013-AC-2
#[test]
fn past_report_wire_is_strict_and_self_consistent() {
    let report = v3("O[0,1]p0");
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(serde_json::to_value(v3("O[0,1]p0")).unwrap(), value);
    let decoded: PastParseReport = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(decoded, report);

    let rejects = |mutate: &dyn Fn(&mut serde_json::Value)| {
        let mut changed = value.clone();
        mutate(&mut changed);
        serde_json::from_value::<PastParseReport>(changed).is_err()
    };
    assert!(rejects(&|value| value["extra"] = true.into()));
    assert!(rejects(
        &|value| value["schema_version"] = "tl-parse.past-parse-report/v2".into()
    ));
    assert!(rejects(
        &|value| value["dialect_revision"] = DIALECT_V2_REVISION.into()
    ));
    assert!(rejects(
        &|value| value["operator_profile"] = "tl-syntax.future-operators/v1".into()
    ));
    assert!(rejects(
        &|value| value["tl_syntax_revision"] = "unreviewed".into()
    ));
    assert!(rejects(
        &|value| value["semantic_profile"] = "mltl.closed-trace/v1".into()
    ));
    assert!(rejects(
        &|value| value["document"]["schema_version"] = "tl-syntax.formula/v1".into()
    ));
    assert!(rejects(&|value| value["document"] = serde_json::Value::Null));
    assert!(rejects(
        &|value| value["limits"]["max_nodes"] = usize::MAX.into()
    ));
    assert!(rejects(&|value| value["stats"]["nodes"] = 99_999.into()));
    assert!(rejects(
        &|value| value["document"]["nodes"][0]["span"] = serde_json::Value::Null
    ));
    assert!(rejects(&|value| value["diagnostics"] =
        vec![serde_json::json!({
            "code":"validation_failure",
            "severity":"error",
            "span":{"start":0,"end":0},
            "found":"<eof>",
            "expected":[],
            "recovery":"stopped",
            "message":"test"
        })]
        .into()));
}

// Trace: TC-055, TC-057, FR-013-AC-2
#[test]
fn v3_diagnostics_have_stable_complete_fields() {
    let report = v3("F[0,1]p0");
    let first = &report.diagnostics[0];
    assert_eq!(first.code, DiagnosticCode::UnsupportedOperator);
    assert_eq!(first.span, span(0, 1));
    assert_eq!(first.found, "\"F\"");
    assert_eq!(first.expected, [tl_parse::ExpectedToken::Expression]);
    assert_eq!(first.recovery, RecoveryAction::SkippedToken);
    assert_eq!(
        first.message,
        "operator \"F\" is outside tl-syntax.past-operators/v1"
    );
    assert!(report.document.is_none());
}

fn past_source_strategy() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("false".to_owned()),
        Just("true".to_owned()),
        (0_u32..=8).prop_map(|value| format!("p{value}")),
    ];
    leaf.prop_recursive(4, 48, 3, |inner| {
        prop_oneof![
            inner.clone().prop_map(|value| format!("!{value}")),
            inner.clone().prop_map(|value| format!("Y{value}")),
            (0_u32..=3, 0_u32..=3, inner.clone()).prop_map(|(start, width, value)| {
                format!("O[{start},{}]{value}", start + width)
            }),
            (0_u32..=3, 0_u32..=3, inner.clone()).prop_map(|(start, width, value)| {
                format!("H[{start},{}]{value}", start + width)
            }),
            (
                inner.clone(),
                prop_oneof![Just("&"), Just("|"), Just("->"), Just("<->")],
                inner.clone(),
            )
                .prop_map(|(left, operator, right)| format!("({left}{operator}{right})")),
            (
                inner.clone(),
                prop_oneof![Just("S"), Just("T")],
                0_u32..=3,
                0_u32..=3,
                inner,
            )
                .prop_map(|(left, operator, start, width, right)| {
                    format!("({left}{operator}[{start},{}]{right})", start + width)
                }),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: TC-055, FR-013-AC-2
    #[test]
    fn generated_v3_formulas_reach_a_parse_format_parse_fixed_point(source in past_source_strategy()) {
        let first = document(&source);
        let formatted = format_clean_ascii_v3(&first, FormatLimits::default()).text.unwrap();
        let second = document(&formatted);
        prop_assert_eq!(
            first.semantic_view(),
            second.semantic_view(),
            "{} -> {}",
            source,
            formatted
        );
    }

    // Trace: TC-046, TC-057, FR-009-AC-5, FR-013-AC-2
    #[test]
    fn arbitrary_bounded_utf8_never_unwinds_or_crosses_profiles(
        characters in prop::collection::vec(any::<char>(), 0..2048)
    ) {
        let source: String = characters.into_iter().collect();
        let report = parse_clean_ascii_v3(
            &source,
            ParseLimits {
                max_source_bytes: 4096,
                max_tokens: 512,
                max_nodes: 512,
                max_depth: 64,
                max_diagnostics: 16,
                max_work: 16_384,
            },
        );
        prop_assert!(report.stats.nodes <= 512);
        prop_assert!(report.stats.work <= 16_384);
        if let Some(document) = report.document {
            prop_assert_eq!(document.schema_version(), FormulaSchemaVersion::V2);
            prop_assert_eq!(document.semantic_profile(), SemanticProfile::OriginCompleteHistoryV1);
            let contains_future = document.nodes().iter().any(|node| {
                node.kind.temporal_family() == Some(tl_syntax::TemporalFamily::Future)
            });
            prop_assert!(!contains_future, "v3 produced a future-time node");
        }
    }
}
