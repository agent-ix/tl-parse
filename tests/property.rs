use proptest::prelude::*;
use tl_parse::{format_document, parse, DiagnosticCode, FormatLimits, ParseLimits};
use tl_syntax::{FormulaDocument, NodeId, NodeKind, SemanticProfile};

fn source_strategy() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("false".to_owned()),
        Just("true".to_owned()),
        (0_u32..=8).prop_map(|value| format!("p{value}")),
    ];
    let recursive = leaf.prop_recursive(4, 48, 3, |inner| {
        prop_oneof![
            inner.clone().prop_map(|value| format!("!{value}")),
            (0_u32..=3, 0_u32..=3, inner.clone()).prop_map(|(start, width, value)| {
                let end = start + width;
                format!("F[{start},{end}]{value}")
            }),
            (0_u32..=3, 0_u32..=3, inner.clone()).prop_map(|(start, width, value)| {
                let end = start + width;
                format!("G[{start},{end}]{value}")
            }),
            (
                inner.clone(),
                prop_oneof![Just("&"), Just("|"), Just("->"), Just("<->")],
                inner.clone(),
            )
                .prop_map(|(left, operator, right)| format!("{left}{operator}{right}")),
            (
                inner.clone(),
                prop_oneof![Just("&"), Just("|"), Just("->"), Just("<->")],
                inner.clone(),
            )
                .prop_map(|(left, operator, right)| format!("({left}{operator}{right})")),
            (
                inner.clone(),
                prop_oneof![Just("U"), Just("R")],
                0_u32..=3,
                0_u32..=3,
                inner.clone(),
            )
                .prop_map(|(left, operator, start, width, right)| {
                    let end = start + width;
                    format!("{left}{operator}[{start},{end}]{right}")
                }),
            (
                inner.clone(),
                prop_oneof![Just("U"), Just("R")],
                0_u32..=3,
                0_u32..=3,
                inner.clone(),
            )
                .prop_map(|(left, operator, start, width, right)| {
                    let end = start + width;
                    format!("({left}{operator}[{start},{end}]{right})")
                }),
        ]
    });
    prop_oneof![
        recursive,
        (0_usize..=128).prop_map(|count| format!("{}p0", "!".repeat(count))),
        (1_usize..=256).prop_map(|count| {
            (0..count)
                .map(|index| format!("p{}", index % 9))
                .collect::<Vec<_>>()
                .join("&")
        }),
    ]
}

fn structural(document: &FormulaDocument) -> (SemanticProfile, u32, Vec<NodeKind>) {
    (
        document.semantic_profile(),
        document.root().0,
        document.nodes().iter().map(|node| node.kind).collect(),
    )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: TC-016, FR-004-AC-2, StR-002-VC-1
    #[test]
    fn generated_formulas_parse_format_parse_without_structural_drift(
        source in source_strategy(),
        online in any::<bool>(),
    ) {
        let profile = if online {
            SemanticProfile::OnlinePrefixV1
        } else {
            SemanticProfile::ClosedTraceV1
        };
        let first = parse(&source, profile, ParseLimits::default());
        prop_assert!(first.diagnostics.is_empty(), "{:?}: {:?}", source, first.diagnostics);
        let first_document = first.document.as_ref().unwrap();
        let formatted = format_document(first_document, FormatLimits::default());
        prop_assert!(formatted.error.is_none());
        let second = parse(formatted.text.as_ref().unwrap(), profile, ParseLimits::default());
        prop_assert!(second.diagnostics.is_empty(), "{:?}", second.diagnostics);
        prop_assert_eq!(structural(first_document), structural(second.document.as_ref().unwrap()));
    }
}

fn children(kind: &NodeKind) -> Vec<NodeId> {
    match *kind {
        NodeKind::Not { operand }
        | NodeKind::Future { operand, .. }
        | NodeKind::Globally { operand, .. } => vec![operand],
        NodeKind::And { left, right }
        | NodeKind::Or { left, right }
        | NodeKind::Implies { left, right }
        | NodeKind::Equivalent { left, right }
        | NodeKind::Until { left, right, .. }
        | NodeKind::Release { left, right, .. } => vec![left, right],
        _ => Vec::new(),
    }
}

fn node_at(document: &FormulaDocument, id: NodeId) -> &tl_syntax::Node {
    &document.nodes()[id.0 as usize]
}

fn operator_name(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::And { .. } => "&",
        NodeKind::Or { .. } => "|",
        NodeKind::Implies { .. } => "->",
        NodeKind::Equivalent { .. } => "<->",
        NodeKind::Until { .. } => "U",
        NodeKind::Release { .. } => "R",
        _ => "leaf",
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 96,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: TC-006, FR-002-AC-1
    #[test]
    fn generated_formula_spans_are_in_bounds_nested_and_name_their_source_text(
        source in source_strategy(),
        online in any::<bool>(),
    ) {
        let profile = if online {
            SemanticProfile::OnlinePrefixV1
        } else {
            SemanticProfile::ClosedTraceV1
        };
        let report = parse(&source, profile, ParseLimits::default());
        let document = report.document.as_ref().expect("generated formula parses");
        for node in document.nodes() {
            let span = node.span.expect("every parsed node carries a span");
            prop_assert!(span.start() <= span.end());
            prop_assert!((span.end() as usize) <= source.len());
            for child in children(&node.kind) {
                let inner = node_at(document, child).span.expect("child span");
                prop_assert!(span.start() <= inner.start() && inner.end() <= span.end());
            }
            if let NodeKind::Proposition { proposition } = node.kind {
                prop_assert_eq!(
                    &source[span.start() as usize..span.end() as usize],
                    format!("p{}", proposition.0)
                );
            }
        }
    }

    // Trace: TC-003, TC-008, FR-001-AC-2, FR-002-AC-3
    #[test]
    fn appended_malformed_tails_are_rejected_with_the_right_code_and_span(
        source in source_strategy(),
        online in any::<bool>(),
    ) {
        let profile = if online {
            SemanticProfile::OnlinePrefixV1
        } else {
            SemanticProfile::ClosedTraceV1
        };
        let end = source.len() as u32;
        // (appended tail, expected code, expected span start, expected span end)
        let cases = [
            ("@", DiagnosticCode::UnexpectedCharacter, end, end + 1),
            (")", DiagnosticCode::TrailingInput, end, end + 1),
            (" p0", DiagnosticCode::TrailingInput, end + 1, end + 3),
        ];
        for (tail, code, start, stop) in cases {
            let mutated = format!("{source}{tail}");
            let report = parse(&mutated, profile, ParseLimits::default());
            prop_assert!(report.document.is_none(), "{:?} must not yield a document", mutated);
            prop_assert!(
                report.diagnostics.iter().any(|item| {
                    item.code == code && item.span.start() == start && item.span.end() == stop
                }),
                "{:?}: expected {:?} at {}..{}, got {:?}",
                mutated, code, start, stop, report.diagnostics
            );
        }
        let dangling = format!("{source}&");
        let report = parse(&dangling, profile, ParseLimits::default());
        prop_assert!(report.document.is_none());
        prop_assert!(report.diagnostics.iter().any(|item| {
            item.code == DiagnosticCode::UnexpectedToken && item.span.start() == end + 1
        }), "{:?}", report.diagnostics);
    }

    // Trace: TC-002, FR-001-AC-1
    #[test]
    fn chained_binary_operators_associate_as_declared(
        count in 3_usize..=7,
        choice in 0_usize..6,
    ) {
        let (operator, spelled, right_associative) = [
            ("&", "&", false),
            ("|", "|", false),
            ("<->", "<->", false),
            ("->", "->", true),
            ("U", "U[0,1]", false),
            ("R", "R[0,1]", false),
        ][choice];
        let source = (0..count).map(|i| format!("p{i}")).collect::<Vec<_>>().join(spelled);
        let report = parse(&source, SemanticProfile::ClosedTraceV1, ParseLimits::default());
        let document = report.document.as_ref().expect("chain parses");
        // Walk the operator spine: the deep side is another operator, the shallow
        // side is the next proposition in source order.
        let mut node = node_at(document, document.root());
        let mut order = Vec::new();
        loop {
            prop_assert_eq!(operator_name(&node.kind), operator);
            let kids = children(&node.kind);
            let (deep, shallow) = if right_associative {
                (kids[1], kids[0])
            } else {
                (kids[0], kids[1])
            };
            let leaf = node_at(document, shallow);
            let NodeKind::Proposition { proposition } = leaf.kind else {
                return Err(TestCaseError::fail("shallow operand must be a proposition"));
            };
            order.push(proposition.0);
            let next = node_at(document, deep);
            if let NodeKind::Proposition { proposition } = next.kind {
                order.push(proposition.0);
                break;
            }
            node = next;
        }
        // Spine visits operands from the outside in: the last operands first for
        // left association, the first operands first for right association.
        let expected: Vec<u32> = if right_associative {
            (0..count as u32).collect()
        } else {
            (0..count as u32).rev().collect()
        };
        prop_assert_eq!(order, expected);
    }

    // Trace: TC-002, FR-001-AC-1
    #[test]
    fn adjacent_precedence_levels_bind_the_tighter_operator_first(
        level in 0_usize..4,
    ) {
        // Loosest to tightest.
        let ladder = ["<->", "->", "|", "&", "U[0,1]"];
        let (loose, tight) = (ladder[level], ladder[level + 1]);
        let tight_name = operator_name_for(tight);
        let loose_name = operator_name_for(loose);
        // loose then tight: the tight operator groups the right two operands.
        let right = parse(&format!("p0{loose}p1{tight}p2"), SemanticProfile::ClosedTraceV1, ParseLimits::default());
        let document = right.document.as_ref().expect("parses");
        let root = node_at(document, document.root());
        prop_assert_eq!(operator_name(&root.kind), loose_name);
        let kids = children(&root.kind);
        prop_assert_eq!(operator_name(&node_at(document, kids[1]).kind), tight_name);
        // tight then loose: the tight operator groups the left two operands.
        let left = parse(&format!("p0{tight}p1{loose}p2"), SemanticProfile::ClosedTraceV1, ParseLimits::default());
        let document = left.document.as_ref().expect("parses");
        let root = node_at(document, document.root());
        prop_assert_eq!(operator_name(&root.kind), loose_name);
        let kids = children(&root.kind);
        prop_assert_eq!(operator_name(&node_at(document, kids[0]).kind), tight_name);
    }
}

fn operator_name_for(spelled: &str) -> &'static str {
    match spelled {
        "<->" => "<->",
        "->" => "->",
        "|" => "|",
        "&" => "&",
        _ => "U",
    }
}
