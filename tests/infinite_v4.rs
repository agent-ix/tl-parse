use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use tl_parse::tl_syntax::SemanticProfile;
use tl_parse::{
    format_clean_ascii_v4, parse_clean_ascii_v4, DiagnosticCode, FormatLimits, InfiniteDisposition,
    ParseArtifactLimits, ParseLimits, DIALECT_V4_REVISION,
};

const PROFILE: SemanticProfile = SemanticProfile::InfiniteTraceV1;

fn parse(source: &str) -> tl_parse::InfiniteParseReport {
    parse_clean_ascii_v4(source, PROFILE, "event_position", ParseLimits::default())
}

// Trace: TC-062, FR-016-AC-2, TC-064, FR-017-AC-2
#[test]
fn v4_report_strict_reader_rejects_identity_and_locus_mutations() {
    let report = parse("fair {F[0,)p0}: G[1,)p1");
    let bytes = serde_json::to_vec(&report).unwrap();
    assert_eq!(
        tl_parse::InfiniteParseReport::from_json_bytes(&bytes, ParseArtifactLimits::default())
            .unwrap(),
        report
    );

    let mut foreign = serde_json::to_value(&report).unwrap();
    foreign["clock"] = serde_json::json!("wall_clock");
    assert!(tl_parse::InfiniteParseReport::from_json_bytes(
        &serde_json::to_vec(&foreign).unwrap(),
        ParseArtifactLimits::default()
    )
    .is_err());

    let mut outside = serde_json::to_value(&report).unwrap();
    outside["premise_spans"][0]["end"] = serde_json::json!(1000);
    assert!(tl_parse::InfiniteParseReport::from_json_bytes(
        &serde_json::to_vec(&outside).unwrap(),
        ParseArtifactLimits::default()
    )
    .is_err());

    for (axis, mutate) in [
        (
            "compiled owner revision",
            Box::new(|wire: &mut serde_json::Value| {
                wire["tl_syntax_revision"] = serde_json::json!("stale-owner");
            }) as Box<dyn Fn(&mut serde_json::Value)>,
        ),
        (
            "work counter",
            Box::new(|wire: &mut serde_json::Value| {
                wire["stats"]["work"] = serde_json::json!(u64::MAX);
            }),
        ),
        (
            "node counter",
            Box::new(|wire: &mut serde_json::Value| {
                wire["stats"]["nodes"] = serde_json::json!(0);
            }),
        ),
        (
            "missing graph",
            Box::new(|wire: &mut serde_json::Value| {
                wire["document"] = serde_json::Value::Null;
                wire["fairness"] = serde_json::Value::Null;
            }),
        ),
        (
            "missing fairness binding",
            Box::new(|wire: &mut serde_json::Value| {
                wire["fairness"] = serde_json::Value::Null;
            }),
        ),
        (
            "foreign fairness graph",
            Box::new(|wire: &mut serde_json::Value| {
                wire["fairness"]["graph_identity"] = serde_json::json!("foreign-graph");
            }),
        ),
    ] {
        let mut wire = serde_json::to_value(&report).unwrap();
        mutate(&mut wire);
        assert!(
            tl_parse::InfiniteParseReport::from_json_bytes(
                &serde_json::to_vec(&wire).unwrap(),
                ParseArtifactLimits::default()
            )
            .is_err(),
            "{axis}"
        );
    }
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
    let again = format_clean_ascii_v4(
        second.document.as_ref().unwrap(),
        second.fairness.as_ref(),
        FormatLimits::default(),
    );
    assert_eq!(again.text.as_deref(), Some(text.as_str()));
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
