use tl_parse::{format_document, parse, FormatErrorCode, FormatLimits, ParseLimits};
use tl_syntax::{FormulaDocument, Node, NodeId, NodeKind, SemanticProfile};

fn parse_closed(source: &str) -> FormulaDocument {
    let report = parse(
        source,
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    report.document.expect("valid source")
}

fn canonical(source: &str) -> String {
    format_document(&parse_closed(source), FormatLimits::default())
        .text
        .expect("format succeeds")
}

// Trace: TC-014, FR-004-AC-1
#[test]
fn every_operator_has_one_exact_canonical_rendering() {
    let cases = [
        ("false", "false"),
        ("true", "true"),
        ("p4294967295", "p4294967295"),
        ("!p0", "!p0"),
        ("p0 & p1", "p0&p1"),
        ("p0 | p1", "p0|p1"),
        ("p0 -> p1", "p0->p1"),
        ("p0 <-> p1", "p0<->p1"),
        ("F[0,2]p0", "F[0,2]p0"),
        ("G[1,3]p0", "G[1,3]p0"),
        ("p0 U[1,2] p1", "p0U[1,2]p1"),
        ("p0 R[2,4] p1", "p0R[2,4]p1"),
        ("true U[1,2] false", "trueU[1,2]false"),
    ];
    for (source, expected) in cases {
        assert_eq!(canonical(source), expected, "{source}");
    }
}

// Trace: TC-015, FR-004-AC-1, NFR-001-AC-1
#[test]
fn canonical_format_is_parseable_and_idempotent() {
    let first = canonical("!(p0 U[1,2] true) <-> G[0,3](p1 -> false)");
    let second = canonical(&first);
    assert_eq!(first, second);

    for (source, expected) in [
        ("!(p0&p1)", "!(p0&p1)"),
        ("(p0|p1)&p2", "(p0|p1)&p2"),
        ("p0&(p1&p2)", "p0&(p1&p2)"),
        ("(p0->p1)->p2", "(p0->p1)->p2"),
        ("p0->(p1->p2)", "p0->p1->p2"),
        ("p0<->(p1<->p2)", "p0<->(p1<->p2)"),
        ("p0U[0,1](p1R[0,1]p2)", "p0U[0,1](p1R[0,1]p2)"),
    ] {
        assert_eq!(canonical(source), expected, "{source}");
    }
}

// Trace: TC-016, FR-004-AC-2, StR-002-VC-1
#[test]
fn accepted_depth_and_chain_boundaries_reparse_under_the_same_limits() {
    for source in [
        format!("{}p0", "!".repeat(128)),
        (0..256)
            .map(|index| format!("p{}", index % 9))
            .collect::<Vec<_>>()
            .join("&"),
    ] {
        let first = parse_closed(&source);
        let text = format_document(&first, FormatLimits::default())
            .text
            .expect("accepted input formats");
        let second = parse_closed(&text);
        let kinds = |document: &FormulaDocument| {
            document
                .nodes()
                .iter()
                .map(|node| node.kind)
                .collect::<Vec<_>>()
        };
        assert_eq!(first.root(), second.root());
        assert_eq!(kinds(&first), kinds(&second));
    }
}

// Trace: TC-017, FR-004-AC-3, NFR-001-AC-2
#[test]
fn deep_shared_graphs_and_formatter_limits_are_bounded() {
    let mut nodes = vec![Node::new(NodeKind::True)];
    for operand in 0..1_000 {
        nodes.push(Node::new(NodeKind::Not {
            operand: NodeId(operand),
        }));
    }
    let deep = FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(1_000), nodes).unwrap();
    let report = format_document(&deep, FormatLimits::default());
    assert!(report.error.is_none());
    assert_eq!(report.stats.nodes, 1_001);

    let shared = FormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(1),
        vec![
            Node::new(NodeKind::Proposition {
                proposition: tl_syntax::PropositionId(3),
            }),
            Node::new(NodeKind::And {
                left: NodeId(0),
                right: NodeId(0),
            }),
        ],
    )
    .unwrap();
    let report = format_document(&shared, FormatLimits::default());
    assert_eq!(report.text.as_deref(), Some("p3&p3"));
    assert_eq!(report.stats.nodes, 2);

    for (limits, expected) in [
        (
            FormatLimits {
                max_output_bytes: 4,
                ..FormatLimits::default()
            },
            FormatErrorCode::OutputLimit,
        ),
        (
            FormatLimits {
                max_work: 1,
                ..FormatLimits::default()
            },
            FormatErrorCode::WorkLimit,
        ),
    ] {
        let report = format_document(&shared, limits);
        assert!(report.text.is_none());
        assert_eq!(report.stats.output_bytes, 0);
        let json = serde_json::to_string(&report).unwrap();
        let decoded: tl_parse::FormatReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);
        let error = report.error.unwrap();
        assert_eq!(error.code, expected);
        assert!(error.to_string().contains(&expected.to_string()));
    }
}
