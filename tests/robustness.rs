//! NFR-005: every declared limit is proved at its value and one past it, every
//! public error/code/disposition variant is produced through the public API, and
//! the Miri carve-outs are explicit.
//!
//! "Accepted" means the input is not refused by that limit's own typed error.
//! Two declared limits are shadowed by another limit at the owner value, and each
//! test records which one: any source of `HARD_MAX_SOURCE_BYTES` bytes costs more
//! than `HARD_MAX_PARSE_WORK` work, and the strict-reader work ceiling equals four
//! times the strict-reader byte ceiling.

use std::collections::BTreeSet;

use tl_parse::tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind, Node, NodeId, NodeKind,
    PropositionId, SemanticProfile,
};
use tl_parse::{
    format_clean_ascii_v4, format_document, parse, parse_clean_ascii_v2, parse_clean_ascii_v3,
    parse_clean_ascii_v4, parse_with_context, ContextualParseError, ContextualParseReport,
    DiagnosticCode, FormatErrorCode, FormatLimits, InfiniteDisposition, ParseArtifactLimits,
    ParseLimits, PastParseReport, StrictParseArtifactReadError, HARD_MAX_DEPTH,
    HARD_MAX_DIAGNOSTICS, HARD_MAX_FORMAT_WORK, HARD_MAX_NODES, HARD_MAX_OUTPUT_BYTES,
    HARD_MAX_PARSE_ARTIFACT_BYTES, HARD_MAX_PARSE_ARTIFACT_DEPTH,
    HARD_MAX_PARSE_ARTIFACT_STRING_BYTES, HARD_MAX_PARSE_ARTIFACT_WORK, HARD_MAX_PARSE_WORK,
    HARD_MAX_SOURCE_BYTES, HARD_MAX_TOKENS,
};

const CLOSED: SemanticProfile = SemanticProfile::ClosedTraceV1;
/// The caller-lowered limit the Miri run substitutes for the owner value.
const MIRI_LOWERED: usize = 4096;

fn closed(source: &str, limits: ParseLimits) -> tl_parse::ParseReport {
    parse(source, CLOSED, limits)
}

fn has(report: &tl_parse::ParseReport, code: DiagnosticCode) -> bool {
    report.diagnostics.iter().any(|item| item.code == code)
}

fn everything() -> ParseLimits {
    ParseLimits {
        max_source_bytes: usize::MAX,
        max_tokens: usize::MAX,
        max_nodes: usize::MAX,
        max_depth: usize::MAX,
        max_diagnostics: usize::MAX,
        max_work: usize::MAX,
    }
}

/// A balanced `&` tree over `leaves` propositions: `2 * leaves - 1` nodes.
fn tree(leaves: usize) -> String {
    if leaves == 1 {
        "p0".to_owned()
    } else {
        let left = leaves / 2;
        format!("({}&{})", tree(left), tree(leaves - left))
    }
}

/// A `leaves`-leaf tree whose leaves carry `pairs` extra redundant parentheses
/// in total, so token count can grow without growing node count.
fn padded_tree(leaves: usize, pairs: usize) -> String {
    fn build(first: usize, count: usize, leaves: usize, pairs: usize) -> String {
        if count == 1 {
            let each = pairs / leaves + usize::from(first < pairs % leaves);
            return format!("{}p0{}", "(".repeat(each), ")".repeat(each));
        }
        let left = count / 2;
        format!(
            "({}&{})",
            build(first, left, leaves, pairs),
            build(first + left, count - left, leaves, pairs)
        )
    }
    build(0, leaves, leaves, pairs)
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn parse_limits_clamp_caller_values_to_the_owner_constants() {
    let report = closed("p0", everything());
    assert_eq!(report.limits.max_source_bytes, HARD_MAX_SOURCE_BYTES);
    assert_eq!(report.limits.max_tokens, HARD_MAX_TOKENS);
    assert_eq!(report.limits.max_nodes, HARD_MAX_NODES);
    assert_eq!(report.limits.max_depth, HARD_MAX_DEPTH);
    assert_eq!(report.limits.max_diagnostics, HARD_MAX_DIAGNOSTICS);
    assert_eq!(report.limits.max_work, HARD_MAX_PARSE_WORK);
    let format = format_document(
        report.document.as_ref().unwrap(),
        FormatLimits {
            max_output_bytes: usize::MAX,
            max_work: usize::MAX,
        },
    );
    assert_eq!(format.limits.max_output_bytes, HARD_MAX_OUTPUT_BYTES);
    assert_eq!(format.limits.max_work, HARD_MAX_FORMAT_WORK);
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn source_bytes_limit_accepts_the_limit_and_refuses_one_past() {
    // Native: the owner value. Miri: a caller-lowered limit on the same code path.
    let limit = if cfg!(miri) {
        MIRI_LOWERED
    } else {
        HARD_MAX_SOURCE_BYTES
    };
    let limits = ParseLimits {
        max_source_bytes: limit,
        ..everything()
    };
    let at = closed(&format!("p0{}", " ".repeat(limit - 2)), limits);
    assert_eq!(at.stats.source_bytes, limit);
    assert!(!has(&at, DiagnosticCode::SourceLimit));
    if cfg!(miri) {
        assert!(at.document.is_some());
    } else {
        // Shadowed at the owner value: a source this long costs more than
        // HARD_MAX_PARSE_WORK, so the work limit is the only refusal.
        assert!(has(&at, DiagnosticCode::WorkLimit));
    }
    let past = closed(&format!("p0{}", " ".repeat(limit - 1)), limits);
    assert_eq!(past.diagnostics[0].code, DiagnosticCode::SourceLimit);
    assert!(past.document.is_none());
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn parse_work_limit_accepts_the_limit_and_refuses_one_past() {
    if cfg!(miri) {
        return; // clamp-down-only limit: the clamp is asserted above.
    }
    let at = closed(
        &format!("p0{}", " ".repeat(HARD_MAX_PARSE_WORK - 4)),
        everything(),
    );
    assert_eq!(at.stats.work, HARD_MAX_PARSE_WORK);
    assert!(at.document.is_some());
    let past = closed(
        &format!("p0{}", " ".repeat(HARD_MAX_PARSE_WORK - 3)),
        everything(),
    );
    assert_eq!(past.diagnostics[0].code, DiagnosticCode::WorkLimit);
    assert!(past.document.is_none());
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn token_limit_accepts_the_limit_and_refuses_one_past() {
    if cfg!(miri) {
        return; // clamp-down-only limit: the clamp is asserted above.
    }
    // 1000 leaves, 48_001 redundant parentheses and a leading `!` make exactly
    // HARD_MAX_TOKENS tokens while staying inside the node, depth and work limits.
    let at = closed(&format!("!{}", padded_tree(1000, 48_001)), everything());
    assert_eq!(at.stats.tokens, HARD_MAX_TOKENS);
    assert!(at.document.is_some(), "{:?}", at.diagnostics.first());
    let past = closed(&padded_tree(1000, 48_002), everything());
    assert_eq!(past.diagnostics[0].code, DiagnosticCode::TokenLimit);
    assert!(past.document.is_none());
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn node_limit_accepts_the_limit_and_refuses_one_past() {
    if cfg!(miri) {
        return; // clamp-down-only limit: the clamp is asserted above.
    }
    let at = closed(&format!("!{}", tree(5000)), everything());
    assert_eq!(at.stats.nodes, HARD_MAX_NODES);
    assert!(at.document.is_some(), "{:?}", at.diagnostics.first());
    let past = closed(&format!("!!{}", tree(5000)), everything());
    assert!(has(&past, DiagnosticCode::NodeLimit));
    assert!(past.document.is_none());
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn depth_limit_accepts_the_limit_and_refuses_one_past() {
    if cfg!(miri) {
        return; // clamp-down-only limit: the clamp is asserted above.
    }
    let at = closed(
        &format!("{}p0", "!".repeat(HARD_MAX_DEPTH - 1)),
        everything(),
    );
    assert_eq!(at.stats.max_depth, HARD_MAX_DEPTH);
    assert!(at.document.is_some(), "{:?}", at.diagnostics.first());
    let past = closed(&format!("{}p0", "!".repeat(HARD_MAX_DEPTH)), everything());
    assert!(has(&past, DiagnosticCode::DepthLimit));
    assert!(past.document.is_none());
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn diagnostic_limit_retains_the_limit_and_marks_one_past_truncated() {
    if cfg!(miri) {
        return; // clamp-down-only limit: the clamp is asserted above.
    }
    let at = closed(&"@".repeat(HARD_MAX_DIAGNOSTICS), everything());
    assert_eq!(at.stats.diagnostics, HARD_MAX_DIAGNOSTICS);
    assert!(!at.stats.diagnostics_truncated);
    let past = closed(&"@".repeat(HARD_MAX_DIAGNOSTICS + 1), everything());
    assert_eq!(past.stats.diagnostics, HARD_MAX_DIAGNOSTICS);
    assert!(past.stats.diagnostics_truncated);
}

/// A shared-subgraph document whose canonical text is exactly `length` bytes.
///
/// `c(0)` is `p0` (2 bytes), `c(1)` is `p0&p0` (5 bytes) and each later `c(k)` is
/// `c(k-1)&(c(k-1))` (`2 * c(k-1) + 3` bytes); a left-deep chain of `&` appends shared pieces or `!` chains until the
/// remainder is exactly spent.
fn document_of_length(length: usize) -> tl_parse::tl_syntax::FormulaDocument {
    let mut nodes = vec![Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    let mut shared = vec![(0_u32, 2_usize)];
    let grown = |size: usize| if size == 2 { 5 } else { size * 2 + 3 };
    while grown(shared.last().unwrap().1) <= length {
        let (id, size) = *shared.last().unwrap();
        nodes.push(Node::new(NodeKind::And {
            left: NodeId(id),
            right: NodeId(id),
        }));
        shared.push((u32::try_from(nodes.len() - 1).unwrap(), grown(size)));
    }
    let (mut root, mut size) = *shared.last().unwrap();
    while length - size > 52 {
        // Largest shared piece that leaves at least 3 bytes (operator plus a `p0`).
        let piece = shared
            .iter()
            .rev()
            .find(|(id, piece)| piece + 3 + usize::from(*id != 0) * 2 <= length - size)
            .copied();
        let Some((piece_id, piece_size)) = piece else {
            break;
        };
        nodes.push(Node::new(NodeKind::And {
            left: NodeId(root),
            right: NodeId(piece_id),
        }));
        root = u32::try_from(nodes.len() - 1).unwrap();
        size += 1 + piece_size + usize::from(piece_id != 0) * 2;
    }
    let remainder = length - size;
    // `&` plus a chain of `!` over `p0`: remainder = 1 + 2 + nots.
    assert!(remainder >= 3, "unreachable length {length}");
    let mut operand = 0_u32;
    for _ in 0..remainder - 3 {
        nodes.push(Node::new(NodeKind::Not {
            operand: NodeId(operand),
        }));
        operand = u32::try_from(nodes.len() - 1).unwrap();
    }
    nodes.push(Node::new(NodeKind::And {
        left: NodeId(root),
        right: NodeId(operand),
    }));
    let root = u32::try_from(nodes.len() - 1).unwrap();
    tl_parse::tl_syntax::FormulaDocument::new(CLOSED, NodeId(root), nodes)
        .expect("shared-subgraph document is valid")
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn output_bytes_limit_accepts_the_limit_and_refuses_one_past() {
    // Native: the owner value. Miri: a caller-lowered limit on the same code path.
    let limit = if cfg!(miri) {
        MIRI_LOWERED
    } else {
        HARD_MAX_OUTPUT_BYTES
    };
    let limits = FormatLimits {
        max_output_bytes: limit,
        max_work: usize::MAX,
    };
    let at = format_document(&document_of_length(limit), limits);
    assert_eq!(at.stats.output_bytes, limit);
    assert_eq!(at.text.as_ref().map(String::len), Some(limit));
    if !cfg!(miri) {
        // Format work is nodes visited plus bytes appended, so it stays below
        // HARD_MAX_FORMAT_WORK for every output the output limit admits; the
        // work ceiling can never be the first refusal at its owner value.
        assert!(at.stats.work < HARD_MAX_FORMAT_WORK, "{}", at.stats.work);
    }
    let past = format_document(&document_of_length(limit + 1), limits);
    assert_eq!(past.error.unwrap().code, FormatErrorCode::OutputLimit);
    assert!(past.text.is_none());
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn format_work_limit_refuses_one_past_a_measured_need() {
    let document = document_of_length(512);
    let need = format_document(&document, FormatLimits::default())
        .stats
        .work;
    let at = format_document(
        &document,
        FormatLimits {
            max_work: need,
            ..FormatLimits::default()
        },
    );
    assert!(at.text.is_some());
    let past = format_document(
        &document,
        FormatLimits {
            max_work: need - 1,
            ..FormatLimits::default()
        },
    );
    assert_eq!(past.error.unwrap().code, FormatErrorCode::WorkLimit);
}

fn spaces(count: usize) -> Vec<u8> {
    vec![b' '; count]
}

fn read(bytes: &[u8], limits: ParseArtifactLimits) -> StrictParseArtifactReadError {
    PastParseReport::from_json_bytes(bytes, limits).unwrap_err()
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn artifact_owner_maxima_are_the_declared_constants() {
    let owner = ParseArtifactLimits::OWNER_MAXIMA;
    assert_eq!(owner.document_bytes, HARD_MAX_PARSE_ARTIFACT_BYTES);
    assert_eq!(owner.json_depth, HARD_MAX_PARSE_ARTIFACT_DEPTH);
    assert_eq!(owner.string_bytes, HARD_MAX_PARSE_ARTIFACT_STRING_BYTES);
    assert_eq!(owner.work, HARD_MAX_PARSE_ARTIFACT_WORK);
    // The work ceiling is four units per input byte, so at the owner values it is
    // never the first refusal: the byte ceiling trips one byte earlier.
    assert_eq!(owner.work, owner.document_bytes * 4);
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn artifact_byte_ceiling_accepts_the_limit_and_refuses_one_past() {
    // Native: the owner value, reached by clamping a larger caller value. Miri: a
    // caller-lowered ceiling on the same code path.
    let limit = if cfg!(miri) {
        MIRI_LOWERED
    } else {
        HARD_MAX_PARSE_ARTIFACT_BYTES
    };
    let limits = ParseArtifactLimits {
        document_bytes: if cfg!(miri) { limit } else { usize::MAX },
        work: usize::MAX,
        ..ParseArtifactLimits::OWNER_MAXIMA
    };
    // Blank input passes every preflight ceiling and is then refused as malformed.
    assert!(matches!(
        read(&spaces(limit), limits),
        StrictParseArtifactReadError::InvalidDocument(_)
    ));
    match read(&spaces(limit + 1), limits) {
        StrictParseArtifactReadError::DocumentTooLarge {
            actual,
            limit: seen,
        } => {
            assert_eq!((actual, seen), (limit + 1, limit));
        }
        other => panic!("{other:?}"),
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn artifact_work_ceiling_accepts_the_limit_and_refuses_one_past() {
    // Native and Miri both lower the ceiling: at the owner value the byte ceiling
    // always trips first (see the owner-maxima test).
    let bytes = MIRI_LOWERED;
    let limits = ParseArtifactLimits {
        work: bytes * 4,
        ..ParseArtifactLimits::OWNER_MAXIMA
    };
    assert!(matches!(
        read(&spaces(bytes), limits),
        StrictParseArtifactReadError::InvalidDocument(_)
    ));
    match read(&spaces(bytes + 1), limits) {
        StrictParseArtifactReadError::WorkLimitExceeded { actual, limit } => {
            assert_eq!((actual, limit), ((bytes + 1) * 4, bytes * 4));
        }
        other => panic!("{other:?}"),
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn artifact_depth_ceiling_accepts_the_limit_and_refuses_one_past() {
    let limits = ParseArtifactLimits {
        json_depth: usize::MAX,
        ..ParseArtifactLimits::OWNER_MAXIMA
    };
    let nested = |depth: usize| format!("{}{}", "[".repeat(depth), "]".repeat(depth)).into_bytes();
    assert!(matches!(
        read(&nested(HARD_MAX_PARSE_ARTIFACT_DEPTH), limits),
        StrictParseArtifactReadError::InvalidDocument(_)
    ));
    match read(&nested(HARD_MAX_PARSE_ARTIFACT_DEPTH + 1), limits) {
        StrictParseArtifactReadError::DepthLimitExceeded { actual, limit } => {
            assert_eq!(
                (actual, limit),
                (
                    HARD_MAX_PARSE_ARTIFACT_DEPTH + 1,
                    HARD_MAX_PARSE_ARTIFACT_DEPTH
                )
            );
        }
        other => panic!("{other:?}"),
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn artifact_string_ceiling_accepts_the_limit_and_refuses_one_past() {
    let limits = ParseArtifactLimits {
        string_bytes: usize::MAX,
        ..ParseArtifactLimits::OWNER_MAXIMA
    };
    let quoted = |length: usize| format!("\"{}\"", "a".repeat(length)).into_bytes();
    assert!(matches!(
        read(&quoted(HARD_MAX_PARSE_ARTIFACT_STRING_BYTES), limits),
        StrictParseArtifactReadError::InvalidDocument(_)
    ));
    match read(&quoted(HARD_MAX_PARSE_ARTIFACT_STRING_BYTES + 1), limits) {
        StrictParseArtifactReadError::StringTooLarge { actual, limit } => {
            assert_eq!(
                (actual, limit),
                (
                    HARD_MAX_PARSE_ARTIFACT_STRING_BYTES + 1,
                    HARD_MAX_PARSE_ARTIFACT_STRING_BYTES
                )
            );
        }
        other => panic!("{other:?}"),
    }
}

fn chain_document(length: usize) -> InfiniteFormulaDocument {
    let mut nodes = vec![InfiniteNode::new(InfiniteNodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    for index in 0..length - 1 {
        nodes.push(InfiniteNode::new(InfiniteNodeKind::Not {
            operand: NodeId(u32::try_from(index).unwrap()),
        }));
    }
    let root = NodeId(u32::try_from(length - 1).unwrap());
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        root,
        nodes,
    )
    .expect("a chain this long is within the owner graph limits")
}

// Trace: TC-069, NFR-005-AC-3
#[test]
fn v4_canonical_text_depth_accepts_the_limit_and_refuses_one_past() {
    let at = format_clean_ascii_v4(
        &chain_document(HARD_MAX_DEPTH),
        None,
        FormatLimits::default(),
    );
    assert!(at.error.is_none(), "{:?}", at.error);
    assert!(at.text.is_some());
    let past = format_clean_ascii_v4(
        &chain_document(HARD_MAX_DEPTH + 1),
        None,
        FormatLimits::default(),
    );
    assert_eq!(past.error.unwrap().code, FormatErrorCode::InvalidGraph);
    assert!(past.text.is_none());
}

fn all_codes() -> [DiagnosticCode; 18] {
    use DiagnosticCode::*;
    [
        SourceLimit,
        TokenLimit,
        NodeLimit,
        DepthLimit,
        WorkLimit,
        UnexpectedCharacter,
        UnknownIdentifier,
        NonCanonicalNumber,
        IntegerOverflow,
        UnexpectedToken,
        MissingToken,
        InvalidInterval,
        TrailingInput,
        ValidationFailure,
        UnsupportedOperator,
        InfiniteProfileMismatch,
        InfiniteClockMismatch,
        DuplicateFairnessPremise,
    ]
}

/// Fails to compile when `DiagnosticCode` gains a variant this file does not list.
fn exhaustive(code: DiagnosticCode) -> usize {
    use DiagnosticCode::*;
    match code {
        SourceLimit
        | TokenLimit
        | NodeLimit
        | DepthLimit
        | WorkLimit
        | UnexpectedCharacter
        | UnknownIdentifier
        | NonCanonicalNumber
        | IntegerOverflow
        | UnexpectedToken
        | MissingToken
        | InvalidInterval
        | TrailingInput
        | ValidationFailure
        | UnsupportedOperator
        | InfiniteProfileMismatch
        | InfiniteClockMismatch
        | DuplicateFairnessPremise => 1,
    }
}

// Trace: TC-071, NFR-005-AC-5
#[test]
fn every_diagnostic_code_is_returned_by_the_parsers() {
    let small = |patch: fn(&mut ParseLimits)| {
        let mut limits = ParseLimits::default();
        patch(&mut limits);
        limits
    };
    let mut seen = BTreeSet::new();
    let mut note = |codes: Vec<DiagnosticCode>| seen.extend(codes);
    let codes = |report: &tl_parse::ParseReport| -> Vec<DiagnosticCode> {
        report.diagnostics.iter().map(|item| item.code).collect()
    };
    for (source, limits) in [
        ("p0", small(|l| l.max_source_bytes = 1)),
        ("p0&p1", small(|l| l.max_tokens = 1)),
        ("!p0", small(|l| l.max_nodes = 1)),
        ("!p0", small(|l| l.max_depth = 1)),
        ("p0&p1", small(|l| l.max_work = 4)),
        ("@", ParseLimits::default()),
        ("x", ParseLimits::default()),
        ("F[01,2]p0", ParseLimits::default()),
        ("p99999999999", ParseLimits::default()),
        ("p0 p1", ParseLimits::default()),
        ("(p0", ParseLimits::default()),
        ("F[2,1]p0", ParseLimits::default()),
        ("p0)", ParseLimits::default()),
    ] {
        note(codes(&closed(source, limits)));
    }
    // A profile the closed-trace dialect cannot validate under reaches the owner
    // validation boundary.
    note(codes(&parse(
        "p0",
        SemanticProfile::OriginCompleteHistoryV1,
        ParseLimits::default(),
    )));
    note(
        parse_clean_ascii_v2("O[0,1]p0", CLOSED, ParseLimits::default())
            .diagnostics
            .iter()
            .map(|item| item.code)
            .collect(),
    );
    note(
        parse_clean_ascii_v3("F[0,1]p0", ParseLimits::default())
            .diagnostics
            .iter()
            .map(|item| item.code)
            .collect(),
    );
    let v4 = |source: &str, profile, clock: &str| -> Vec<DiagnosticCode> {
        parse_clean_ascii_v4(source, profile, clock, ParseLimits::default())
            .diagnostics
            .iter()
            .map(|item| item.code)
            .collect()
    };
    note(v4("F[0,)p0", CLOSED, "event_position"));
    note(v4(
        "F[0,)p0",
        SemanticProfile::InfiniteTraceV1,
        "wall_clock",
    ));
    note(v4(
        "fair { F[0,)p0; F[0,)p0; } : p1",
        SemanticProfile::InfiniteTraceV1,
        "event_position",
    ));
    let expected: BTreeSet<_> = all_codes().into_iter().collect();
    assert_eq!(expected.len(), 18);
    assert!(all_codes().into_iter().all(|code| exhaustive(code) == 1));
    let missing: Vec<_> = expected.difference(&seen).collect();
    assert!(missing.is_empty(), "never returned: {missing:?}");
}

// Trace: TC-071, NFR-005-AC-5
#[test]
fn every_format_error_code_is_returned_by_the_formatters() {
    let mut seen = BTreeSet::new();
    let formatted = format_document(
        &document_of_length(64),
        FormatLimits {
            max_output_bytes: 4,
            ..FormatLimits::default()
        },
    );
    seen.insert(formatted.error.unwrap().code.as_str());
    let formatted = format_document(
        &document_of_length(64),
        FormatLimits {
            max_work: 4,
            ..FormatLimits::default()
        },
    );
    seen.insert(formatted.error.unwrap().code.as_str());
    let deep = format_clean_ascii_v4(
        &chain_document(HARD_MAX_DEPTH + 1),
        None,
        FormatLimits::default(),
    );
    seen.insert(deep.error.unwrap().code.as_str());
    let disconnected = InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
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
    .unwrap();
    let report = format_clean_ascii_v4(&disconnected, None, FormatLimits::default());
    seen.insert(report.error.unwrap().code.as_str());
    let expected: BTreeSet<_> = [
        FormatErrorCode::InvalidGraph,
        FormatErrorCode::UnrepresentableGraph,
        FormatErrorCode::OutputLimit,
        FormatErrorCode::WorkLimit,
    ]
    .into_iter()
    .map(FormatErrorCode::as_str)
    .collect();
    assert_eq!(seen, expected);
}

// Trace: TC-071, NFR-005-AC-5
#[test]
fn every_strict_reader_error_is_returned_by_the_readers() {
    let report = parse_clean_ascii_v3("O[0,1]p0", ParseLimits::default());
    let bytes = report.canonical_json_bytes().unwrap();
    let mut seen = BTreeSet::new();
    let mut note = |error: StrictParseArtifactReadError| {
        seen.insert(match error {
            StrictParseArtifactReadError::DocumentTooLarge { .. } => "DocumentTooLarge",
            StrictParseArtifactReadError::DepthLimitExceeded { .. } => "DepthLimitExceeded",
            StrictParseArtifactReadError::StringTooLarge { .. } => "StringTooLarge",
            StrictParseArtifactReadError::WorkLimitExceeded { .. } => "WorkLimitExceeded",
            StrictParseArtifactReadError::NonCanonicalDocument => "NonCanonicalDocument",
            StrictParseArtifactReadError::InvalidDocument(_) => "InvalidDocument",
            _ => "unlisted",
        });
    };
    for limits in [
        ParseArtifactLimits {
            document_bytes: bytes.len() - 1,
            ..ParseArtifactLimits::OWNER_MAXIMA
        },
        ParseArtifactLimits {
            json_depth: 1,
            ..ParseArtifactLimits::OWNER_MAXIMA
        },
        ParseArtifactLimits {
            string_bytes: 1,
            ..ParseArtifactLimits::OWNER_MAXIMA
        },
        ParseArtifactLimits {
            work: bytes.len() * 4 - 1,
            ..ParseArtifactLimits::OWNER_MAXIMA
        },
    ] {
        note(read(&bytes, limits));
    }
    let mut padded = bytes.clone();
    padded.push(b'\n');
    note(read(&padded, ParseArtifactLimits::OWNER_MAXIMA));
    note(read(b"{}", ParseArtifactLimits::OWNER_MAXIMA));
    let expected: BTreeSet<_> = [
        "DocumentTooLarge",
        "DepthLimitExceeded",
        "StringTooLarge",
        "WorkLimitExceeded",
        "NonCanonicalDocument",
        "InvalidDocument",
    ]
    .into_iter()
    .collect();
    assert_eq!(seen, expected);
}

mod contextual {
    use super::*;
    use serde_json::Value;
    use sha2::Digest;
    use tl_parse::tl_syntax::{
        OwnedSignalDeclaration, PropositionBinding, SignalCatalogDocument, SignalDomain, SignalId,
    };

    fn catalog() -> SignalCatalogDocument {
        SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(10),
                "brake".into(),
                SignalDomain::Boolean,
            )],
            vec![PropositionBinding::new(PropositionId(1), SignalId(10))],
        )
        .unwrap()
    }

    fn bind(source: &str) -> Result<ContextualParseReport, ContextualParseError> {
        parse_with_context(source, CLOSED, ParseLimits::default(), &catalog(), None)
    }

    // Trace: TC-071, NFR-005-AC-5
    #[test]
    fn every_contextual_error_is_returned_by_binding() {
        assert!(matches!(
            bind("p1 &"),
            Err(ContextualParseError::ParseFailed)
        ));
        assert!(matches!(
            bind("p2"),
            Err(ContextualParseError::MissingProposition { .. })
        ));
        // A catalog cannot be invalid: `new` and deserialization both validate it, so
        // the contextual parser has no invalid-catalog error to return.
        let invalid = serde_json::from_str::<SignalCatalogDocument>(
            &serde_json::to_string(&catalog())
                .unwrap()
                .replace("\"signal\":10", "\"signal\":99"),
        );
        assert!(invalid.is_err());
        // `parse` spans every node it emits, so the span error is reachable only
        // through the strict wire reader. Strip a span and restore the request
        // digest so the reader reaches the binding check.
        let report = bind("p1").unwrap();
        let mut wire: Value = serde_json::to_value(&report).unwrap();
        wire["formulaDocument"]["nodes"][0]
            .as_object_mut()
            .unwrap()
            .remove("span");
        let request = serde_json::json!({
            "domain": "tl-parse.contextual-binding/request/v2",
            "formulaDocument": wire["formulaDocument"],
            "signalCatalog": wire["signalCatalog"],
            "requirementContext": wire["requirementContext"],
        });
        wire["requestSha256"] = Value::String(format!(
            "{:x}",
            sha2::Sha256::digest(serde_json::to_vec(&request).unwrap())
        ));
        let error = serde_json::from_value::<ContextualParseReport>(wire).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("missing parser span for proposition 1"),
            "{error}"
        );
    }
}

// Trace: TC-071, NFR-005-AC-5
#[test]
fn every_infinite_disposition_is_returned_by_the_v4_report() {
    let v4 = |source: &str, profile, limits| {
        parse_clean_ascii_v4(source, profile, "event_position", limits)
    };
    let profile = SemanticProfile::InfiniteTraceV1;
    let accepted = v4("F[0,)p0", profile, ParseLimits::default());
    assert_eq!(
        accepted.disposition(),
        InfiniteDisposition::NoTemporalVerdict
    );
    let mismatch = v4("F[0,)p0", CLOSED, ParseLimits::default());
    assert_eq!(mismatch.disposition(), InfiniteDisposition::Unsupported);
    let limited = v4(
        "F[0,)p0",
        profile,
        ParseLimits {
            max_tokens: 1,
            ..ParseLimits::default()
        },
    );
    assert_eq!(
        limited.disposition(),
        InfiniteDisposition::ResourceIncomplete
    );
    // A report that holds neither a graph nor a typed refusal is a failure.
    let mut failed = accepted;
    failed.document = None;
    failed.refusal = None;
    assert_eq!(failed.disposition(), InfiniteDisposition::Failed);
}

// Trace: TC-072, NFR-005-AC-6
#[test]
fn miri_target_runs_in_the_ci_composite_on_the_pinned_nightly() {
    let makefile = include_str!("../Makefile");
    let composite = makefile
        .split("\nci:")
        .nth(1)
        .and_then(|rest| rest.split("\n\n").next())
        .expect("Makefile declares the ci composite");
    assert!(
        composite.split_whitespace().any(|word| word == "miri"),
        "{composite}"
    );
    assert!(makefile.contains("MIRI_TOOLCHAIN ?= nightly-"));
    assert!(makefile.contains("MIRIFLAGS"));
    assert!(makefile.contains("miri test"));
}
