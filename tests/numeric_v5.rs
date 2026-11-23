//! `tl-parse.clean-ascii/v5` numeric predicates (TL-421). UNBUILT, UNTESTED.

use std::{collections::BTreeSet, fs, path::PathBuf};

use proptest::prelude::*;
use serde::Deserialize;
use tl_parse::{
    format_clean_ascii_v5, parse, parse_clean_ascii_v2, parse_clean_ascii_v3,
    parse_clean_ascii_v4, parse_clean_ascii_v5, FormatErrorCode, FormatLimits,
    NumericDiagnosticCode, NumericParseReport, ParseArtifactLimits, ParseLimits,
    PredicateTableDocument, DIALECT_V5_REVISION, HARD_MAX_PARSE_WORK,
};
use tl_syntax::{
    FixedDecimalSignalDomain, FormulaDocument, IntegerSignalDomain, Node, NodeId, NodeKind,
    OwnedSignalDeclaration, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId,
};

const CLOSED: SemanticProfile = SemanticProfile::ClosedTraceV1;

fn integer(minimum: i64, maximum: i64) -> SignalDomain {
    SignalDomain::Integer(IntegerSignalDomain::new(minimum, maximum).unwrap())
}

fn decimal(minimum: i64, maximum: i64, scale: u8) -> SignalDomain {
    SignalDomain::FixedDecimal(FixedDecimalSignalDomain::new(minimum, maximum, scale).unwrap())
}

fn signals(extra: &[(&str, SignalDomain)]) -> Vec<OwnedSignalDeclaration> {
    let mut declared: Vec<(String, SignalDomain)> = vec![
        ("speed".to_owned(), integer(-1_000, 1_000)),
        ("temp".to_owned(), decimal(-10_000, 10_000, 2)),
        ("door".to_owned(), SignalDomain::Boolean),
        ("alt".to_owned(), integer(0, 100_000)),
        ("fine".to_owned(), decimal(-1_000_000_000_000, 1_000_000_000_000, 9)),
        (
            "micro".to_owned(),
            decimal(-1_000_000_000_000_000_000, 1_000_000_000_000_000_000, 18),
        ),
        ("tenth".to_owned(), decimal(-1_000, 1_000, 1)),
        ("big".to_owned(), integer(-(1_i64 << 62), 1_i64 << 62)),
        ("a".repeat(255), integer(0, 10)),
    ];
    for (name, domain) in extra {
        declared.push(((*name).to_owned(), *domain));
    }
    declared
        .into_iter()
        .enumerate()
        .map(|(index, (name, domain))| {
            OwnedSignalDeclaration::new(SignalId(u32::try_from(index).unwrap()), name, domain)
        })
        .collect()
}

/// p0 is bound to the Boolean `door` (signal 2), so fresh predicates start at p1.
fn standard_catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        signals(&[]),
        vec![PropositionBinding::new(PropositionId(0), SignalId(2))],
    )
    .unwrap()
}

/// A valid tl-syntax catalog whose names `each` and `p17` are unusable in v5.
fn reserved_catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        signals(&[("each", integer(0, 10)), ("p17", integer(0, 10))]),
        vec![PropositionBinding::new(PropositionId(0), SignalId(2))],
    )
    .unwrap()
}

fn v5(source: &str) -> NumericParseReport {
    parse_clean_ascii_v5(source, &standard_catalog(), CLOSED, ParseLimits::default())
}

fn first_code(report: &NumericParseReport) -> Option<NumericDiagnosticCode> {
    report.refusal.map(|refusal| refusal.code)
}

fn canonical(report: &NumericParseReport, catalog: &SignalCatalogDocument) -> String {
    let document = report.document.as_ref().expect("graph");
    format_clean_ascii_v5(
        document,
        report.predicate_table.as_ref(),
        catalog,
        FormatLimits::default(),
    )
    .text
    .expect("canonical text")
}

/// Graph identity without source spans: spans of non-canonical source differ
/// from those of its canonical text by construction.
fn shape(report: &NumericParseReport) -> Option<(NodeId, Vec<NodeKind>)> {
    report.document.as_ref().map(|document| {
        (
            document.root(),
            document.nodes().iter().map(|node| node.kind).collect(),
        )
    })
}

fn span(report: &NumericParseReport) -> (u32, u32) {
    let refusal = report.refusal.expect("refusal");
    (refusal.span.start(), refusal.span.end())
}

// ------------------------------------------------------------------ FR-028

// Trace: TC-1540, FR-028-AC-1
#[test]
fn dialect_profile_and_catalog_are_explicit_and_refused_before_lexing() {
    // The catalog is a required argument: a v5 call without one does not compile.
    let _entry: fn(&str, &SignalCatalogDocument, SemanticProfile, ParseLimits) -> NumericParseReport =
        parse_clean_ascii_v5;
    assert_eq!(DIALECT_V5_REVISION, "tl-parse.clean-ascii/v5");
    for profile in [
        SemanticProfile::OriginCompleteHistoryV1,
        SemanticProfile::InfiniteTraceV1,
    ] {
        // `=` would be a lexical error; the profile refusal precedes lexing.
        let report =
            parse_clean_ascii_v5("(speed = 5)", &standard_catalog(), profile, ParseLimits::default());
        assert_eq!(first_code(&report), Some(NumericDiagnosticCode::ProfileMismatch));
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.stats.tokens, 0);
        assert!(report.document.is_none());
    }
    for profile in [SemanticProfile::ClosedTraceV1, SemanticProfile::OnlinePrefixV1] {
        let report =
            parse_clean_ascii_v5("(speed > 5)", &standard_catalog(), profile, ParseLimits::default());
        assert!(report.document.is_some(), "{:?}", report.diagnostics);
        assert_eq!(report.semantic_profile, profile);
    }
}

// Trace: TC-1541, FR-028-AC-2
#[test]
fn existing_dialects_reject_every_v5_only_form() {
    let forms = [
        "(speed > 5)",
        "speed",
        "(x -5 == 1)",
        "each x in S: p1",
        "some x in S: p1",
        "atleast 2 x in S: p1",
        "contract c: assume p1; guarantee p1;",
        "set S = {x};",
    ];
    let limits = ParseLimits::default();
    for form in forms {
        assert!(parse(form, CLOSED, limits).document.is_none(), "v1 {form}");
        assert!(
            parse_clean_ascii_v2(form, CLOSED, limits).document.is_none(),
            "v2 {form}"
        );
        assert!(parse_clean_ascii_v3(form, limits).document.is_none(), "v3 {form}");
        assert!(
            parse_clean_ascii_v4(form, SemanticProfile::InfiniteTraceV1, "event_position", limits)
                .document
                .is_none(),
            "v4 {form}"
        );
    }
    // Byte identity of v1..v4 outputs is carried by the unchanged suites
    // (corpus, format, versioned_dialects, infinite_v4); this change edits none
    // of their sources, records or corpora.
}

// Trace: TC-1542, FR-028-AC-3
#[test]
fn past_unbounded_and_fairness_are_unsupported_with_a_locus() {
    for (source, start, end) in [
        ("O[0,1]p1", 0, 1),
        ("H[0,1]p1", 0, 1),
        ("Y p1", 0, 1),
        ("p1 S[0,1] p1", 3, 4),
        ("F[0,)p1", 1, 5),
        ("fair {p1}: p1", 5, 6),
    ] {
        let report = v5(source);
        assert!(report.document.is_none(), "{source}");
        assert_eq!(
            first_code(&report),
            Some(NumericDiagnosticCode::UnsupportedOperator),
            "{source}"
        );
        assert_eq!(span(&report), (start, end), "{source}");
    }
}

// ------------------------------------------------------------------ FR-029

// Trace: TC-1543, FR-029-AC-1
#[test]
fn allocation_is_first_occurrence_above_explicit_and_bound_identities() {
    let report = v5("(speed > 0) & p5 & (temp < 1.00) & (speed>0)");
    assert!(report.document.is_some(), "{:?}", report.diagnostics);
    // max(explicit p5, binding p0) = 5, so predicates are p6 then p7; the
    // repeated comparison reuses p6.
    let table = report.predicate_table.as_ref().unwrap();
    let text = canonical(&report, &standard_catalog());
    assert_eq!(text, "(speed > 0)&p5&(temp < 1.00)&(speed > 0)");
    assert_eq!(report.predicate_spans.len(), 2);
    let again = v5(&text);
    assert_eq!(again.predicate_table.as_ref(), Some(table));
    let document = report.document.as_ref().unwrap();
    let propositions: BTreeSet<u32> = document
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            NodeKind::Proposition { proposition } => Some(proposition.0),
            _ => None,
        })
        .collect();
    assert_eq!(propositions, BTreeSet::from([5, 6, 7]));
}

// Trace: TC-1544, FR-029-AC-2
#[test]
fn minus_and_parenthesis_forms_parse_as_specified() {
    let catalog = standard_catalog();
    assert_eq!(canonical(&v5("(speed -5 > 0)"), &catalog), "(speed - 5 > 0)");
    assert_eq!(canonical(&v5("(speed - -5 > 0)"), &catalog), "(speed - -5 > 0)");
    assert_eq!(canonical(&v5("(-speed > 0)"), &catalog), "(-speed > 0)");
    assert_eq!(canonical(&v5("(p1)"), &catalog), "p1");
    let bare = v5("(speed)");
    assert_eq!(first_code(&bare), Some(NumericDiagnosticCode::MissingToken));
    assert_eq!(span(&bare), (6, 7));
}

// Trace: TC-1544, FR-029-AC-2
#[test]
fn reserved_catalog_names_are_refused_only_where_referenced() {
    let catalog = reserved_catalog();
    let limits = ParseLimits::default();
    let unreferenced = parse_clean_ascii_v5("(speed > 0)", &catalog, CLOSED, limits);
    assert!(unreferenced.document.is_some());
    for (source, start, end) in [("(each > 0)", 1, 5), ("(p17 > 0)", 1, 4)] {
        let report = parse_clean_ascii_v5(source, &catalog, CLOSED, limits);
        assert!(report.document.is_none());
        assert_eq!(
            first_code(&report),
            Some(NumericDiagnosticCode::ReservedIdentifier)
        );
        assert_eq!(span(&report), (start, end));
    }
}

// Trace: TC-1544, FR-029-AC-2
#[test]
fn typing_refusals_carry_the_offending_sub_expression() {
    for (source, code, locus) in [
        ("(nope > 0)", NumericDiagnosticCode::UnknownSignal, "nope"),
        ("(door > 0)", NumericDiagnosticCode::NonNumericSignal, "door"),
        ("(speed + temp > 0)", NumericDiagnosticCode::ScaleMismatch, "speed + temp"),
        ("(micro * temp > 0)", NumericDiagnosticCode::ScaleOverflow, "micro * temp"),
        ("(scale(temp, 1) > tenth)", NumericDiagnosticCode::RescaleNarrowing, "scale(temp, 1)"),
        ("(big * big > 0)", NumericDiagnosticCode::RangeOverflow, "big * big"),
        ("(speed < 1 < 2)", NumericDiagnosticCode::NonAssociativeComparison, "< 1 <"),
        ("speed > 0", NumericDiagnosticCode::UnparenthesizedComparison, "speed >"),
        ("(speed / 2 > 0)", NumericDiagnosticCode::UnsupportedOperator, "/"),
    ] {
        let report = v5(source);
        assert!(report.document.is_none(), "{source}");
        assert_eq!(first_code(&report), Some(code), "{source}");
        let start = u32::try_from(source.find(locus).unwrap()).unwrap();
        let end = start + u32::try_from(locus.len()).unwrap();
        assert_eq!(span(&report), (start, end), "{source}");
    }
}

fn arith_strategy() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("speed".to_owned()),
        Just("alt".to_owned()),
        (-50_i64..50).prop_map(|value| value.to_string()),
    ];
    leaf.prop_recursive(4, 16, 2, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{a} + {b}")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a}) - ({b})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a})*({b})")),
            inner.clone().prop_map(|a| format!("-({a})")),
            inner.clone().prop_map(|a| format!("abs({a})")),
        ]
    })
}

fn formula_strategy() -> impl Strategy<Value = String> {
    let comparison = (arith_strategy(), 0_usize..6, arith_strategy()).prop_map(|(a, op, b)| {
        let operator = ["<", "<=", ">", ">=", "==", "!="][op];
        format!("({a} {operator} {b})")
    });
    let leaf = prop_oneof![comparison, Just("p0".to_owned()), Just("true".to_owned())];
    leaf.prop_recursive(3, 12, 2, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a}) & ({b})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a})|({b})")),
            inner.clone().prop_map(|a| format!("G[0,3]({a})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a}) U[1,2] ({b})")),
            inner.clone().prop_map(|a| format!("!({a})")),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(if cfg!(miri) { 4 } else { 256 }))]

    // Trace: TC-1545, FR-029-AC-3
    #[test]
    fn arbitrary_bytes_never_panic_and_yield_graph_xor_refusal(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
        let source = String::from_utf8_lossy(&bytes);
        let report = v5(&source);
        prop_assert!(report.document.is_some() != report.refusal.is_some());
        prop_assert!(report.diagnostics.len() <= report.limits.max_diagnostics);
    }

    // Trace: TC-1550, FR-031-AC-2
    #[test]
    fn canonical_text_is_a_parse_format_fixed_point(source in formula_strategy()) {
        let catalog = standard_catalog();
        let first = v5(&source);
        if first.document.is_some() {
            let text = canonical(&first, &catalog);
            let second = v5(&text);
            prop_assert_eq!(shape(&second), shape(&first));
            prop_assert_eq!(second.predicate_table.as_ref(), first.predicate_table.as_ref());
            prop_assert_eq!(canonical(&second, &catalog), text);
        }
    }
}

// ------------------------------------------------------------------ FR-030

// Trace: TC-1546, FR-030-AC-1
#[test]
fn the_closed_code_set_is_exactly_the_specified_list() {
    let spelled: Vec<&str> = NumericDiagnosticCode::ALL
        .iter()
        .map(|code| code.as_str())
        .collect();
    assert_eq!(
        spelled,
        vec![
            "source_limit",
            "token_limit",
            "node_limit",
            "depth_limit",
            "work_limit",
            "unexpected_character",
            "unexpected_token",
            "missing_token",
            "trailing_input",
            "non_canonical_number",
            "integer_overflow",
            "unsupported_operator",
            "profile_mismatch",
            "unknown_signal",
            "non_numeric_signal",
            "scale_mismatch",
            "scale_overflow",
            "rescale_narrowing",
            "range_overflow",
            "reserved_identifier",
            "non_associative_comparison",
            "unparenthesized_comparison",
            "undefined_set",
            "duplicate_set",
            "slot_outside_template",
            "aggregation_too_large",
            "duplicate_contract",
            "missing_contract_clause",
            "proposition_overflow",
        ]
    );
    for code in NumericDiagnosticCode::ALL {
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, format!("\"{}\"", code.as_str()));
    }
}

// Trace: TC-1547, FR-030-AC-2
#[test]
fn more_than_sixty_four_diagnostics_keep_the_first_and_the_decisive_refusal() {
    let source = "= ".repeat(70);
    let limits = ParseLimits {
        max_diagnostics: 64,
        ..ParseLimits::default()
    };
    let report = parse_clean_ascii_v5(&source, &standard_catalog(), CLOSED, limits);
    assert_eq!(report.diagnostics.len(), 64);
    assert!(report.stats.diagnostics_truncated);
    for (index, diagnostic) in report.diagnostics.iter().enumerate() {
        assert_eq!(diagnostic.span.start(), u32::try_from(index * 2).unwrap());
    }
    assert_eq!(
        first_code(&report),
        Some(NumericDiagnosticCode::UnexpectedCharacter)
    );
    // A later resource refusal outranks the retained syntax diagnostics.
    let capped = parse_clean_ascii_v5(
        &format!("{source}p1"),
        &standard_catalog(),
        CLOSED,
        ParseLimits {
            max_tokens: 69,
            ..limits
        },
    );
    assert_eq!(first_code(&capped), Some(NumericDiagnosticCode::TokenLimit));
}

// Trace: TC-1548, FR-030-AC-3
#[test]
fn the_report_reader_is_strict() {
    let report = v5("(speed > 0) & p0");
    let bytes = report.to_json().unwrap().into_bytes();
    let limits = ParseArtifactLimits::default();
    assert_eq!(
        NumericParseReport::from_json_bytes(&bytes, limits).unwrap(),
        report
    );
    let text = String::from_utf8(bytes.clone()).unwrap();
    let unknown_field = text.replacen('{', "{\"extra\":1,", 1);
    assert!(NumericParseReport::from_json_bytes(unknown_field.as_bytes(), limits).is_err());
    let refused = v5("(nope > 0)").to_json().unwrap();
    let unknown_code = refused.replace("unknown_signal", "made_up_code");
    assert!(NumericParseReport::from_json_bytes(unknown_code.as_bytes(), limits).is_err());
    let trailing = format!("{text} ");
    assert!(NumericParseReport::from_json_bytes(trailing.as_bytes(), limits).is_err());
    let small = ParseArtifactLimits {
        document_bytes: 8,
        ..limits
    };
    assert!(NumericParseReport::from_json_bytes(&bytes, small).is_err());
}

// ------------------------------------------------------------------ FR-031

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    revision: String,
    dialect: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CaseLimits {
    max_tokens: Option<usize>,
    max_nodes: Option<usize>,
    max_work: Option<usize>,
    max_source_bytes: Option<usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    class: String,
    path: String,
    catalog: String,
    profile: String,
    limits: Option<CaseLimits>,
    expected_canonical: Option<String>,
    expect_fixed_point: Option<bool>,
    expected_code: Option<String>,
    expected_span: Option<[u32; 2]>,
    reason: String,
}

const REQUIRED_CLASSES: [&str; 22] = [
    "valid-operator",
    "valid-node-kind",
    "scale-0",
    "scale-1",
    "scale-2",
    "scale-9",
    "scale-18",
    "diagnostic-code",
    "minus-forms",
    "paren-forms",
    "reserved-catalog",
    "literal-forms",
    "non-ascii",
    "invalid-utf8",
    "identifier-length",
    "reserved-signal-names",
    "chained-comparison",
    "single-equals",
    "division-modulo",
    "nesting",
    "token-flood",
    "unsupported-temporal",
];

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/numeric-v5")
}

fn load_manifest() -> Manifest {
    serde_json::from_slice(&fs::read(corpus_dir().join("manifest.json")).unwrap()).unwrap()
}

fn missing_classes(manifest: &Manifest) -> Vec<&'static str> {
    let present: BTreeSet<&str> = manifest.cases.iter().map(|case| case.class.as_str()).collect();
    REQUIRED_CLASSES
        .iter()
        .copied()
        .filter(|class| !present.contains(class))
        .collect()
}

fn case_limits(case: &Case) -> ParseLimits {
    let defaults = ParseLimits::default();
    let Some(limits) = case.limits.as_ref() else {
        return defaults;
    };
    ParseLimits {
        max_tokens: limits.max_tokens.unwrap_or(defaults.max_tokens),
        max_nodes: limits.max_nodes.unwrap_or(defaults.max_nodes),
        max_work: limits.max_work.unwrap_or(defaults.max_work),
        max_source_bytes: limits.max_source_bytes.unwrap_or(defaults.max_source_bytes),
        ..defaults
    }
}

fn replay(case: &Case) -> Option<NumericParseReport> {
    let bytes = fs::read(corpus_dir().join(&case.path)).unwrap();
    let Ok(source) = std::str::from_utf8(&bytes) else {
        // Byte-level case: invalid UTF-8 is refused before the parser.
        assert_eq!(case.class, "invalid-utf8", "{}", case.id);
        return None;
    };
    let catalog = match case.catalog.as_str() {
        "standard" => standard_catalog(),
        "reserved" => reserved_catalog(),
        other => panic!("unknown catalog {other}"),
    };
    let profile = match case.profile.as_str() {
        "closed" => CLOSED,
        "online" => SemanticProfile::OnlinePrefixV1,
        "infinite" => SemanticProfile::InfiniteTraceV1,
        other => panic!("unknown profile {other}"),
    };
    let report = parse_clean_ascii_v5(source, &catalog, profile, case_limits(case));
    assert!(!case.reason.is_empty(), "{}", case.id);
    match (&case.expected_canonical, &case.expected_code) {
        (Some(expected), None) => {
            assert!(report.document.is_some(), "{}: {:?}", case.id, report.diagnostics);
            let text = canonical(&report, &catalog);
            assert_eq!(&text, expected, "{}", case.id);
            let again = parse_clean_ascii_v5(&text, &catalog, profile, ParseLimits::default());
            assert_eq!(shape(&again), shape(&report), "{}", case.id);
            assert_eq!(again.predicate_table, report.predicate_table, "{}", case.id);
        }
        (None, Some(code)) => {
            assert!(report.document.is_none(), "{}", case.id);
            let refusal = report.refusal.expect("refusal");
            assert_eq!(refusal.code.as_str(), code, "{}", case.id);
            let [start, end] = case.expected_span.expect("span");
            assert_eq!((refusal.span.start(), refusal.span.end()), (start, end), "{}", case.id);
        }
        (None, None) if case.expect_fixed_point == Some(true) => {
            let text = canonical(&report, &catalog);
            let again = parse_clean_ascii_v5(&text, &catalog, profile, ParseLimits::default());
            assert_eq!(canonical(&again, &catalog), text, "{}", case.id);
        }
        _ => panic!("case {} has no single expected outcome", case.id),
    }
    Some(report)
}

// Trace: TC-1549, FR-031-AC-1, StR-014-VC-1, TC-1559
#[test]
fn every_valid_corpus_case_round_trips_to_its_recorded_canonical_text() {
    let manifest = load_manifest();
    let mut valid = 0;
    for case in &manifest.cases {
        if case.expected_canonical.is_some() || case.expect_fixed_point == Some(true) {
            replay(case);
            valid += 1;
        }
    }
    assert!(valid >= 20);
}

// Trace: TC-1551, FR-031-AC-3
#[test]
fn unrepresentable_graphs_are_refused_with_no_text() {
    let catalog = standard_catalog();
    let past = FormulaDocument::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(1),
        vec![
            Node::new(NodeKind::Proposition {
                proposition: PropositionId(1),
            }),
            Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
        ],
    );
    if let Ok(past) = past {
        let report = format_clean_ascii_v5(&past, None, &catalog, FormatLimits::default());
        assert_eq!(report.text, None);
        assert_eq!(
            report.error.map(|error| error.code),
            Some(FormatErrorCode::UnrepresentableGraph)
        );
    }
    // A table whose signal is absent from the supplied catalog.
    let parsed = v5("(speed > 0)");
    let table: &PredicateTableDocument = parsed.predicate_table.as_ref().unwrap();
    let other = SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(2),
            "door".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(2))],
    )
    .unwrap();
    let report = format_clean_ascii_v5(
        parsed.document.as_ref().unwrap(),
        Some(table),
        &other,
        FormatLimits::default(),
    );
    assert_eq!(report.text, None);
    assert_eq!(
        report.error.map(|error| error.code),
        Some(FormatErrorCode::UnrepresentableGraph)
    );
    // A proposition both bound by the catalog and defined by the table: bind p1.
    let both = SignalCatalogDocument::new(
        signals(&[]),
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(2)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap();
    let report = format_clean_ascii_v5(
        parsed.document.as_ref().unwrap(),
        Some(table),
        &both,
        FormatLimits::default(),
    );
    assert_eq!(
        report.error.map(|error| error.code),
        Some(FormatErrorCode::UnrepresentableGraph)
    );
}

// ------------------------------------------------------------------ FR-033

// Trace: TC-1555, FR-033-AC-1
#[test]
fn hostile_corpus_covers_every_class_and_matches_the_parser() {
    let manifest = load_manifest();
    assert_eq!(manifest.schema_version, "tl-parse.numeric-corpus/v1");
    assert_eq!(manifest.revision, tl_parse::CORPUS_REVISION);
    assert_eq!(manifest.dialect, DIALECT_V5_REVISION);
    assert_eq!(missing_classes(&manifest), Vec::<&str>::new());
    let ids: BTreeSet<&str> = manifest.cases.iter().map(|case| case.id.as_str()).collect();
    assert_eq!(ids.len(), manifest.cases.len());
    for case in &manifest.cases {
        replay(case);
    }
    // Removing a class fails the check.
    let mut reduced = load_manifest();
    reduced.cases.retain(|case| case.class != "nesting");
    assert_eq!(missing_classes(&reduced), vec!["nesting"]);
}

// Trace: TC-1556, FR-033-AC-2
#[test]
fn fuzz_target_and_seeds_are_checked_in() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("fuzz/fuzz_targets/numeric_v5_roundtrip.rs").is_file());
    let seeds = fs::read_dir(root.join("fuzz/corpus/numeric_v5_roundtrip"))
        .unwrap()
        .count();
    assert!(seeds >= 4);
}

// Trace: TC-1557, FR-033-AC-3, TC-1560, StR-014-VC-2
#[test]
fn hostile_corpus_stays_within_the_hard_maxima() {
    for case in &load_manifest().cases {
        if let Some(report) = replay(case) {
            assert!(report.stats.work <= HARD_MAX_PARSE_WORK, "{}", case.id);
            assert!(report.stats.work <= report.limits.max_work, "{}", case.id);
            assert!(report.stats.tokens <= report.limits.max_tokens, "{}", case.id);
            assert!(report.stats.nodes <= report.limits.max_nodes, "{}", case.id);
            assert!(report.stats.max_depth <= report.limits.max_depth + 1, "{}", case.id);
        }
    }
}

// Trace: TC-1558, NFR-016-AC-1
#[test]
fn repeated_runs_are_byte_identical() {
    for case in &load_manifest().cases {
        let first = replay(case);
        let second = replay(case);
        let first_json = first.map(|report| report.to_json().unwrap());
        let second_json = second.map(|report| report.to_json().unwrap());
        assert_eq!(first_json, second_json, "{}", case.id);
    }
}
