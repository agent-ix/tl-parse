use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use tl_parse::tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind, NodeId, PropositionId, SemanticProfile, SourceSpan, TemporalInterval,
    UnboundedInterval,
};
use tl_parse::{
    format_clean_ascii_v4, parse_clean_ascii_v4, DiagnosticCode, FormatErrorCode, FormatLimits,
    InfiniteDisposition, InfiniteParseReport, ParseArtifactLimits, ParseLimits,
    StrictParseArtifactReadError, DIALECT_V4_REVISION,
};

const PROFILE: SemanticProfile = SemanticProfile::InfiniteTraceV1;

fn parse(source: &str) -> tl_parse::InfiniteParseReport {
    parse_clean_ascii_v4(source, PROFILE, "event_position", ParseLimits::default())
}

fn strict_read(
    report: &InfiniteParseReport,
) -> Result<InfiniteParseReport, tl_parse::StrictParseArtifactReadError> {
    InfiniteParseReport::from_json_bytes(
        &serde_json::to_vec(report).unwrap(),
        ParseArtifactLimits::default(),
    )
}

// Trace: TC-062, FR-016-AC-2, TC-064, FR-017-AC-2
#[test]
fn v4_report_strict_reader_rejects_identity_and_locus_mutations() {
    let report = parse("fair {F[0,)p0}: G[1,)p1");
    assert_eq!(strict_read(&report).unwrap(), report);

    let mut foreign_clock = report.clone();
    foreign_clock.clock = "wall_clock".to_owned();
    assert!(strict_read(&foreign_clock).is_err());

    let mut outside = report.clone();
    outside.premise_spans[0] = SourceSpan::new(0, 1000).unwrap();
    assert!(strict_read(&outside).is_err());

    let mut stale_owner = report.clone();
    stale_owner.tl_syntax_revision = "stale-owner".to_owned();
    assert!(strict_read(&stale_owner).is_err());

    let mut missing_graph = report.clone();
    missing_graph.document = None;
    missing_graph.fairness = None;
    assert!(strict_read(&missing_graph).is_err());

    let mut missing_fairness = report.clone();
    missing_fairness.fairness = None;
    assert!(strict_read(&missing_fairness).is_err());

    let identity = report.fairness.as_ref().unwrap().graph_identity();
    let original = String::from_utf8(serde_json::to_vec(&report).unwrap()).unwrap();
    let foreign = original.replacen(identity, "foreign-graph", 1);
    assert_ne!(foreign, original);
    assert!(InfiniteParseReport::from_json_bytes(
        foreign.as_bytes(),
        ParseArtifactLimits::default()
    )
    .is_err());
}

// Each counter and limit is checked independently; one bad counter cannot hide
// behind another valid field in the report wire.
// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_report_counter_limits_accept_exact_boundaries_and_refuse_one_over() {
    let refused = parse("!");
    assert!(refused.document.is_none());
    assert!(!refused.diagnostics.is_empty());
    assert_eq!(strict_read(&refused).unwrap(), refused);

    let mut tokens = refused.clone();
    tokens.limits.max_tokens = tokens.stats.tokens;
    assert!(strict_read(&tokens).is_ok());
    tokens.stats.tokens += 1;
    assert!(strict_read(&tokens).is_err());

    let mut nodes = refused.clone();
    nodes.limits.max_nodes = nodes.stats.nodes;
    assert!(strict_read(&nodes).is_ok());
    nodes.stats.nodes += 1;
    assert!(strict_read(&nodes).is_err());

    let mut work = refused.clone();
    work.limits.max_work = work.stats.work;
    assert!(strict_read(&work).is_ok());
    work.stats.work += 1;
    assert!(strict_read(&work).is_err());

    let mut diagnostics = refused.clone();
    diagnostics.limits.max_diagnostics = diagnostics.stats.diagnostics;
    assert!(strict_read(&diagnostics).is_ok());
    diagnostics.limits.max_diagnostics = 0;
    assert!(strict_read(&diagnostics).is_err());

    let mut unclamped = refused;
    unclamped.limits.max_tokens = usize::MAX;
    assert!(strict_read(&unclamped).is_err());
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_strict_reader_rejects_conflicting_success_state_and_outside_loci() {
    let baseline = parse("fair {F[0,)p0}: G[1,)p1");
    assert!(baseline.document.is_some());
    assert_eq!(strict_read(&baseline).unwrap(), baseline);

    let mut wrong_profile = baseline.clone();
    wrong_profile.semantic_profile = SemanticProfile::ClosedTraceV1;
    assert!(strict_read(&wrong_profile).is_err());

    let mut extra_diagnostic = baseline.clone();
    extra_diagnostic.diagnostics = parse("!").diagnostics;
    extra_diagnostic.stats.diagnostics = extra_diagnostic.diagnostics.len();
    assert!(strict_read(&extra_diagnostic).is_err());

    let mut truncated = baseline.clone();
    truncated.stats.diagnostics_truncated = true;
    assert!(strict_read(&truncated).is_err());

    let mut wrong_node_count = baseline.clone();
    wrong_node_count.stats.nodes += 1;
    assert!(strict_read(&wrong_node_count).is_err());

    let mut outside_interval = baseline.clone();
    outside_interval.interval_spans[0] = SourceSpan::new(0, 1000).unwrap();
    assert!(strict_read(&outside_interval).is_err());

    let mut source_limit = baseline.clone();
    source_limit.limits.max_source_bytes = source_limit.stats.source_bytes - 1;
    assert!(strict_read(&source_limit).is_err());

    let mut depth_limit = baseline.clone();
    depth_limit.limits.max_depth = depth_limit.stats.max_depth - 1;
    assert!(strict_read(&depth_limit).is_err());

    let foreign = parse("fair {O[0,)p2}: p3");
    let mut foreign_fairness = baseline.clone();
    foreign_fairness.fairness = foreign.fairness.clone();
    assert!(strict_read(&foreign_fairness).is_err());

    let mut missing_graph = baseline;
    missing_graph.document = None;
    missing_graph.diagnostics = parse("!").diagnostics;
    missing_graph.stats.diagnostics = missing_graph.diagnostics.len();
    assert!(strict_read(&missing_graph).is_err());
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_strict_reader_checks_canonical_bytes_diagnostic_loci_and_owner_nodes() {
    let successful = parse("p0");
    assert_eq!(strict_read(&successful).unwrap(), successful);
    let mut noncanonical = serde_json::to_vec(&successful).unwrap();
    noncanonical.push(b' ');
    assert!(matches!(
        InfiniteParseReport::from_json_bytes(&noncanonical, ParseArtifactLimits::default()),
        Err(StrictParseArtifactReadError::NonCanonicalDocument)
    ));

    let mut wrong_count = successful.clone();
    wrong_count.stats.diagnostics = 1;
    assert!(strict_read(&wrong_count).is_err());

    let mut outside_node = successful;
    outside_node.document = Some(
        InfiniteFormulaDocument::new(
            PROFILE,
            InfiniteClock::EventPosition,
            NodeId(0),
            vec![InfiniteNode::with_span(
                InfiniteNodeKind::Proposition {
                    proposition: PropositionId(0),
                },
                SourceSpan::new(0, 1000).unwrap(),
            )],
        )
        .unwrap(),
    );
    assert!(strict_read(&outside_node).is_err());

    let mut outside_diagnostic = parse("!");
    outside_diagnostic.diagnostics[0].span = SourceSpan::new(0, 1000).unwrap();
    assert!(strict_read(&outside_diagnostic).is_err());
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_fairness_report_rejects_unbound_roots_and_missing_loci() {
    let baseline = parse("fair {F[0,)p0}: p1");
    assert_eq!(strict_read(&baseline).unwrap(), baseline);

    let mut missing_locus = baseline.clone();
    missing_locus.premise_spans.clear();
    assert!(strict_read(&missing_locus).is_err());

    let mut fairness_wire = serde_json::to_value(baseline.fairness.as_ref().unwrap()).unwrap();
    fairness_wire["roots"][0] = serde_json::json!(999);
    let invalid_roots: FairnessPremisesDocument = serde_json::from_value(fairness_wire).unwrap();
    let mut foreign_root = baseline.clone();
    foreign_root.fairness = Some(invalid_roots.clone());
    assert!(strict_read(&foreign_root).is_err());

    let formatted = format_clean_ascii_v4(
        baseline.document.as_ref().unwrap(),
        Some(&invalid_roots),
        FormatLimits::default(),
    );
    assert!(formatted.text.is_none());
    assert_eq!(formatted.error.unwrap().code, FormatErrorCode::InvalidGraph);
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn v4_disposition_distinguishes_invalid_identity_from_truncated_resources() {
    let valid = parse("p0");
    assert_eq!(valid.disposition(), InfiniteDisposition::NoTemporalVerdict);
    let mut wrong_clock = valid;
    wrong_clock.clock = "wall_clock".to_owned();
    assert_eq!(wrong_clock.disposition(), InfiniteDisposition::Unsupported);

    let truncated = parse_clean_ascii_v4(
        "!",
        PROFILE,
        "event_position",
        ParseLimits {
            max_diagnostics: 0,
            ..ParseLimits::default()
        },
    );
    assert!(truncated.document.is_none());
    assert!(truncated.diagnostics.is_empty());
    assert!(truncated.stats.diagnostics_truncated);
    assert_eq!(strict_read(&truncated).unwrap(), truncated);
    assert_eq!(
        truncated.disposition(),
        InfiniteDisposition::ResourceIncomplete
    );
}

// Trace: TC-058, FR-015-AC-1
#[test]
fn admits_all_infinite_future_and_past_forms() {
    for source in [
        "F[0,)p0",
        "G[1,2]p0",
        "p0 U[0,) p1",
        "p0 R[2,4] p1",
        "O[0,)p0",
        "H[1,2]p0",
        "p0 S[0,) p1",
        "p0 T[2,4] p1",
        "Yp0",
        "YF[0,)p0",
        "YG[1,2]p0",
        "p0 W[0,) p1",
        "p0 M[1,3] p1",
        "!p0",
    ] {
        let result = parse(source);
        assert!(
            result.document.is_some(),
            "{source}: {:?}",
            result.diagnostics
        );
        assert_eq!(result.disposition(), InfiniteDisposition::NoTemporalVerdict);
        let canonical = format_clean_ascii_v4(
            result.document.as_ref().unwrap(),
            None,
            FormatLimits::default(),
        )
        .text
        .unwrap();
        assert!(
            parse(&canonical).document.is_some(),
            "canonical {canonical}"
        );
    }
    assert_eq!(DIALECT_V4_REVISION, "tl-parse.clean-ascii/v4");
}

// Trace: TC-059, FR-015-AC-2
#[test]
fn fairness_roots_are_bound_to_one_graph_and_duplicates_refuse() {
    let report = parse("fair { F[0,)p0; G[0,)p1; } : (p0 U[0,) p1)");
    let document = report.document.as_ref().expect("valid graph");
    let fairness = report.fairness.as_ref().expect("ordered premises");
    assert_eq!(fairness.roots().len(), 2);
    assert_eq!(
        fairness.graph_identity(),
        document.content_identity().unwrap()
    );
    assert_eq!(report.premise_spans.len(), 2);
    assert!(fairness
        .roots()
        .iter()
        .all(|id| document.formula().node(*id).is_some()));

    let duplicate = parse("fair { F[0,)p0; F[0,)p0; } : p1");
    assert!(duplicate.document.is_none());
    assert_eq!(
        duplicate.diagnostics[0].code,
        DiagnosticCode::DuplicateFairnessPremise
    );
    assert!(parse("fair {} : p0").fairness.is_none());
}

// Trace: TC-060, FR-015-AC-3, TC-062, FR-016-AC-2
#[test]
fn mismatched_profile_and_clock_are_typed_refusals() {
    let profile = parse_clean_ascii_v4(
        "F[0,)p0",
        SemanticProfile::ClosedTraceV1,
        "event_position",
        ParseLimits::default(),
    );
    assert_eq!(
        profile.diagnostics[0].code,
        DiagnosticCode::InfiniteProfileMismatch
    );
    assert_eq!(profile.disposition(), InfiniteDisposition::Unsupported);
    let clock = parse_clean_ascii_v4("F[0,)p0", PROFILE, "wall_clock", ParseLimits::default());
    assert_eq!(
        clock.diagnostics[0].code,
        DiagnosticCode::InfiniteClockMismatch
    );
    assert!(clock.document.is_none());

    let no_diagnostics = parse_clean_ascii_v4(
        "p0",
        SemanticProfile::ClosedTraceV1,
        "event_position",
        ParseLimits {
            max_diagnostics: 0,
            ..ParseLimits::default()
        },
    );
    assert!(no_diagnostics.diagnostics.is_empty());
    assert!(no_diagnostics.stats.diagnostics_truncated);
    assert_eq!(
        no_diagnostics.disposition(),
        InfiniteDisposition::Unsupported
    );

    for source in ["F[0,)p0", "fair {G[0,)p0}: p1"] {
        let v1 = tl_parse::parse(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert!(v1.document.is_none(), "v1 admitted {source}");
        let v2 = tl_parse::parse_clean_ascii_v2(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert!(v2.document.is_none(), "v2 admitted {source}");
        let v3 = tl_parse::parse_clean_ascii_v3(source, ParseLimits::default());
        assert!(v3.document.is_none(), "v3 admitted {source}");
    }
}

// Trace: TC-061, FR-016-AC-1, TC-063, FR-017-AC-1
#[test]
fn interval_and_premise_spans_are_bytes_and_text_reaches_fixed_point() {
    let source = "fair { F[0,)p0; O[1,2]p1; } : (p0 U[3,) p1)";
    let first = parse(source);
    let document = first.document.as_ref().unwrap();
    let interval = first.interval_spans[0];
    assert_eq!(
        &source[interval.start() as usize..interval.end() as usize],
        "[0,)"
    );
    let premise = first.premise_spans[1];
    assert_eq!(
        &source[premise.start() as usize..premise.end() as usize],
        "O[1,2]p1"
    );
    let formatted =
        format_clean_ascii_v4(document, first.fairness.as_ref(), FormatLimits::default());
    let text = formatted.text.expect("canonical text");
    let second = parse(&text);
    assert!(
        second.document.is_some(),
        "{text}: {:?}",
        second.diagnostics
    );
    assert_eq!(
        document.content_identity().unwrap(),
        second
            .document
            .as_ref()
            .unwrap()
            .content_identity()
            .unwrap()
    );
    assert_eq!(
        first.fairness.as_ref().unwrap().roots(),
        second.fairness.as_ref().unwrap().roots()
    );
    let again = format_clean_ascii_v4(
        second.document.as_ref().unwrap(),
        second.fairness.as_ref(),
        FormatLimits::default(),
    );
    assert_eq!(again.text.as_deref(), Some(text.as_str()));
}

// W/M lowering shares an operand in the owner graph, so formatting the lowered
// primitives separately changes its identity.
// Trace: TC-063, FR-017-AC-1
#[test]
fn derived_future_text_round_trips_the_exact_lowered_graph() {
    for source in [" p0 W[0,) p1 ", "p0 M[1,3] p1"] {
        let first = parse(source);
        let document = first.document.as_ref().unwrap();
        let formatted = format_clean_ascii_v4(document, None, FormatLimits::default());
        let text = formatted.text.expect("derived form must format");
        let second = parse(&text);
        assert_eq!(
            document.content_identity().unwrap(),
            second
                .document
                .as_ref()
                .unwrap()
                .content_identity()
                .unwrap(),
            "{source} -> {text}"
        );
        let again = format_clean_ascii_v4(
            second.document.as_ref().unwrap(),
            None,
            FormatLimits::default(),
        );
        assert_eq!(again.text.as_deref(), Some(text.as_str()));
    }
}

// Trace: TC-063, FR-017-AC-1
#[test]
fn ordinary_boolean_roots_do_not_masquerade_as_derived_temporal_forms() {
    for source in [
        "p0 | p1",
        "p0 & p1",
        "p0 & p1 & p2",
        "(p0 U[0,)p1) | G[0,)p2",
        "(p0 U[0,)p1) & p2",
        "(p0 R[0,)p1) | p2",
    ] {
        let first = parse(source);
        let document = first.document.as_ref().expect("valid boolean graph");
        let text = format_clean_ascii_v4(document, None, FormatLimits::default())
            .text
            .expect("boolean graph has canonical text");
        let second = parse(&text);
        assert_eq!(
            document.content_identity().unwrap(),
            second
                .document
                .as_ref()
                .unwrap()
                .content_identity()
                .unwrap(),
            "{source} -> {text}"
        );
    }
}

// A near match to W or M has valid owner topology but cannot be abbreviated
// without changing its interval. The formatter must refuse any text that
// would silently change the graph.
// Trace: TC-063, FR-017-AC-1
#[test]
fn derived_temporal_near_matches_do_not_emit_a_false_canonical_text() {
    let interval = TemporalInterval::Unbounded(UnboundedInterval::new(0));
    let other_interval = TemporalInterval::Unbounded(UnboundedInterval::new(1));
    for (weak, wrong_second_kind) in [(true, false), (false, false), (true, true), (false, true)] {
        let first = if weak {
            InfiniteNodeKind::Until {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            }
        } else {
            InfiniteNodeKind::Release {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            }
        };
        let second = if weak != wrong_second_kind {
            InfiniteNodeKind::Globally {
                interval: other_interval,
                operand: NodeId(0),
            }
        } else {
            InfiniteNodeKind::Future {
                interval: other_interval,
                operand: NodeId(0),
            }
        };
        let root = if weak {
            InfiniteNodeKind::Or {
                left: NodeId(2),
                right: NodeId(3),
            }
        } else {
            InfiniteNodeKind::And {
                left: NodeId(2),
                right: NodeId(3),
            }
        };
        let document = InfiniteFormulaDocument::new(
            PROFILE,
            InfiniteClock::EventPosition,
            NodeId(4),
            vec![
                InfiniteNode::new(InfiniteNodeKind::Proposition {
                    proposition: PropositionId(0),
                }),
                InfiniteNode::new(InfiniteNodeKind::Proposition {
                    proposition: PropositionId(1),
                }),
                InfiniteNode::new(first),
                InfiniteNode::new(second),
                InfiniteNode::new(root),
            ],
        )
        .unwrap();
        let result = format_clean_ascii_v4(&document, None, FormatLimits::default());
        assert!(result.text.is_none(), "weak={weak}");
        assert_eq!(
            result.error.unwrap().code,
            FormatErrorCode::UnrepresentableGraph,
            "weak={weak}, wrong_second_kind={wrong_second_kind}"
        );
    }
}

// W and M abbreviations require the two Boolean children and their parent to
// be adjacent in the owner's postorder. A valid graph with an inserted node
// at either boundary cannot be rendered as the same graph by the text dialect.
// Trace: TC-063, FR-017-AC-1
#[test]
fn nonadjacent_derived_temporal_nodes_refuse_canonical_text() {
    let interval = TemporalInterval::Unbounded(UnboundedInterval::new(0));
    for (weak, gap_after_first) in [(true, true), (false, true), (true, false), (false, false)] {
        let first = if weak {
            InfiniteNodeKind::Until {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            }
        } else {
            InfiniteNodeKind::Release {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            }
        };
        let second = if weak {
            InfiniteNodeKind::Globally {
                interval,
                operand: NodeId(0),
            }
        } else {
            InfiniteNodeKind::Future {
                interval,
                operand: NodeId(0),
            }
        };
        let filler = InfiniteNode::new(InfiniteNodeKind::Not { operand: NodeId(1) });
        let (second_id, remaining) = if gap_after_first {
            (NodeId(4), vec![filler, InfiniteNode::new(second)])
        } else {
            (NodeId(3), vec![InfiniteNode::new(second), filler])
        };
        let root = if weak {
            InfiniteNodeKind::Or {
                left: NodeId(2),
                right: second_id,
            }
        } else {
            InfiniteNodeKind::And {
                left: NodeId(2),
                right: second_id,
            }
        };
        let mut nodes = vec![
            InfiniteNode::new(InfiniteNodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            InfiniteNode::new(InfiniteNodeKind::Proposition {
                proposition: PropositionId(1),
            }),
            InfiniteNode::new(first),
        ];
        nodes.extend(remaining);
        nodes.push(InfiniteNode::new(root));
        let document =
            InfiniteFormulaDocument::new(PROFILE, InfiniteClock::EventPosition, NodeId(5), nodes)
                .expect("nonadjacent owner topology is valid");
        let report = format_clean_ascii_v4(&document, None, FormatLimits::default());
        assert!(
            report.text.is_none(),
            "weak={weak}, gap_after_first={gap_after_first}"
        );
        assert_eq!(
            report.error.expect("typed refusal").code,
            FormatErrorCode::UnrepresentableGraph,
            "weak={weak}, gap_after_first={gap_after_first}"
        );
    }
}

// Trace: TC-063, FR-017-AC-1
#[test]
fn formatter_refuses_fairness_from_a_different_owner_graph() {
    let owner = parse("fair {F[0,)p0}: G[0,)p1");
    let foreign = parse("fair {O[0,)p2}: G[0,)p1");
    let report = format_clean_ascii_v4(
        owner.document.as_ref().unwrap(),
        foreign.fairness.as_ref(),
        FormatLimits::default(),
    );
    assert!(report.text.is_none());
    assert_eq!(report.error.unwrap().code, FormatErrorCode::InvalidGraph);
}

// Trace: TC-063, FR-017-AC-1
#[test]
fn empty_fairness_document_formats_as_the_unqualified_claim() {
    let report = parse("p0");
    let document = report.document.as_ref().unwrap();
    let empty = FairnessPremisesDocument::new(
        document,
        document.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        Vec::new(),
    )
    .unwrap();
    let plain = format_clean_ascii_v4(document, None, FormatLimits::default());
    let qualified = format_clean_ascii_v4(document, Some(&empty), FormatLimits::default());
    assert_eq!(qualified.text, plain.text);
    assert_eq!(qualified.text.as_deref(), Some("p0"));
}

// Loci are retained for diagnostics but have no effect on the semantic graph
// identity used by fairness and round trips.
// Trace: TC-063, FR-017-AC-1
#[test]
fn source_spacing_does_not_change_v4_graph_identity() {
    let compact = parse("p0 U[1,) p1");
    let spaced = parse("  p0   U[1,)   p1  ");
    let compact_graph = compact.document.as_ref().unwrap();
    let spaced_graph = spaced.document.as_ref().unwrap();
    assert_ne!(compact_graph.nodes()[0].span, spaced_graph.nodes()[0].span);
    assert_eq!(
        compact_graph.content_identity().unwrap(),
        spaced_graph.content_identity().unwrap()
    );
}

// The text dialect has no general DAG reference syntax; a valid shared graph
// must receive a typed formatting refusal.
// Trace: TC-063, FR-017-AC-1
#[test]
fn unrepresentable_shared_graph_refuses_before_emitting_text() {
    let graph = InfiniteFormulaDocument::new(
        PROFILE,
        InfiniteClock::EventPosition,
        NodeId(2),
        vec![
            InfiniteNode::new(InfiniteNodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            InfiniteNode::new(InfiniteNodeKind::Not { operand: NodeId(0) }),
            InfiniteNode::new(InfiniteNodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    )
    .unwrap();
    let report = format_clean_ascii_v4(&graph, None, FormatLimits::default());
    assert!(report.text.is_none());
    assert_eq!(
        report.error.unwrap().code,
        FormatErrorCode::UnrepresentableGraph
    );
}

// The owner permits graphs deeper than canonical text's 256-node rendering
// boundary. Formatting must refuse that valid graph without emitting a prefix.
// Trace: TC-063, FR-017-AC-1
#[test]
fn formatter_refuses_an_owner_graph_beyond_its_text_depth() {
    let mut nodes = vec![InfiniteNode::new(InfiniteNodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    for index in 0..256 {
        nodes.push(InfiniteNode::new(InfiniteNodeKind::Not {
            operand: NodeId(index),
        }));
    }
    let document =
        InfiniteFormulaDocument::new(PROFILE, InfiniteClock::EventPosition, NodeId(256), nodes)
            .expect("257-deep graph is within owner limits");

    let report = format_clean_ascii_v4(&document, None, FormatLimits::default());
    assert!(report.text.is_none());
    assert_eq!(report.error.unwrap().code, FormatErrorCode::InvalidGraph);
}

// The owner can carry a disconnected node, while canonical text must account
// for the exact node sequence before it claims to represent the graph.
// Trace: TC-063, FR-017-AC-1
#[test]
fn formatter_refuses_a_disconnected_owner_node() {
    let document = InfiniteFormulaDocument::new(
        PROFILE,
        InfiniteClock::EventPosition,
        NodeId(0),
        vec![
            InfiniteNode::new(InfiniteNodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            InfiniteNode::new(InfiniteNodeKind::Proposition {
                proposition: PropositionId(1),
            }),
        ],
    )
    .expect("disconnected owner node is a valid graph");

    let report = format_clean_ascii_v4(&document, None, FormatLimits::default());
    assert!(report.text.is_none());
    assert_eq!(
        report.error.unwrap().code,
        FormatErrorCode::UnrepresentableGraph
    );
}

// Trace: TC-061, FR-016-AC-1, TC-062, FR-016-AC-2
#[test]
fn malformed_loci_use_utf8_byte_offsets_and_stable_codes() {
    let utf8 = parse("🙂 @ F[0,)p0");
    assert!(utf8.document.is_none());
    assert_eq!(
        utf8.diagnostics[0].code,
        DiagnosticCode::UnexpectedCharacter
    );
    assert_eq!(
        (
            utf8.diagnostics[0].span.start(),
            utf8.diagnostics[0].span.end()
        ),
        (0, 4)
    );
    assert_eq!(
        (
            utf8.diagnostics[1].span.start(),
            utf8.diagnostics[1].span.end()
        ),
        (5, 6)
    );

    for (source, code, locus) in [
        ("F[x,)p0", DiagnosticCode::UnknownIdentifier, (2, 3)),
        ("F[0 1]p0", DiagnosticCode::MissingToken, (4, 5)),
        ("F[0,)p0)", DiagnosticCode::TrailingInput, (7, 8)),
        ("fair {p0 p1}: p2", DiagnosticCode::MissingToken, (9, 11)),
    ] {
        let result = parse(source);
        assert!(result.document.is_none(), "{source}");
        assert_eq!(result.diagnostics[0].code, code, "{source}");
        assert_eq!(
            (
                result.diagnostics[0].span.start(),
                result.diagnostics[0].span.end()
            ),
            locus,
            "{source}"
        );
        assert_eq!(result.disposition(), InfiniteDisposition::Unsupported);
    }
}

// Trace: TC-058, FR-015-AC-1
#[test]
fn compact_previous_future_is_specific_to_the_v4_lexical_policy() {
    let v4 = parse("YF[0,)p0");
    assert!(v4.document.is_some(), "{:?}", v4.diagnostics);
    let v3 = tl_parse::parse_clean_ascii_v3("YF[0,1]p0", ParseLimits::default());
    assert!(v3.document.is_none());
    assert!(v3
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::UnknownIdentifier));
}

// Trace: TC-061, FR-016-AC-1
#[test]
fn v4_interval_refusals_cover_missing_and_inverted_bounds() {
    for (source, code) in [
        ("F[", DiagnosticCode::UnexpectedToken),
        ("F[,1]p0", DiagnosticCode::UnexpectedToken),
        ("F[0,", DiagnosticCode::UnexpectedToken),
        ("F[0,(p0", DiagnosticCode::UnexpectedToken),
        ("F[2,1]p0", DiagnosticCode::InvalidInterval),
    ] {
        let report = parse(source);
        assert!(report.document.is_none(), "{source}");
        assert_eq!(report.diagnostics[0].code, code, "{source}");
        assert_eq!(report.disposition(), InfiniteDisposition::Unsupported);
    }
}

// Trace: TC-066, NFR-004-AC-1
#[test]
fn node_and_output_limits_refuse_without_partial_artifacts() {
    let limits = ParseLimits {
        max_nodes: 1,
        ..ParseLimits::default()
    };
    let refused = parse_clean_ascii_v4("F[0,)p0", PROFILE, "event_position", limits);
    assert!(refused.document.is_none());
    assert_eq!(refused.diagnostics[0].code, DiagnosticCode::NodeLimit);
    assert_eq!(
        refused.disposition(),
        InfiniteDisposition::ResourceIncomplete
    );

    let admitted = parse("F[0,)p0");
    let format_limits = FormatLimits {
        max_output_bytes: 3,
        ..FormatLimits::default()
    };
    let formatted = format_clean_ascii_v4(admitted.document.as_ref().unwrap(), None, format_limits);
    assert!(formatted.text.is_none());
}

// Trace: TC-184, FR-047-AC-2; TC-066, NFR-004-AC-1
#[test]
fn zero_diagnostic_budget_truncates_clock_refusal_without_emitting_a_diagnostic() {
    let report = parse_clean_ascii_v4(
        "p0",
        PROFILE,
        "wall_clock",
        ParseLimits {
            max_diagnostics: 0,
            ..ParseLimits::default()
        },
    );
    assert!(report.document.is_none());
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.stats.diagnostics, 0);
    assert!(report.stats.diagnostics_truncated);
}

// Trace: TC-184, FR-047-AC-2; TC-066, NFR-004-AC-1
#[test]
fn parser_work_diagnostics_never_leave_a_usable_graph() {
    let mut observed_work_diagnostic_after_nodes = false;
    for source in ["p0", "F[0,)p0", "p0 W[0,) p1"] {
        let baseline = parse(source);
        assert!(baseline.document.is_some());
        for max_work in 0..baseline.stats.work {
            let report = parse_clean_ascii_v4(
                source,
                PROFILE,
                "event_position",
                ParseLimits {
                    max_work,
                    ..ParseLimits::default()
                },
            );
            if report.stats.nodes > 0
                && report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == DiagnosticCode::WorkLimit)
            {
                observed_work_diagnostic_after_nodes = true;
            }
            assert!(report.document.is_none(), "{source}, max_work={max_work}");
            assert!(
                !report.diagnostics.is_empty() || report.stats.diagnostics_truncated,
                "{source}, max_work={max_work}"
            );
        }
    }
    assert!(observed_work_diagnostic_after_nodes);

    let truncated = parse_clean_ascii_v4(
        "p0",
        PROFILE,
        "event_position",
        ParseLimits {
            max_work: 2,
            max_diagnostics: 0,
            ..ParseLimits::default()
        },
    );
    assert_eq!(truncated.stats.nodes, 1);
    assert!(truncated.diagnostics.is_empty());
    assert!(truncated.stats.diagnostics_truncated);
    assert!(truncated.document.is_none());
}

// Trace: TC-184, FR-047-AC-2; TC-066, NFR-004-AC-1
#[test]
fn derived_lowering_accepts_exact_node_budget_and_refuses_before_partial_push() {
    let source = "p0 W[0,) p1";
    let baseline = parse(source);
    let node_count = baseline.stats.nodes;
    assert!(baseline.document.is_some());
    assert_eq!(node_count, 5);
    let exact = parse_clean_ascii_v4(
        source,
        PROFILE,
        "event_position",
        ParseLimits {
            max_nodes: node_count,
            ..ParseLimits::default()
        },
    );
    assert!(exact.document.is_some(), "{:?}", exact.diagnostics);
    assert_eq!(exact.stats.nodes, node_count);
    let under = parse_clean_ascii_v4(
        source,
        PROFILE,
        "event_position",
        ParseLimits {
            max_nodes: node_count - 1,
            ..ParseLimits::default()
        },
    );
    assert!(under.document.is_none());
    assert_eq!(under.diagnostics[0].code, DiagnosticCode::NodeLimit);
    assert_eq!(
        under.stats.nodes, 2,
        "lowering must refuse before partial push"
    );
}

// Trace: TC-066, NFR-004-AC-1
#[test]
fn exact_v4_limits_admit_and_one_under_refuses_deterministically() {
    let source = "fair {G[0,)p0; O[1,2]p1}: (p0 U[2,) p1)";
    let baseline = parse(source);
    assert!(baseline.document.is_some());
    assert_eq!(
        serde_json::to_vec(&baseline).unwrap(),
        serde_json::to_vec(&parse(source)).unwrap()
    );
    for axis in 0..5 {
        let mut exact = ParseLimits::default();
        let mut under = exact;
        match axis {
            0 => {
                exact.max_source_bytes = source.len();
                under.max_source_bytes = source.len() - 1;
            }
            1 => {
                exact.max_tokens = baseline.stats.tokens;
                under.max_tokens = baseline.stats.tokens - 1;
            }
            2 => {
                exact.max_nodes = baseline.stats.nodes;
                under.max_nodes = baseline.stats.nodes - 1;
            }
            3 => {
                exact.max_depth = baseline.stats.max_depth;
                under.max_depth = baseline.stats.max_depth - 1;
            }
            _ => {
                exact.max_work = baseline.stats.work;
                under.max_work = baseline.stats.work - 1;
            }
        }
        let admitted = parse_clean_ascii_v4(source, PROFILE, "event_position", exact);
        assert!(
            admitted.document.is_some(),
            "axis {axis}: {:?}",
            admitted.diagnostics
        );
        let refused = parse_clean_ascii_v4(source, PROFILE, "event_position", under);
        assert!(refused.document.is_none(), "axis {axis}");
        assert_eq!(
            refused.disposition(),
            InfiniteDisposition::ResourceIncomplete,
            "axis {axis}"
        );
    }

    let document = baseline.document.as_ref().unwrap();
    let fairness = baseline.fairness.as_ref();
    let normal = format_clean_ascii_v4(document, fairness, FormatLimits::default());
    let text = normal.text.as_ref().unwrap();
    let exact = FormatLimits {
        max_output_bytes: text.len(),
        max_work: normal.stats.work,
    };
    assert_eq!(
        format_clean_ascii_v4(document, fairness, exact)
            .text
            .as_deref(),
        Some(text.as_str())
    );
    for under in [
        FormatLimits {
            max_output_bytes: text.len() - 1,
            ..exact
        },
        FormatLimits {
            max_work: normal.stats.work - 1,
            ..exact
        },
    ] {
        assert!(format_clean_ascii_v4(document, fairness, under)
            .text
            .is_none());
    }
}

// Trace: TC-065, FR-017-AC-3
#[test]
fn checked_v4_and_v3_fuzz_seeds_execute_real_parser_paths() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus");
    for (target, files) in [
        (
            "unbounded_parse_roundtrip",
            &["fairness.txt", "invalid.txt", "unbounded.txt"][..],
        ),
        ("clean_ascii_v3", &["invalid.txt", "past.txt"][..]),
    ] {
        let directory = root.join(target);
        let sums = fs::read_to_string(directory.join("SHA256SUMS")).unwrap();
        assert_eq!(sums.lines().count(), files.len());
        for (line, file) in sums.lines().zip(files.iter()) {
            let (digest, pinned_file) = line.split_once("  ").unwrap();
            assert_eq!(pinned_file, *file);
            let bytes = fs::read(directory.join(file)).unwrap();
            assert_eq!(digest, format!("{:x}", Sha256::digest(&bytes)));
            let source = std::str::from_utf8(&bytes).unwrap();
            if target == "unbounded_parse_roundtrip" {
                let report = parse(source);
                assert!(report.stats.work <= report.limits.max_work);
                if let Some(document) = report.document.as_ref() {
                    let text = format_clean_ascii_v4(
                        document,
                        report.fairness.as_ref(),
                        FormatLimits::default(),
                    )
                    .text
                    .unwrap();
                    assert!(parse(&text).document.is_some());
                }
            } else {
                let report = tl_parse::parse_clean_ascii_v3(source, ParseLimits::default());
                assert!(report.stats.work <= report.limits.max_work);
                if let Some(document) = report.document.as_ref() {
                    let text = tl_parse::format_clean_ascii_v3(document, FormatLimits::default())
                        .text
                        .unwrap();
                    assert!(
                        tl_parse::parse_clean_ascii_v3(&text, ParseLimits::default())
                            .document
                            .is_some()
                    );
                }
            }
        }
    }
}
