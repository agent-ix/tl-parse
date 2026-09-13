use std::{fs, path::PathBuf, process::Command};

use proptest::prelude::*;
use tl_parse::{
    dialect_v2_digest, dialect_v2_document_digest, format_document, parse, parse_clean_ascii_v2,
    report_json, DerivedOperator, DerivedParseReport, DiagnosticCode, FormatLimits, ParseLimits,
    DERIVED_PARSE_REPORT_SCHEMA_VERSION, DIALECT_REVISION, DIALECT_V2_DOCUMENT,
    DIALECT_V2_REVISION, TL_SYNTAX_REVISION,
};
use tl_syntax::{
    Formula, FormulaDocument, FutureLoweringRequest, Interval, Node, NodeId, NodeKind,
    PropositionId, RawBounds, SemanticProfile, SourceSpan, FUTURE_LOWERING_REQUEST_V1,
    FUTURE_OPERATORS_V1,
};

const PROFILE: SemanticProfile = SemanticProfile::ClosedTraceV1;

fn span(start: u32, end: u32) -> SourceSpan {
    SourceSpan::new(start, end).unwrap()
}

fn interval(start: u32, end: u32) -> Interval {
    Interval::new(start, end).unwrap()
}

fn v2(source: &str) -> DerivedParseReport {
    parse_clean_ascii_v2(source, PROFILE, ParseLimits::default())
}

fn first_code(report: &DerivedParseReport) -> (DiagnosticCode, SourceSpan) {
    let diagnostic = report.diagnostics.first().expect("a diagnostic");
    (diagnostic.code, diagnostic.span)
}

/// A generated v2 expression. Binary forms render fully parenthesized, so the
/// generator never depends on precedence; TC-041 pins precedence explicitly.
#[derive(Clone, Debug)]
enum Expr {
    Proposition(u32),
    True,
    Not(Box<Expr>),
    Future(u32, u32, Box<Expr>),
    Boolean(&'static str, Box<Expr>, Box<Expr>),
    Temporal(&'static str, u32, u32, Box<Expr>, Box<Expr>),
}

fn expr_strategy() -> impl Strategy<Value = Expr> {
    let leaf = prop_oneof![(0_u32..=4).prop_map(Expr::Proposition), Just(Expr::True),];
    leaf.prop_recursive(4, 32, 2, |inner| {
        prop_oneof![
            inner
                .clone()
                .prop_map(|operand| Expr::Not(Box::new(operand))),
            (0_u32..=3, 0_u32..=3, inner.clone()).prop_map(|(start, width, operand)| Expr::Future(
                start,
                start + width,
                Box::new(operand)
            )),
            (
                prop_oneof![Just("&"), Just("|"), Just("->")],
                inner.clone(),
                inner.clone()
            )
                .prop_map(|(operator, left, right)| Expr::Boolean(
                    operator,
                    Box::new(left),
                    Box::new(right)
                )),
            (
                prop_oneof![Just("U"), Just("R"), Just("W"), Just("M")],
                0_u32..=3,
                0_u32..=3,
                inner.clone(),
                inner
            )
                .prop_map(|(operator, start, width, left, right)| Expr::Temporal(
                    operator,
                    start,
                    start + width,
                    Box::new(left),
                    Box::new(right)
                )),
        ]
    })
}

/// Direct construction: renders the source and builds the expected graph with
/// tl-syntax lowering, returning the node and its source extent.
struct Builder {
    source: String,
    nodes: Vec<Node>,
    lowerings: usize,
}

impl Builder {
    fn push(&mut self, kind: NodeKind, start: usize, end: usize) -> NodeId {
        self.nodes
            .push(Node::with_span(kind, span(start as u32, end as u32)));
        NodeId(self.nodes.len() as u32 - 1)
    }

    fn emit(&mut self, expr: &Expr) -> (NodeId, usize, usize) {
        let start = self.source.len();
        match expr {
            Expr::Proposition(value) => {
                self.source.push_str(&format!("p{value}"));
                let end = self.source.len();
                let proposition = PropositionId(*value);
                (
                    self.push(NodeKind::Proposition { proposition }, start, end),
                    start,
                    end,
                )
            }
            Expr::True => {
                self.source.push_str("true");
                let end = self.source.len();
                (self.push(NodeKind::True, start, end), start, end)
            }
            Expr::Not(operand) => {
                self.source.push('!');
                let (operand, _, end) = self.emit(operand);
                (self.push(NodeKind::Not { operand }, start, end), start, end)
            }
            Expr::Future(low, high, operand) => {
                self.source.push_str(&format!("F[{low},{high}]"));
                let (operand, _, end) = self.emit(operand);
                let kind = NodeKind::Future {
                    interval: interval(*low, *high),
                    operand,
                };
                (self.push(kind, start, end), start, end)
            }
            Expr::Boolean(operator, left, right) => {
                self.source.push('(');
                let (left, left_start, _) = self.emit(left);
                self.source.push_str(&format!(" {operator} "));
                let (right, _, right_end) = self.emit(right);
                let kind = match *operator {
                    "&" => NodeKind::And { left, right },
                    "|" => NodeKind::Or { left, right },
                    _ => NodeKind::Implies { left, right },
                };
                let node = self.push(kind, left_start, right_end);
                self.source.push(')');
                (node, start, self.source.len())
            }
            Expr::Temporal(operator, low, high, left, right) => {
                self.source.push('(');
                let (left, left_start, _) = self.emit(left);
                self.source.push(' ');
                let operator_start = self.source.len();
                self.source.push_str(&format!("{operator}[{low},{high}]"));
                let operator_end = self.source.len();
                self.source.push(' ');
                let (right, _, right_end) = self.emit(right);
                let interval = interval(*low, *high);
                let node = match *operator {
                    "U" => self.push(
                        NodeKind::Until {
                            interval,
                            left,
                            right,
                        },
                        left_start,
                        right_end,
                    ),
                    "R" => self.push(
                        NodeKind::Release {
                            interval,
                            left,
                            right,
                        },
                        left_start,
                        right_end,
                    ),
                    kind => {
                        let root = NodeId(self.nodes.len() as u32 - 1);
                        let formula = Formula::new(PROFILE, root, &self.nodes).unwrap();
                        let lowering = FutureLoweringRequest {
                            request_identity: FUTURE_LOWERING_REQUEST_V1.as_bytes(),
                            operator_profile: FUTURE_OPERATORS_V1.as_bytes(),
                            kind: kind.as_bytes(),
                            semantic_profile: PROFILE.as_str().as_bytes(),
                            formula,
                            left: u64::from(left.0),
                            right: u64::from(right.0),
                            interval: Some(RawBounds::new(u64::from(*low), u64::from(*high))),
                            operator_span: Some(RawBounds::new(
                                operator_start as u64,
                                operator_end as u64,
                            )),
                            expression_span: Some(RawBounds::new(
                                left_start as u64,
                                right_end as u64,
                            )),
                        }
                        .lower()
                        .unwrap();
                        self.nodes.extend_from_slice(lowering.nodes());
                        self.lowerings += 1;
                        lowering.root()
                    }
                };
                self.source.push(')');
                (node, start, self.source.len())
            }
        }
    }
}

fn build(expr: &Expr) -> (String, FormulaDocument, usize) {
    let mut builder = Builder {
        source: String::new(),
        nodes: Vec::new(),
        lowerings: 0,
    };
    let (root, _, _) = builder.emit(expr);
    let document = FormulaDocument::new(PROFILE, root, builder.nodes).unwrap();
    (builder.source, document, builder.lowerings)
}

// Trace: TC-039, FR-008-AC-1, StR-001-VC-1
#[test]
fn v2_identity_is_explicit_and_v1_is_unchanged() {
    assert_eq!(DIALECT_V2_REVISION, "tl-parse.clean-ascii/v2");
    assert_eq!(DIALECT_REVISION, "tl-parse.clean-ascii/v1");
    assert_eq!(
        DERIVED_PARSE_REPORT_SCHEMA_VERSION,
        "tl-parse.derived-parse-report/v1"
    );
    assert!(DIALECT_V2_DOCUMENT.contains(DIALECT_V2_REVISION));
    assert_eq!(
        dialect_v2_digest(),
        "8542aebdc8ecde044659d89f293324d958aee2fac06dbef203d5d12ef71a59b8"
    );
    assert_eq!(
        dialect_v2_document_digest(),
        "f3365451484a0585170d9382e253b8c39b75b77a20ca5e478629177b7faf3f73"
    );

    let report = v2("p0 W[0,1] p1");
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema_version"], DERIVED_PARSE_REPORT_SCHEMA_VERSION);
    assert_eq!(json["dialect_revision"], DIALECT_V2_REVISION);
    assert_eq!(json["operator_profile"], FUTURE_OPERATORS_V1);
    assert_eq!(json["tl_syntax_revision"], TL_SYNTAX_REVISION);

    for source in ["p0 W[0,1] p1", "p0 M[0,1] p1", "X p0", "p0 S[0,1] p1"] {
        let report = parse(source, PROFILE, ParseLimits::default());
        assert!(report.document.is_none(), "{source}");
        assert_eq!(report.dialect_revision, DIALECT_REVISION);
        assert!(report
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != DiagnosticCode::UnsupportedOperator));
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::UnknownIdentifier));
        let json = report_json(&report).unwrap();
        assert!(!json.contains("lowerings") && !json.contains("clean-ascii/v2"));
    }

    // A v1 source yields the same document under both dialects.
    let source = "p0 U[0,2] (p1 & !p2) -> G[1,1] true";
    let v1 = parse(source, PROFILE, ParseLimits::default());
    let v2 = v2(source);
    assert_eq!(v1.document, v2.document);
    assert!(v2.lowerings.is_empty());
    assert_eq!(v1.stats, v2.stats);
    assert_eq!(v1.diagnostics, v2.diagnostics);
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: TC-040, FR-008-AC-2
    #[test]
    fn v2_parse_matches_direct_tl_syntax_lowering(expr in expr_strategy()) {
        let (source, expected, lowerings) = build(&expr);
        let report = v2(&source);
        prop_assert!(report.diagnostics.is_empty(), "{source}: {:?}", report.diagnostics);
        prop_assert_eq!(report.document.as_ref(), Some(&expected), "{}", source);
        prop_assert_eq!(report.lowerings.len(), lowerings);
    }

    // Trace: TC-043, FR-008-AC-4
    #[test]
    fn v2_canonical_text_is_primitive_and_reaches_a_fixed_point(expr in expr_strategy()) {
        let (source, expected, _) = build(&expr);
        let document = v2(&source).document.unwrap();
        prop_assert_eq!(
            serde_json::to_string(&document).unwrap(),
            serde_json::to_string(&expected).unwrap()
        );
        let text = format_document(&document, FormatLimits::default()).text.unwrap();
        prop_assert!(!text.contains('W') && !text.contains('M'), "{}", text);
        prop_assert!(parse(&text, PROFILE, ParseLimits::default()).document.is_some());
        let reparsed = v2(&text).document.unwrap();
        let again = format_document(&reparsed, FormatLimits::default()).text.unwrap();
        prop_assert_eq!(again, text);
    }
}

// Trace: TC-041, FR-008-AC-2
#[test]
fn v2_precedence_associativity_and_spans_are_exact() {
    let report = v2("p0 W[1,2] p1");
    let document = report.document.as_ref().unwrap();
    let whole = span(0, 12);
    assert_eq!(
        document.nodes(),
        &[
            Node::with_span(
                NodeKind::Proposition {
                    proposition: PropositionId(0)
                },
                span(0, 2)
            ),
            Node::with_span(
                NodeKind::Proposition {
                    proposition: PropositionId(1)
                },
                span(10, 12)
            ),
            Node::with_span(
                NodeKind::Until {
                    interval: interval(1, 2),
                    left: NodeId(0),
                    right: NodeId(1)
                },
                whole
            ),
            Node::with_span(
                NodeKind::Globally {
                    interval: interval(1, 2),
                    operand: NodeId(0)
                },
                whole
            ),
            Node::with_span(
                NodeKind::Or {
                    left: NodeId(2),
                    right: NodeId(3)
                },
                whole
            ),
        ]
    );
    assert_eq!(document.root(), NodeId(4));
    let record = report.lowerings[0];
    assert_eq!(record.kind, DerivedOperator::WeakUntil);
    assert_eq!(record.operator_span, span(3, 9));
    assert_eq!(record.expression_span, whole);
    assert_eq!(
        (
            record.left,
            record.right,
            record.first_generated,
            record.root
        ),
        (NodeId(0), NodeId(1), NodeId(2), NodeId(4))
    );

    // Strong release lowers to R, F, And.
    let document = v2("p0 M[0,3] p1").document.unwrap();
    assert!(matches!(document.nodes()[2].kind, NodeKind::Release { .. }));
    assert!(matches!(document.nodes()[3].kind, NodeKind::Future { .. }));
    assert!(matches!(document.nodes()[4].kind, NodeKind::And { .. }));

    // Left associativity at the U/R level: (p0 U p1) W p2.
    let report = v2("p0 U[0,1] p1 W[2,3] p2");
    let document = report.document.as_ref().unwrap();
    assert!(matches!(document.nodes()[2].kind, NodeKind::Until { .. }));
    assert_eq!(report.lowerings[0].left, NodeId(2));
    assert_eq!(report.lowerings[0].operator_span, span(13, 19));
    assert_eq!(report.lowerings[0].expression_span, span(0, 22));

    // (p0 W p1) M p2 lowers twice, the second over the first root.
    let report = v2("p0 W[0,1] p1 M[0,1] p2");
    assert_eq!(report.lowerings.len(), 2);
    assert_eq!(report.lowerings[1].left, report.lowerings[0].root);
    assert_eq!(report.document.unwrap().root(), report.lowerings[1].root);

    // W and M bind tighter than & and |.
    let document = v2("p0 | p1 M[0,1] p2").document.unwrap();
    let NodeKind::Or { left, right } = document.nodes()[document.root().0 as usize].kind else {
        panic!("root is not Or");
    };
    assert_eq!(left, NodeId(0));
    assert!(matches!(
        document.nodes()[right.0 as usize].kind,
        NodeKind::And { .. }
    ));
    let document = v2("p0 W[0,1] p1 & p2").document.unwrap();
    let NodeKind::And { left, .. } = document.nodes()[document.root().0 as usize].kind else {
        panic!("root is not And");
    };
    assert!(matches!(
        document.nodes()[left.0 as usize].kind,
        NodeKind::Or { .. }
    ));

    // W and M bind tighter than -> and <->, which stay right-associative.
    let document = v2("p0 -> p1 W[0,1] p2").document.unwrap();
    let NodeKind::Implies { left, right } = document.nodes()[document.root().0 as usize].kind
    else {
        panic!("root is not Implies");
    };
    assert_eq!(left, NodeId(0));
    assert!(matches!(
        document.nodes()[right.0 as usize].kind,
        NodeKind::Or { .. }
    ));
    let document = v2("p0 M[0,1] p1 <-> p2").document.unwrap();
    let NodeKind::Equivalent { left, right } = document.nodes()[document.root().0 as usize].kind
    else {
        panic!("root is not Equivalent");
    };
    assert!(matches!(
        document.nodes()[left.0 as usize].kind,
        NodeKind::And { .. }
    ));
    assert!(matches!(
        document.nodes()[right.0 as usize].kind,
        NodeKind::Proposition { .. }
    ));

    // The selected semantic profile reaches the request, document, and report;
    // lowering itself is profile-independent.
    let closed = v2("p0 W[1,2] p1");
    let online = parse_clean_ascii_v2(
        "p0 W[1,2] p1",
        SemanticProfile::OnlinePrefixV1,
        ParseLimits::default(),
    );
    assert_eq!(online.semantic_profile, SemanticProfile::OnlinePrefixV1);
    let online_document = online.document.as_ref().unwrap();
    assert_eq!(
        online_document.semantic_profile(),
        SemanticProfile::OnlinePrefixV1
    );
    assert_eq!(
        online_document.nodes(),
        closed.document.as_ref().unwrap().nodes()
    );
    assert_eq!(online.lowerings, closed.lowerings);

    // Parenthesized operands widen the expression span; interval whitespace
    // widens the operator span.
    let report = v2("(p0 W[0,1] p1) M[ 2 , 3 ] (p2)");
    assert_eq!(report.lowerings[1].operator_span, span(15, 25));
    assert_eq!(report.lowerings[1].expression_span, span(0, 30));

    // Keyword boundaries match v1's U/R handling.
    assert!(v2("trueW[0,1]false").document.is_some());
    assert!(v2("p0M[0,1]p1").document.is_some());
}

// Trace: TC-042, FR-008-AC-3
#[test]
fn v2_refusals_are_typed_and_located() {
    let cases: &[(&str, DiagnosticCode, (u32, u32))] = &[
        ("p0 W p1", DiagnosticCode::MissingToken, (5, 7)),
        ("p0 M", DiagnosticCode::MissingToken, (4, 4)),
        ("p0 w[0,1] p1", DiagnosticCode::UnknownIdentifier, (3, 4)),
        ("p0 m[0,1] p1", DiagnosticCode::UnknownIdentifier, (3, 4)),
        (
            "p0 WeakUntil[0,1] p1",
            DiagnosticCode::UnknownIdentifier,
            (3, 12),
        ),
        ("p0 WU[0,1] p1", DiagnosticCode::UnknownIdentifier, (3, 5)),
        ("X[0,1] p0", DiagnosticCode::UnsupportedOperator, (0, 1)),
        ("X p0", DiagnosticCode::UnsupportedOperator, (0, 1)),
        ("Y p0", DiagnosticCode::UnsupportedOperator, (0, 1)),
        ("O[0,1] p0", DiagnosticCode::UnsupportedOperator, (0, 1)),
        ("H[0,1] p0", DiagnosticCode::UnsupportedOperator, (0, 1)),
        ("p0 S[0,1] p1", DiagnosticCode::UnsupportedOperator, (3, 4)),
        ("p0 T[0,1] p1", DiagnosticCode::UnsupportedOperator, (3, 4)),
        ("p0 W[01,2] p1", DiagnosticCode::NonCanonicalNumber, (5, 7)),
        (
            "p0 W[0,4294967296] p1",
            DiagnosticCode::IntegerOverflow,
            (7, 17),
        ),
        ("p0 M[3,2] p1", DiagnosticCode::InvalidInterval, (4, 9)),
        ("p0 W[0,) p1", DiagnosticCode::UnexpectedToken, (7, 8)),
        ("p0 W[0,] p1", DiagnosticCode::UnexpectedToken, (7, 8)),
        ("p0 W[0,5s] p1", DiagnosticCode::UnknownIdentifier, (8, 9)),
        (
            "p0 W[0.5,1] p1",
            DiagnosticCode::UnexpectedCharacter,
            (6, 7),
        ),
        (
            "p0 M[2026-01-01,1] p1",
            DiagnosticCode::UnexpectedCharacter,
            (9, 10),
        ),
        ("p0 W[0,inf] p1", DiagnosticCode::UnknownIdentifier, (7, 10)),
        ("trueS[0,1]p1", DiagnosticCode::UnsupportedOperator, (4, 5)),
        ("falseX[0,1]p1", DiagnosticCode::UnsupportedOperator, (5, 6)),
        ("trueX p0", DiagnosticCode::UnknownIdentifier, (0, 5)),
    ];
    for (source, code, (start, end)) in cases {
        let report = v2(source);
        assert_eq!(first_code(&report), (*code, span(*start, *end)), "{source}");
        assert!(report.document.is_none(), "{source}");
        assert!(report.lowerings.is_empty(), "{source}");
    }

    // The three-node charge is checked before anything is appended.
    let limited = |max_nodes| ParseLimits {
        max_nodes,
        ..ParseLimits::default()
    };
    let report = parse_clean_ascii_v2("p0 W[0,1] p1", PROFILE, limited(4));
    assert_eq!(
        first_code(&report),
        (DiagnosticCode::NodeLimit, span(0, 12))
    );
    assert_eq!(report.stats.nodes, 2);
    assert!(report.document.is_none() && report.lowerings.is_empty());
    assert!(parse_clean_ascii_v2("p0 W[0,1] p1", PROFILE, limited(5))
        .document
        .is_some());

    // Lowering charges one work unit per borrowed node, on top of the parse.
    let primitive = parse("p0 U[0,1] p1", PROFILE, ParseLimits::default());
    let derived = v2("p0 W[0,1] p1");
    assert_eq!(derived.stats.work, primitive.stats.work + 2);
    let report = parse_clean_ascii_v2(
        "p0 W[0,1] p1",
        PROFILE,
        ParseLimits {
            max_work: derived.stats.work - 1,
            ..ParseLimits::default()
        },
    );
    assert_eq!(
        first_code(&report),
        (DiagnosticCode::WorkLimit, span(0, 12))
    );
    assert!(report.document.is_none() && report.lowerings.is_empty());
}

// Trace: TC-044, FR-008-AC-5
#[test]
fn every_checked_v2_fuzz_seed_is_bounded() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let seeds = root.join("fuzz/corpus/clean_ascii_v2");
    let checksum = Command::new("sha256sum")
        .args(["--check", "SHA256SUMS"])
        .current_dir(&seeds)
        .output()
        .unwrap();
    assert!(checksum.status.success());
    assert!(root.join("fuzz/fuzz_targets/clean_ascii_v2.rs").is_file());
    // The build itself is proven by `make fuzz-build`; this pins that the
    // manifest declares the target it builds.
    let manifest = fs::read_to_string(root.join("fuzz/Cargo.toml")).unwrap();
    assert!(manifest.contains("name = \"clean_ascii_v2\""));
    assert!(manifest.contains("path = \"fuzz_targets/clean_ascii_v2.rs\""));

    let limits = ParseLimits {
        max_source_bytes: 4_096,
        max_tokens: 512,
        max_nodes: 256,
        max_depth: 64,
        max_diagnostics: 16,
        max_work: 16_384,
    };
    let mut paths = fs::read_dir(&seeds)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("txt"))
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 5);
    let mut lowered = 0;
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let report = parse_clean_ascii_v2(&source, PROFILE, limits);
        assert!(report.stats.nodes <= limits.max_nodes);
        assert!(report.stats.work <= limits.max_work);
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains(DIALECT_REVISION), "{}", path.display());
        if report.document.is_some() {
            lowered += usize::from(!report.lowerings.is_empty());
        } else {
            assert!(report.lowerings.is_empty());
            assert!(!report.diagnostics.is_empty());
        }
    }
    assert!(lowered > 0, "no seed exercises lowering");
}

// Trace: TC-043, FR-008-AC-4
#[test]
fn v2_canonical_text_beyond_limits_is_refused_only_for_resources() {
    // Chained left operands double in canonical text. The checked growth seed
    // formats within fuzz output limits, but v1 refuses the reparse through a
    // resource limit, which the fuzz target treats as a bounded outcome.
    let seed =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/clean_ascii_v2/growth.txt");
    let source = fs::read_to_string(seed).unwrap();
    let document = v2(&source).document.unwrap();
    let text = format_document(
        &document,
        FormatLimits {
            max_output_bytes: 16_384,
            max_work: 65_536,
        },
    )
    .text
    .unwrap();
    let reparsed = parse(&text, PROFILE, ParseLimits::default());
    assert!(reparsed.document.is_none());
    assert_eq!(reparsed.diagnostics[0].code, DiagnosticCode::NodeLimit);
}

// Trace: TC-045, FR-008-AC-6
#[test]
fn derived_report_is_deterministic_and_strict() {
    let source = "(p0 W[0,1] p1) M[2,3] !p2";
    let report = v2(source);
    assert_eq!(report, v2(source));
    let json = serde_json::to_string(&report).unwrap();
    assert_eq!(json, serde_json::to_string(&v2(source)).unwrap());
    let decoded: DerivedParseReport = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, report);

    let value = serde_json::to_value(&report).unwrap();
    let rejects = |mutate: &dyn Fn(&mut serde_json::Value)| {
        let mut mutated = value.clone();
        mutate(&mut mutated);
        serde_json::from_value::<DerivedParseReport>(mutated).is_err()
    };
    assert!(rejects(&|value| value["extra"] = true.into()));
    assert!(rejects(
        &|value| value["lowerings"][0]["extra"] = true.into()
    ));
    assert!(rejects(
        &|value| value["schema_version"] = "tl-parse.diagnostics/v1".into()
    ));
    assert!(rejects(
        &|value| value["dialect_revision"] = DIALECT_REVISION.into()
    ));
    assert!(rejects(
        &|value| value["operator_profile"] = "tl-syntax.past-operators/v1".into()
    ));
    assert!(rejects(&|value| value["lowerings"][0]["kind"] = "U".into()));
    assert!(rejects(&|value| value["document"] = serde_json::Value::Null));
    let diagnostic = serde_json::to_value(&v2("p0 W p1").diagnostics[0]).unwrap();
    assert!(rejects(
        &|value| value["diagnostics"] = vec![diagnostic.clone()].into()
    ));
    assert!(rejects(
        &|value| value["stats"]["diagnostics_truncated"] = true.into()
    ));
    for field in ["left", "right", "root"] {
        assert!(rejects(&|value| value["lowerings"][1][field] = 99.into()));
    }

    let changes = |mutate: &dyn Fn(&mut serde_json::Value)| {
        let mut mutated = value.clone();
        mutate(&mut mutated);
        serde_json::from_value::<DerivedParseReport>(mutated).unwrap() != report
    };
    assert!(changes(&|value| value["lowerings"][0]["kind"] = "M".into()));
    assert!(changes(&|value| value["lowerings"][0]["root"] = 9.into()));
    assert!(changes(&|value| value["lowerings"][1]["left"] = 0.into()));
    assert!(changes(
        &|value| value["lowerings"][0]["operator_span"]["end"] = 11.into()
    ));
    assert!(changes(
        &|value| value["lowerings"][1]["expression_span"]["start"] = 1.into()
    ));
    assert!(changes(&|value| value["tl_syntax_revision"] = "0".into()));

    let failed = v2("p0 W p1");
    let decoded: DerivedParseReport =
        serde_json::from_str(&serde_json::to_string(&failed).unwrap()).unwrap();
    assert_eq!(decoded, failed);
}
