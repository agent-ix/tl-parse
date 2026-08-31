use proptest::prelude::*;
use tl_parse::{format_document, parse, FormatLimits, ParseLimits};
use tl_syntax::{FormulaDocument, NodeKind, SemanticProfile};

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
