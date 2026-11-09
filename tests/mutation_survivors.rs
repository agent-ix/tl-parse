//! TL-371: behavior tests for mutants that survived the first full cargo-mutants
//! run. Each test names the observable behavior it pins down; none restates the
//! implementation. SR-231 carries the dated measurements.

use std::{io::Write, process::Command};

use tempfile::NamedTempFile;
use tl_parse::tl_syntax::SemanticProfile;
use tl_parse::{
    dialect_digest, dialect_v2_digest, dialect_v3_digest, format_document, FormatLimits,
    ParseArtifactLimits, ParseLimits, PastParseReport, StrictParseArtifactReadError,
    DIALECT_RECORD, DIALECT_V2_RECORD, DIALECT_V3_RECORD, HARD_MAX_PARSE_ARTIFACT_BYTES,
    HARD_MAX_PARSE_ARTIFACT_DEPTH, HARD_MAX_PARSE_ARTIFACT_STRING_BYTES,
    HARD_MAX_PARSE_ARTIFACT_WORK,
};

/// Reads `bytes` with a lowered string limit and reports the size the strict
/// scanner charged when it refused the string, or `None` when the scanner
/// admitted it (the JSON parser then refuses the non-report document).
fn string_charge(literal: &str, string_bytes: usize) -> Option<usize> {
    // An array has no member names, so only the literal's decoded size is charged.
    let bytes = format!("[{literal}]");
    let limits = ParseArtifactLimits {
        string_bytes,
        ..ParseArtifactLimits::default()
    };
    match PastParseReport::from_json_bytes(bytes.as_bytes(), limits) {
        Err(StrictParseArtifactReadError::StringTooLarge { actual, limit }) => {
            assert_eq!(limit, string_bytes, "{literal}");
            Some(actual)
        }
        Err(StrictParseArtifactReadError::InvalidDocument(_)) => None,
        other => panic!("{literal}: unexpected {other:?}"),
    }
}

/// Rows write `@` where the JSON text has a backslash, so the escapes below
/// read as the JSON the scanner sees.
fn json(literal: &str) -> String {
    literal.replace('@', "\\")
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn strict_scanner_charges_each_escape_its_decoded_utf8_width() {
    // (literal, decoded bytes). A limit one below the width refuses with the
    // width; a limit equal to it admits the string.
    let rows: [(&str, usize); 23] = [
        ("\"abc\"", 3),
        ("\"@n\"", 1),
        ("\"@u0041\"", 1),
        ("\"@u007f\"", 1),
        ("\"@u007F\"", 1),
        ("\"@u0080\"", 2),
        ("\"@u00a0\"", 2),
        ("\"@u00A0\"", 2),
        ("\"@u07ff\"", 2),
        ("\"@u07FF\"", 2),
        ("\"@u0800\"", 3),
        ("\"@uabcd\"", 3),
        ("\"@uABCD\"", 3),
        ("\"@ud83d@ude00\"", 4),
        ("\"@ud800@udc00\"", 4),
        ("\"@uD83D@uDE00\"", 4),
        // A high surrogate whose second escape is not a low surrogate: both are scalars.
        ("\"@ud83d@u00e9\"", 5),
        // A high surrogate followed by an escaped backslash is not a pair.
        ("\"@ud83d@@u00e9\"", 9),
        // Each part of the pair test must hold: backslash, `u`, and a low surrogate.
        ("\"@ud83dxxdc00\"", 9),
        ("\"@ud83dxudc00\"", 9),
        ("\"@ud83d@xdc00\"", 8),
        // A pair needs a high surrogate first: a low surrogate after a scalar is a scalar.
        ("\"@u00e9@udc00\"", 5),
        // A malformed escape charges nothing, then one byte per remaining character.
        ("\"@u12G4\"", 4),
    ];
    for (literal, width) in rows {
        let literal = json(literal);
        assert_eq!(string_charge(&literal, width - 1), Some(width), "{literal}");
        assert_eq!(
            string_charge(&literal, width),
            None,
            "{literal} at its limit"
        );
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn strict_scanner_skips_the_whole_escape_it_decoded() {
    // `@u00e9` is two bytes however it is spelled; the four hex digits are not
    // charged again as plain characters.
    let twice = json("\"@u00e9@u00e9\"");
    assert_eq!(string_charge(&twice, 3), Some(4));
    assert_eq!(string_charge(&twice, 4), None);
    // A string whose closing quote follows an escape ends there.
    assert_eq!(string_charge(&json("\"@n\""), 1), None);
    // An escaped quote does not close the string.
    assert_eq!(string_charge(&json("\"@\"@\"@\"\""), 2), Some(3));
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn strict_scanner_accepts_depth_equal_to_the_limit_and_refuses_one_past() {
    for limit in [1_usize, 2, 7] {
        let limits = ParseArtifactLimits {
            json_depth: limit,
            ..ParseArtifactLimits::default()
        };
        let at = format!("{}{}", "[".repeat(limit), "]".repeat(limit));
        assert!(
            matches!(
                PastParseReport::from_json_bytes(at.as_bytes(), limits),
                Err(StrictParseArtifactReadError::InvalidDocument(_))
            ),
            "depth {limit} is within the limit"
        );
        let past = format!("{}{}", "[".repeat(limit + 1), "]".repeat(limit + 1));
        match PastParseReport::from_json_bytes(past.as_bytes(), limits) {
            Err(StrictParseArtifactReadError::DepthLimitExceeded { actual, limit: l }) => {
                assert_eq!((actual, l), (limit + 1, limit));
            }
            other => panic!("depth {} was not refused: {other:?}", limit + 1),
        }
        // Closing a container lowers the depth: sequential containers never nest.
        let sequential = format!("[{}]", "[]".repeat(limit));
        let sequential_limits = ParseArtifactLimits {
            json_depth: 2,
            ..ParseArtifactLimits::default()
        };
        assert!(matches!(
            PastParseReport::from_json_bytes(sequential.as_bytes(), sequential_limits),
            Err(StrictParseArtifactReadError::InvalidDocument(_))
        ));
        // Both bracket kinds close a container.
        let braces = format!("[{}]", "{}".repeat(limit));
        assert!(matches!(
            PastParseReport::from_json_bytes(braces.as_bytes(), sequential_limits),
            Err(StrictParseArtifactReadError::InvalidDocument(_))
        ));
    }
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn strict_reader_ceilings_have_their_documented_values() {
    assert_eq!(HARD_MAX_PARSE_ARTIFACT_BYTES, 67_108_864);
    assert_eq!(HARD_MAX_PARSE_ARTIFACT_DEPTH, 64);
    assert_eq!(HARD_MAX_PARSE_ARTIFACT_STRING_BYTES, 65_536);
    assert_eq!(HARD_MAX_PARSE_ARTIFACT_WORK, 268_435_456);
    let owner = ParseArtifactLimits::OWNER_MAXIMA;
    assert_eq!(owner.document_bytes, HARD_MAX_PARSE_ARTIFACT_BYTES);
    assert_eq!(owner.json_depth, HARD_MAX_PARSE_ARTIFACT_DEPTH);
    assert_eq!(owner.string_bytes, HARD_MAX_PARSE_ARTIFACT_STRING_BYTES);
    assert_eq!(owner.work, HARD_MAX_PARSE_ARTIFACT_WORK);
    assert_eq!(ParseArtifactLimits::default(), owner);
}

// Trace: TC-068, NFR-005-AC-2
#[test]
fn strict_reader_admits_a_document_exactly_at_its_byte_and_work_ceilings() {
    let report = past("O[0,1](p0S[0,2]Yp1)");
    let valid = report.canonical_json_bytes().unwrap();
    let exact = ParseArtifactLimits {
        document_bytes: valid.len(),
        work: valid.len() * 4,
        ..ParseArtifactLimits::default()
    };
    assert!(PastParseReport::from_json_bytes(&valid, exact).is_ok());
    let bytes_short = ParseArtifactLimits {
        document_bytes: valid.len() - 1,
        ..exact
    };
    assert!(matches!(
        PastParseReport::from_json_bytes(&valid, bytes_short),
        Err(StrictParseArtifactReadError::DocumentTooLarge { .. })
    ));
}

#[test]
fn strict_scanner_survives_strings_left_open_at_the_end_of_input() {
    // The scanner must stop at the end of input instead of reading past it.
    for open in ["[\"abc", "[\"", "[\"\\", "[\"\\u00", "[\"\\ud83d\\u"] {
        assert!(
            matches!(
                PastParseReport::from_json_bytes(open.as_bytes(), ParseArtifactLimits::default()),
                Err(StrictParseArtifactReadError::InvalidDocument(_))
            ),
            "{open:?}"
        );
    }
}

// Trace: TC-070, NFR-005-AC-4
#[test]
#[cfg_attr(miri, ignore = "spawns the CLI binary")]
fn cli_file_read_accepts_the_source_limit_and_refuses_one_byte_past() {
    let limit = tl_parse::HARD_MAX_SOURCE_BYTES;
    let run = |bytes: usize| {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(format!("p0{}", " ".repeat(bytes - 2)).as_bytes())
            .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_tl-parse"))
            .args(["validate", file.path().to_str().unwrap()])
            .output()
            .unwrap();
        String::from_utf8(output.stderr).unwrap()
    };
    // A file of exactly the limit is read; the work limit (not the source limit) refuses it.
    let at = run(limit);
    assert!(!at.contains("source_limit"), "{at}");
    assert!(at.contains("work_limit"), "{at}");
    let past = run(limit + 1);
    assert!(past.contains("source_limit"), "{past}");
}

#[test]
fn dialect_digests_are_the_lowercase_sha256_of_their_records() {
    use sha2::{Digest, Sha256};
    let rows = [
        (dialect_digest(), DIALECT_RECORD),
        (dialect_v2_digest(), DIALECT_V2_RECORD),
        (dialect_v3_digest(), DIALECT_V3_RECORD),
    ];
    for (digest, record) in &rows {
        assert_eq!(digest.len(), 64);
        assert!(digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
        assert_eq!(*digest, format!("{:x}", Sha256::digest(record.as_bytes())));
    }
    assert_ne!(rows[0].0, rows[1].0);
    assert_ne!(rows[1].0, rows[2].0);
    assert_ne!(rows[0].0, rows[2].0);
}

// Trace: TC-046, FR-009-AC-3
#[test]
fn format_document_selects_the_v3_dialect_for_origin_complete_history_documents() {
    let report = past("O[0,2]p0S[0,1]Yp1");
    let document = report.document.expect("v3 source parses");
    assert_eq!(
        document.semantic_profile(),
        SemanticProfile::OriginCompleteHistoryV1
    );
    let formatted = format_document(&document, FormatLimits::default());
    assert_eq!(formatted.text.as_deref(), Some("O[0,2]p0S[0,1]Yp1"));
}

#[test]
fn contextual_parse_errors_render_their_diagnostics() {
    use tl_parse::tl_syntax::{PropositionId, SourceSpan};
    use tl_parse::ContextualParseError;
    assert_eq!(
        ContextualParseError::ParseFailed.to_string(),
        "parse failed"
    );
    let missing = ContextualParseError::MissingProposition {
        proposition: PropositionId(7),
        parser_span: SourceSpan::new(3, 5).unwrap(),
    };
    assert_eq!(
        missing.to_string(),
        "missing signal binding for proposition 7 at 3..5"
    );
}

fn past(source: &str) -> tl_parse::PastParseReport {
    tl_parse::parse_clean_ascii_v3(source, ParseLimits::default())
}

fn derived(source: &str) -> tl_parse::DerivedParseReport {
    tl_parse::parse_clean_ascii_v2(
        source,
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
    )
}

fn infinite(source: &str) -> tl_parse::InfiniteParseReport {
    tl_parse::parse_clean_ascii_v4(
        source,
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        ParseLimits::default(),
    )
}

fn has_work_limit(diagnostics: &[tl_parse::Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == tl_parse::DiagnosticCode::WorkLimit)
}

/// Work consumed by `source`, and whether `max_work` refused it.
fn work_run(source: &'static str, lowering: bool, max_work: usize) -> (usize, bool) {
    let limits = ParseLimits {
        max_work,
        ..ParseLimits::default()
    };
    if lowering {
        let report = tl_parse::parse_clean_ascii_v2(source, SemanticProfile::ClosedTraceV1, limits);
        (report.stats.work, has_work_limit(&report.diagnostics))
    } else {
        let report = tl_parse::parse(source, SemanticProfile::ClosedTraceV1, limits);
        (report.stats.work, has_work_limit(&report.diagnostics))
    }
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn parser_work_limit_admits_exactly_the_work_a_parse_costs() {
    let shared = [
        "p0",
        "p0&p1",
        "F[0,3](p0->!p1)",
        "G[1,2]p0 U[0,4] (p1|p2)",
        "!!!!p0",
    ];
    let derived = ["p0 W[0,1] p1", "p0 M[0,2] p1", "(p0 W[0,1] p1) M[1,2] p2"];
    let rows = shared
        .iter()
        .flat_map(|source| [(*source, false), (*source, true)])
        .chain(derived.iter().map(|source| (*source, true)));
    for (source, lowering) in rows {
        let (work, refused) = work_run(source, lowering, usize::MAX);
        assert!(!refused && work > 0, "{source}");
        let (_, refused) = work_run(source, lowering, work);
        assert!(!refused, "{source}: max_work == work must be admitted");
        let (_, refused) = work_run(source, lowering, work - 1);
        assert!(refused, "{source}: max_work == work - 1 must be refused");
    }
}

#[test]
fn work_limit_recovery_diagnostics_are_exact() {
    use tl_parse::{DiagnosticCode as C, RecoveryAction as R};
    // The limit trips mid-production: the refusal is followed only by the
    // recovery the interrupted production emits, never by a second parse.
    let rows: Vec<(&str, usize, Vec<Shape>)> = vec![
        (
            "F[0,3](p0->!p1)",
            14,
            vec![
                (C::WorkLimit, 0, 1, R::Stopped),
                (C::MissingToken, 0, 1, R::InsertedToken),
            ],
        ),
        (
            "F[0,3](p0->!p1)",
            15,
            vec![
                (C::WorkLimit, 1, 2, R::Stopped),
                (C::UnexpectedToken, 1, 2, R::SkippedToken),
                (C::WorkLimit, 1, 2, R::Stopped),
            ],
        ),
        (
            "F[0,3](p0->!p1)",
            17,
            vec![
                (C::WorkLimit, 3, 4, R::Stopped),
                (C::UnexpectedToken, 3, 4, R::SkippedToken),
                (C::WorkLimit, 3, 4, R::Stopped),
            ],
        ),
        (
            "F[0,3](p0->!p1)",
            19,
            vec![
                (C::WorkLimit, 5, 6, R::Stopped),
                (C::WorkLimit, 5, 6, R::Stopped),
            ],
        ),
    ];
    for (source, max_work, expected) in rows {
        let report = tl_parse::parse(
            source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits {
                max_work,
                ..ParseLimits::default()
            },
        );
        assert_eq!(
            shapes(&report.diagnostics),
            expected,
            "{source} at {max_work}"
        );
    }
}

type Shape = (tl_parse::DiagnosticCode, u32, u32, tl_parse::RecoveryAction);

fn shapes(diagnostics: &[tl_parse::Diagnostic]) -> Vec<Shape> {
    diagnostics
        .iter()
        .map(|d| (d.code, d.span.start(), d.span.end(), d.recovery))
        .collect()
}

fn closed(source: &str) -> tl_parse::ParseReport {
    let source = source.to_owned();
    tl_parse::parse(
        &source,
        SemanticProfile::ClosedTraceV1,
        ParseLimits::default(),
    )
}

#[test]
fn recovery_actions_and_skipped_tokens_are_exact_for_malformed_input() {
    use tl_parse::{DiagnosticCode as C, RecoveryAction as R};
    // (source, diagnostics): an unexpected token is skipped exactly once, and a
    // missing token at end of input is inserted, never skipped.
    let rows: Vec<(&str, Vec<Shape>)> = vec![
        ("", vec![(C::UnexpectedToken, 0, 0, R::InsertedToken)]),
        ("&", vec![(C::UnexpectedToken, 0, 1, R::SkippedToken)]),
        ("(", vec![(C::UnexpectedToken, 1, 1, R::InsertedToken)]),
        (
            "F[&,1]p0",
            vec![
                (C::UnexpectedToken, 2, 3, R::SkippedToken),
                (C::TrailingInput, 3, 4, R::SkippedToken),
                (C::TrailingInput, 4, 5, R::SkippedToken),
                (C::TrailingInput, 5, 6, R::SkippedToken),
                (C::TrailingInput, 6, 8, R::SkippedToken),
            ],
        ),
        (
            "F[0,&]p0",
            vec![
                (C::UnexpectedToken, 4, 5, R::SkippedToken),
                (C::TrailingInput, 5, 6, R::SkippedToken),
                (C::TrailingInput, 6, 8, R::SkippedToken),
            ],
        ),
        ("F[0,5", vec![(C::MissingToken, 5, 5, R::InsertedToken)]),
        ("F[", vec![(C::UnexpectedToken, 2, 2, R::SkippedToken)]),
        ("F[0,", vec![(C::UnexpectedToken, 4, 4, R::SkippedToken)]),
        ("p", vec![(C::UnknownIdentifier, 0, 1, R::SkippedToken)]),
    ];
    for (source, expected) in rows {
        let report = closed(source);
        assert_eq!(shapes(&report.diagnostics), expected, "{source:?}");
    }
}

// Trace: TC-004, FR-001-AC-2
#[test]
fn a_number_that_ends_the_input_is_lexed_to_its_last_digit() {
    // `scan_number` reaches the end of the source without reading past it.
    for source in ["F[0,12", "F[12"] {
        let report = closed(source);
        assert!(report.document.is_none(), "{source}");
        assert!(!report.diagnostics.is_empty(), "{source}");
    }
    let report = closed("p00");
    assert_eq!(
        report.diagnostics[0].code,
        tl_parse::DiagnosticCode::NonCanonicalNumber
    );
    assert_eq!(report.diagnostics[0].found, "\"p00\"");
}

#[test]
fn found_text_is_clipped_after_thirty_two_characters() {
    let at_limit = "a".repeat(32);
    let past_limit = "a".repeat(33);
    assert_eq!(
        closed(&at_limit).diagnostics[0].found,
        format!("{at_limit:?}")
    );
    assert_eq!(
        closed(&past_limit).diagnostics[0].found,
        format!("\"{at_limit}\u{2026}\"")
    );
}

#[test]
fn infinite_dialect_keeps_an_identifier_tail_with_its_proposition() {
    let parse = |source: &str| {
        let source = source.to_owned();
        tl_parse::parse_clean_ascii_v4(
            &source,
            SemanticProfile::InfiniteTraceV1,
            "event_position",
            ParseLimits::default(),
        )
    };
    // An underscore continues the identifier, so the whole word is one unknown identifier.
    let report = parse("p0_x");
    assert_eq!(
        report.diagnostics[0].code,
        tl_parse::DiagnosticCode::UnknownIdentifier
    );
    assert_eq!(
        (
            report.diagnostics[0].span.start(),
            report.diagnostics[0].span.end()
        ),
        (0, 4)
    );
    let bare = parse("p");
    assert_eq!(
        bare.diagnostics[0].code,
        tl_parse::DiagnosticCode::UnknownIdentifier
    );
}

#[test]
fn lexing_terminates_on_propositions_identifiers_and_numbers() {
    for source in [
        "p12345",
        "abcdef",
        "F[123456,654321]p9",
        "true&false",
        "p1U[0,1]p2",
    ] {
        let report = closed(source);
        assert!(report.stats.tokens > 0, "{source}");
    }
}

fn past_ok(report: &PastParseReport) -> bool {
    let bytes = serde_json::to_vec(report).unwrap();
    PastParseReport::from_json_bytes(&bytes, ParseArtifactLimits::default()).is_ok()
}

#[test]
fn past_report_reader_admits_each_counter_at_its_limit_and_refuses_one_past() {
    let good = past("O[0,1](p0S[0,2]Yp1)");
    assert!(past_ok(&good));
    let stats = good.stats;
    assert!(stats.max_depth >= 2, "the fixture must be nested");

    type Edit = fn(&mut PastParseReport, usize);
    // (counter, value of the counter, edit that sets the matching limit to a value).
    let rows: [(&str, usize, Edit); 5] = [
        ("tokens", stats.tokens, |r, v| r.limits.max_tokens = v),
        ("nodes", stats.nodes, |r, v| r.limits.max_nodes = v),
        ("work", stats.work, |r, v| r.limits.max_work = v),
        ("source bytes", stats.source_bytes, |r, v| {
            r.limits.max_source_bytes = v;
        }),
        ("depth", stats.max_depth, |r, v| r.limits.max_depth = v),
    ];
    for (name, value, set_limit) in rows {
        let mut at = good.clone();
        set_limit(&mut at, value);
        assert!(past_ok(&at), "{name} equal to its limit is admitted");
        let mut past = good.clone();
        set_limit(&mut past, value - 1);
        assert!(!past_ok(&past), "{name} one past its limit is refused");
    }
    // The depth ceiling of a failed parse is one deeper than a successful one.
    let mut deeper = good.clone();
    deeper.document = None;
    deeper.diagnostics = past("O[0,").diagnostics;
    deeper.stats.diagnostics = deeper.diagnostics.len();
    deeper.limits.max_depth = stats.max_depth - 1;
    assert!(
        past_ok(&deeper),
        "a failed parse may be one level past max_depth"
    );
    deeper.limits.max_depth = stats.max_depth - 2;
    assert!(!past_ok(&deeper), "two levels past max_depth is refused");
}

#[test]
fn past_report_reader_applies_each_failed_parse_rule_independently() {
    let good = past("O[0,1](p0S[0,2]Yp1)");
    let failed = past("O[0,");
    assert!(failed.document.is_none() && past_ok(&failed));

    // A failed parse with no retained diagnostic is admitted only when truncated.
    let mut truncated = failed.clone();
    truncated.diagnostics.clear();
    truncated.stats.diagnostics = 0;
    assert!(!past_ok(&truncated));
    truncated.stats.diagnostics_truncated = true;
    assert!(past_ok(&truncated));

    // A document is refused next to either kind of failure evidence.
    let mut with_truncation = good.clone();
    with_truncation.stats.diagnostics_truncated = true;
    assert!(!past_ok(&with_truncation));

    // Diagnostic count and limit.
    let mut capped = failed.clone();
    capped.limits.max_diagnostics = failed.diagnostics.len();
    assert!(past_ok(&capped));
    capped.limits.max_diagnostics = failed.diagnostics.len() - 1;
    assert!(!past_ok(&capped));

    // Spans may end exactly at the last submitted byte and no later.
    let source_bytes = u32::try_from(failed.stats.source_bytes).unwrap();
    let mut edge = failed.clone();
    edge.diagnostics[0].span = tl_parse::tl_syntax::SourceSpan::new(0, source_bytes).unwrap();
    assert!(past_ok(&edge));
    edge.diagnostics[0].span = tl_parse::tl_syntax::SourceSpan::new(0, source_bytes + 1).unwrap();
    assert!(!past_ok(&edge));
    let mut node_edge = good.clone();
    node_edge.stats.source_bytes -= 1;
    assert!(
        !past_ok(&node_edge),
        "a node span past the source is refused"
    );
}

fn derived_ok(report: &tl_parse::DerivedParseReport) -> bool {
    let bytes = serde_json::to_vec(report).unwrap();
    serde_json::from_slice::<tl_parse::DerivedParseReport>(&bytes).is_ok()
}

#[test]
fn derived_report_reader_checks_lowering_records_against_the_document() {
    use tl_parse::tl_syntax::NodeId;
    let good = derived("p0 W[0,1] p1");
    assert!(good.document.is_some() && good.lowerings.len() == 1);
    assert!(derived_ok(&good));
    let nodes = u32::try_from(good.document.as_ref().unwrap().nodes().len()).unwrap();

    // Each node a record names must lie inside the document: the last node is
    // fine, the first index past the end is refused, for every named node.
    let edits: [fn(&mut tl_parse::LoweringRecord, NodeId); 3] = [
        |record, id| record.left = id,
        |record, id| record.right = id,
        |record, id| record.root = id,
    ];
    for edit in edits {
        let mut inside = good.clone();
        edit(&mut inside.lowerings[0], NodeId(nodes - 1));
        assert!(derived_ok(&inside));
        let mut outside = good.clone();
        edit(&mut outside.lowerings[0], NodeId(nodes));
        assert!(!derived_ok(&outside));
    }

    // Lowering records need a document; a document needs a diagnostic-free parse.
    let mut orphan = good.clone();
    orphan.document = None;
    assert!(
        !derived_ok(&orphan),
        "lowerings without a document are refused"
    );
    orphan.lowerings.clear();
    orphan.diagnostics = closed("&").diagnostics;
    orphan.stats.diagnostics = orphan.diagnostics.len();
    assert!(
        derived_ok(&orphan),
        "a failed parse without lowerings is admitted"
    );
    let mut truncated = good.clone();
    truncated.stats.diagnostics_truncated = true;
    assert!(!derived_ok(&truncated));
    let mut with_diagnostic = good.clone();
    with_diagnostic.diagnostics = closed("&").diagnostics;
    assert!(!derived_ok(&with_diagnostic));
}

fn infinite_ok(report: &tl_parse::InfiniteParseReport) -> bool {
    let bytes = serde_json::to_vec(report).unwrap();
    tl_parse::InfiniteParseReport::from_json_bytes(&bytes, ParseArtifactLimits::default()).is_ok()
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn infinite_report_reader_admits_source_and_depth_at_their_limits() {
    let good = infinite("fair {F[0,)p0}: G[1,)p1");
    assert!(infinite_ok(&good));
    let mut source = good.clone();
    source.limits.max_source_bytes = good.stats.source_bytes;
    assert!(infinite_ok(&source));
    source.limits.max_source_bytes -= 1;
    assert!(!infinite_ok(&source));
    let mut depth = good.clone();
    depth.limits.max_depth = good.stats.max_depth;
    assert!(infinite_ok(&depth));
    depth.limits.max_depth -= 1;
    assert!(!infinite_ok(&depth));
}

/// Nodes the formatter visits for each row of `V4_ROWS`, in order.
const V4_NODE_COUNTS: [usize; 16] = [5, 5, 3, 3, 2, 3, 2, 2, 2, 3, 3, 9, 6, 9, 3, 5];

const V4_ROWS: [(&str, &str); 16] = [
    ("p0W[1,2]p1", "(p0 W[1,2] p1)"),
    ("p0 M[1,)p1", "(p0 M[1,) p1)"),
    ("p0U[0,3]p1", "(p0 U[0,3] p1)"),
    ("p0R[0,)p1", "(p0 R[0,) p1)"),
    ("F[0,)p0", "F[0,)p0"),
    ("G[2,5]!p0", "G[2,5]!p0"),
    ("O[1,2]p0", "O[1,2]p0"),
    ("H[0,)p0", "H[0,)p0"),
    ("Yp0", "Yp0"),
    ("p0S[0,2]p1", "(p0 S[0,2] p1)"),
    ("p0T[1,)p1", "(p0 T[1,) p1)"),
    ("p0&p1|p2->p3<->p4", "((((p0 & p1) | p2) -> p3) <-> p4)"),
    ("!(p0 W[0,3] p1)", "!(p0 W[0,3] p1)"),
    ("(p0 W[0,3] p1) W[1,2] p2", "((p0 W[0,3] p1) W[1,2] p2)"),
    ("true & false", "(true & false)"),
    ("p0 W[0,)p1", "(p0 W[0,) p1)"),
];

fn v4_format(source: &'static str, limits: tl_parse::FormatLimits) -> tl_parse::FormatReport {
    let parsed = tl_parse::parse_clean_ascii_v4(
        source,
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        ParseLimits::default(),
    );
    let document = parsed.document.expect("fixture parses");
    tl_parse::format_clean_ascii_v4(&document, parsed.fairness.as_ref(), limits)
}

#[test]
fn v4_formatter_renders_each_operator_family_canonically() {
    for ((source, expected), nodes) in V4_ROWS.into_iter().zip(V4_NODE_COUNTS) {
        let report = v4_format(source, tl_parse::FormatLimits::default());
        assert_eq!(report.stats.nodes, nodes, "{source}");
        assert_eq!(report.text.as_deref(), Some(expected), "{source}");
        assert_eq!(report.stats.output_bytes, expected.len(), "{source}");
        assert!(report.error.is_none(), "{source}");
    }
    // A derived weak-until written out by hand is the same graph as the shorthand.
    assert_eq!(
        v4_format("p0W[1,2]p1", tl_parse::FormatLimits::default()).text,
        v4_format("p0 W[1,2] p1", tl_parse::FormatLimits::default()).text
    );
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn v4_formatter_limits_admit_exactly_the_cost_of_a_format() {
    use tl_parse::{FormatErrorCode, FormatLimits};
    for (source, expected) in V4_ROWS {
        let full = v4_format(source, FormatLimits::default());
        let (work, bytes) = (full.stats.work, expected.len());
        let exact_work = v4_format(
            source,
            FormatLimits {
                max_work: work,
                ..FormatLimits::default()
            },
        );
        assert_eq!(
            exact_work.text.as_deref(),
            Some(expected),
            "{source} at work"
        );
        let short_work = v4_format(
            source,
            FormatLimits {
                max_work: work - 1,
                ..FormatLimits::default()
            },
        );
        assert_eq!(
            short_work.error.map(|error| error.code),
            Some(FormatErrorCode::WorkLimit),
            "{source} one below work"
        );
        let exact_bytes = v4_format(
            source,
            FormatLimits {
                max_output_bytes: bytes,
                ..FormatLimits::default()
            },
        );
        assert_eq!(
            exact_bytes.text.as_deref(),
            Some(expected),
            "{source} at bytes"
        );
        let short_bytes = v4_format(
            source,
            FormatLimits {
                max_output_bytes: bytes - 1,
                ..FormatLimits::default()
            },
        );
        assert_eq!(
            short_bytes.error.map(|error| error.code),
            Some(FormatErrorCode::OutputLimit),
            "{source} one below bytes"
        );
    }
}

fn v4_parse(source: &'static str, limits: ParseLimits) -> tl_parse::InfiniteParseReport {
    tl_parse::parse_clean_ascii_v4(
        source,
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        limits,
    )
}

// Trace: TC-067, NFR-005-AC-1
#[test]
fn v4_parser_limits_admit_exactly_the_cost_of_a_parse() {
    type LimitRow = (
        &'static str,
        usize,
        fn(&mut ParseLimits, usize),
        DiagnosticCode,
    );
    use tl_parse::DiagnosticCode;
    for (source, _) in V4_ROWS {
        let full = v4_parse(source, ParseLimits::default());
        assert!(full.document.is_some(), "{source}");
        let stats = full.stats;
        let at = |edit: fn(&mut ParseLimits, usize), value: usize| {
            let mut limits = ParseLimits::default();
            edit(&mut limits, value);
            v4_parse(source, limits)
        };
        let rows: [LimitRow; 4] = [
            (
                "work",
                stats.work,
                |l, v| l.max_work = v,
                DiagnosticCode::WorkLimit,
            ),
            (
                "nodes",
                stats.nodes,
                |l, v| l.max_nodes = v,
                DiagnosticCode::NodeLimit,
            ),
            (
                "tokens",
                stats.tokens,
                |l, v| l.max_tokens = v,
                DiagnosticCode::TokenLimit,
            ),
            (
                "depth",
                stats.max_depth,
                |l, v| l.max_depth = v,
                DiagnosticCode::DepthLimit,
            ),
        ];
        for (name, value, edit, code) in rows {
            let admitted = at(edit, value);
            assert!(
                admitted.document.is_some(),
                "{source}: {name} == {value} is admitted"
            );
            let refused = at(edit, value - 1);
            assert!(
                refused.document.is_none(),
                "{source}: {name} == {value} - 1 is refused"
            );
            assert!(
                refused.diagnostics.iter().any(|d| d.code == code),
                "{source}: {name}: {:?}",
                refused.diagnostics
            );
        }
        // The source-byte limit sits on the submitted length.
        let exact = at(|l, v| l.max_source_bytes = v, source.len());
        assert!(exact.document.is_some(), "{source}: source bytes at limit");
        let short = at(|l, v| l.max_source_bytes = v, source.len() - 1);
        assert!(short.document.is_none(), "{source}: source bytes one past");
    }
}

// Trace: TC-062, FR-016-AC-2
#[test]
fn infinite_refusal_keeps_the_first_cause_of_a_class_and_yields_to_a_resource_limit() {
    use tl_parse::DiagnosticCode as C;
    let refusal = |source: &str, max_work: usize| {
        let report = tl_parse::parse_clean_ascii_v4(
            source,
            SemanticProfile::InfiniteTraceV1,
            "event_position",
            ParseLimits {
                max_work,
                ..ParseLimits::default()
            },
        );
        let decisive = report.refusal.expect("the parse is refused");
        (decisive.code, decisive.span.start(), decisive.span.end())
    };
    let roomy = ParseLimits::default().max_work;
    // Two causes of the same class: the first one decides.
    assert_eq!(refusal("$ $", roomy), (C::UnexpectedCharacter, 0, 1));
    assert_eq!(refusal("F[01,2]p0 $", roomy), (C::NonCanonicalNumber, 2, 4));
    // A resource limit reached after a syntax cause replaces it.
    assert_eq!(refusal("$ p0&p1&p2&p3", 6), (C::WorkLimit, 8, 9));
    assert_eq!(refusal("$ p0&p1&p2&p3", 9), (C::UnexpectedCharacter, 0, 1));
}

type StatRow = (&'static str, usize, usize, usize, usize);

fn stat_row(source: &'static str, stats: tl_parse::ParseStats) -> StatRow {
    (
        source,
        stats.max_depth,
        stats.nodes,
        stats.tokens,
        stats.work,
    )
}

#[test]
fn parse_statistics_are_exact_for_each_dialect() {
    // (source, max depth, nodes, tokens, work): the deterministic observations
    // every limit is judged against.
    let v1: [StatRow; 12] = [
        ("p0", 1, 1, 1, 4),
        ("!p0", 2, 2, 2, 7),
        ("!!p0", 3, 3, 3, 10),
        ("(p0)", 2, 1, 3, 10),
        ("((p0))", 3, 1, 5, 16),
        ("p0&p1", 2, 3, 3, 10),
        ("p0&p1&p2", 2, 5, 5, 16),
        ("!(p0&!p1)", 5, 5, 7, 22),
        ("F[0,3]p0", 2, 2, 7, 17),
        ("G[1,2]F[0,3]p0", 3, 3, 13, 30),
        ("p0U[0,3]p1", 2, 3, 8, 20),
        ("p0R[0,3]p1", 2, 3, 8, 20),
    ];
    for row in v1 {
        assert_eq!(stat_row(row.0, closed(row.0).stats), row, "v1");
    }
    let v2: [StatRow; 3] = [
        ("p0 W[0,3] p1", 2, 5, 8, 24),
        ("p0 M[0,3] p1", 2, 5, 8, 24),
        ("F[0,2](p0 W[0,1] p1)", 4, 6, 16, 43),
    ];
    for row in v2 {
        assert_eq!(stat_row(row.0, derived(row.0).stats), row, "v2");
    }
    let v3: [StatRow; 3] = [
        ("Yp0", 2, 2, 2, 7),
        ("O[0,2]p0 S[0,1] p1", 2, 4, 14, 35),
        ("H[0,1]Yp0", 3, 3, 8, 20),
    ];
    for row in v3 {
        assert_eq!(stat_row(row.0, past(row.0).stats), row, "v3");
    }
    let v4: [StatRow; 5] = [
        ("p0", 1, 1, 1, 3),
        ("!!p0", 5, 3, 3, 9),
        ("p0S[0,1](p1|p2)", 5, 5, 12, 28),
        ("O[0,1]!p0", 5, 3, 8, 19),
        ("(p0 W[0,1] p1) M[1,2] p2", 4, 9, 17, 42),
    ];
    for row in v4 {
        assert_eq!(stat_row(row.0, infinite(row.0).stats), row, "v4");
    }
}

#[test]
fn infinite_diagnostics_quote_the_offending_text_or_name_the_end_of_input() {
    let quoted = infinite("p0 $");
    assert_eq!(quoted.diagnostics[0].found, "\"$\"");
    let at_end = infinite("(p0 &");
    assert_eq!(at_end.diagnostics[0].found, "<eof>");
}

#[test]
fn v4_formatter_refuses_fairness_premises_bound_to_another_graph() {
    use tl_parse::{FormatErrorCode, FormatLimits};
    let with_premises = infinite("fair {F[0,)p0}: G[1,)p1");
    let other = infinite("p0 & p1 & p2");
    let premises = with_premises.fairness.expect("premises parse");
    let report = tl_parse::format_clean_ascii_v4(
        &other.document.expect("graph parses"),
        Some(&premises),
        FormatLimits::default(),
    );
    assert!(report.text.is_none());
    let error = report
        .error
        .expect("a foreign fairness document is refused");
    assert_eq!(error.code, FormatErrorCode::InvalidGraph);
    assert_eq!(error.message, "foreign fairness document");
}

// Shapes for hand-built v4 graphs. Node ids are post-order, which is the only
// numbering the canonical text can reproduce.
#[derive(Clone, Copy)]
enum Shape4 {
    Not,
    StrongPrevious,
    Future,
    AndLeft,
    AndRight,
    OrLeft,
    UntilLeft,
    UntilRight,
}

fn push_leaf(nodes: &mut Vec<tl_parse::tl_syntax::InfiniteNode>) -> tl_parse::tl_syntax::NodeId {
    use tl_parse::tl_syntax::{InfiniteNode, InfiniteNodeKind, NodeId, PropositionId};
    nodes.push(InfiniteNode::new(InfiniteNodeKind::Proposition {
        proposition: PropositionId(0),
    }));
    NodeId(u32::try_from(nodes.len() - 1).unwrap())
}

/// Builds a spine of `length` nodes (the leaf included) of the given shape.
fn spine(
    nodes: &mut Vec<tl_parse::tl_syntax::InfiniteNode>,
    length: usize,
    shape: Shape4,
) -> tl_parse::tl_syntax::NodeId {
    use tl_parse::tl_syntax::{
        InfiniteNode, InfiniteNodeKind as K, NodeId, TemporalInterval, UnboundedInterval,
    };
    if length == 1 {
        return push_leaf(nodes);
    }
    let interval = TemporalInterval::Unbounded(UnboundedInterval::new(0));
    let (left, right) = match shape {
        Shape4::AndRight | Shape4::UntilRight => {
            let leaf = push_leaf(nodes);
            let rest = spine(nodes, length - 1, shape);
            (leaf, rest)
        }
        Shape4::AndLeft | Shape4::OrLeft | Shape4::UntilLeft => {
            let rest = spine(nodes, length - 1, shape);
            let leaf = push_leaf(nodes);
            (rest, leaf)
        }
        _ => {
            let rest = spine(nodes, length - 1, shape);
            (rest, rest)
        }
    };
    let kind = match shape {
        Shape4::Not => K::Not { operand: left },
        Shape4::StrongPrevious => K::StrongPrevious { operand: left },
        Shape4::Future => K::Future {
            interval,
            operand: left,
        },
        Shape4::AndLeft | Shape4::AndRight => K::And { left, right },
        Shape4::OrLeft => K::Or { left, right },
        Shape4::UntilLeft | Shape4::UntilRight => K::Until {
            interval,
            left,
            right,
        },
    };
    nodes.push(InfiniteNode::new(kind));
    NodeId(u32::try_from(nodes.len() - 1).unwrap())
}

fn spine_format(length: usize, shape: Shape4) -> tl_parse::FormatReport {
    use tl_parse::tl_syntax::{InfiniteClock, InfiniteFormulaDocument};
    let mut nodes = Vec::new();
    let root = spine(&mut nodes, length, shape);
    let document = InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        root,
        nodes,
    )
    .expect("a spine this long is within the owner graph limits");
    tl_parse::format_clean_ascii_v4(&document, None, tl_parse::FormatLimits::default())
}

// Trace: TC-069, NFR-005-AC-3
#[test]
fn v4_canonical_text_depth_is_counted_along_every_operator_shape() {
    use tl_parse::{FormatErrorCode, HARD_MAX_DEPTH};
    let shapes = [
        Shape4::Not,
        Shape4::StrongPrevious,
        Shape4::Future,
        Shape4::AndLeft,
        Shape4::AndRight,
        Shape4::OrLeft,
        Shape4::UntilLeft,
        Shape4::UntilRight,
    ];
    for shape in shapes {
        let at = spine_format(HARD_MAX_DEPTH, shape);
        assert!(at.error.is_none(), "{:?}", at.error);
        let past = spine_format(HARD_MAX_DEPTH + 1, shape);
        let error = past.error.expect("one level past the limit is refused");
        assert_eq!(error.code, FormatErrorCode::InvalidGraph);
        assert_eq!(error.message, "v4 graph exceeds format depth");
    }
}

#[test]
fn v4_formatter_does_not_read_a_look_alike_pair_as_a_derived_operator() {
    use tl_parse::tl_syntax::{
        InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind as K, NodeId,
        PropositionId, TemporalInterval, UnboundedInterval,
    };
    use tl_parse::FormatErrorCode;
    let interval = TemporalInterval::Unbounded(UnboundedInterval::new(0));
    let x = K::Proposition {
        proposition: PropositionId(0),
    };
    let y = K::Proposition {
        proposition: PropositionId(1),
    };
    // `x` is shared by both temporal operands, which plain text cannot spell
    // without changing node identity; only the exact derived layout can.
    let until = K::Until {
        interval,
        left: NodeId(0),
        right: NodeId(1),
    };
    let release = K::Release {
        interval,
        left: NodeId(0),
        right: NodeId(1),
    };
    let globally = K::Globally {
        interval,
        operand: NodeId(0),
    };
    let future = K::Future {
        interval,
        operand: NodeId(0),
    };
    let and = K::And {
        left: NodeId(2),
        right: NodeId(3),
    };
    let or = K::Or {
        left: NodeId(2),
        right: NodeId(3),
    };
    let rows = [
        ("and of release and globally", release, globally, and),
        ("and of until and future", until, future, and),
        ("or of release and globally", release, globally, or),
        ("or of until and future", until, future, or),
    ];
    let format = |kinds: Vec<K>| {
        let root = NodeId(u32::try_from(kinds.len() - 1).unwrap());
        let document = InfiniteFormulaDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            root,
            kinds.into_iter().map(InfiniteNode::new).collect(),
        )
        .unwrap();
        tl_parse::format_clean_ascii_v4(&document, None, tl_parse::FormatLimits::default())
    };
    for (name, first, second, root) in rows {
        let report = format(vec![x, y, first, second, root]);
        let error = report
            .error
            .unwrap_or_else(|| panic!("{name}: {:?}", report.text));
        assert_eq!(error.code, FormatErrorCode::UnrepresentableGraph, "{name}");
    }
    // The exact layout is still read as the derived operator.
    let weak = format(vec![
        x,
        y,
        K::Until {
            interval,
            left: NodeId(0),
            right: NodeId(1),
        },
        globally,
        or,
    ]);
    assert_eq!(weak.text.as_deref(), Some("(p0 W[0,) p1)"));
    let strong = format(vec![x, y, release, future, and]);
    assert_eq!(strong.text.as_deref(), Some("(p0 M[0,) p1)"));
}

type First = Option<(tl_parse::DiagnosticCode, u32, u32)>;

fn first(diagnostics: &[tl_parse::Diagnostic]) -> First {
    diagnostics
        .first()
        .map(|d| (d.code, d.span.start(), d.span.end()))
}

// Trace: TC-001, FR-001-AC-1
#[test]
fn a_keyword_is_one_word_unless_the_dialect_lets_an_operator_follow_it() {
    use tl_parse::DiagnosticCode::UnknownIdentifier as U;
    let some = |code, start, end| Some((code, start, end));
    // (source, first diagnostic under v1, v3, v4). v1 lets `U`/`R` and v3 lets
    // `S`/`T` follow a keyword when an interval opens next; v4 lets any of them.
    let rows: Vec<(&str, First, First, First)> = vec![
        ("true1", some(U, 0, 5), some(U, 0, 5), some(U, 0, 5)),
        ("falsex", some(U, 0, 6), some(U, 0, 6), some(U, 0, 6)),
        ("trueU p0", some(U, 0, 5), some(U, 0, 5), some(U, 0, 5)),
        ("falseR p0", some(U, 0, 6), some(U, 0, 6), some(U, 0, 6)),
        ("trueS p0", some(U, 0, 5), some(U, 0, 5), some(U, 0, 5)),
        ("trueW p0", some(U, 0, 5), some(U, 0, 5), some(U, 0, 5)),
        ("trueU[0,1]p0", None, some(U, 0, 5), None),
        ("trueS[0,1]p0", some(U, 0, 5), None, None),
        ("trueT[0,1]p0", some(U, 0, 5), None, None),
        ("falseW[0,1]p0", some(U, 0, 6), some(U, 0, 6), None),
        ("trueM[0,1]p0", some(U, 0, 5), some(U, 0, 5), None),
        ("true&p0", None, None, None),
    ];
    for (source, v1, v3, v4) in rows {
        assert_eq!(first(&closed(source).diagnostics), v1, "v1 {source}");
        assert_eq!(first(&past(source).diagnostics), v3, "v3 {source}");
        assert_eq!(first(&infinite(source).diagnostics), v4, "v4 {source}");
    }
}
