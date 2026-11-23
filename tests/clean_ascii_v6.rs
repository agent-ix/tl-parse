//! `tl-parse.clean-ascii/v6` bracket-free LTL and LTLf (TL-429). UNBUILT, UNTESTED.
//!
//! The specification (FR-058..FR-062, TM-043) calls this dialect `v5`; the TL
//! program decision of 2026-11-23 renumbered it to `v6`.

use std::{fs, marker::PhantomData, path::PathBuf};

use serde::Deserialize;
use tl_parse::{
    format_clean_ascii_v6, parse, parse_clean_ascii_v2, parse_clean_ascii_v3,
    parse_clean_ascii_v4, parse_clean_ascii_v6, parse_clean_ascii_v6_with_profile_identity,
    FormatLimits, LtlDiagnosticCode, LtlDocument, LtlFormatErrorCode, LtlParseReport, LtlProfile,
    ParseArtifactLimits, ParseLimits, DIALECT_V6_REVISION,
};
use tl_syntax::{
    InfiniteClock, InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind, Interval, NodeId,
    PropositionId, SemanticProfile, TemporalInterval, UnboundedInterval,
};

const LTLF: LtlProfile = LtlProfile::LtlfFiniteTrace;
const INFINITE: LtlProfile = LtlProfile::InfiniteTrace;

fn v6(source: &str, profile: LtlProfile) -> LtlParseReport {
    parse_clean_ascii_v6(source, profile, ParseLimits::default())
}

fn text(report: &LtlParseReport) -> String {
    format_clean_ascii_v6(
        report.document.as_ref().expect("graph"),
        report.profile,
        FormatLimits::default(),
    )
    .text
    .expect("canonical text")
}

fn code(report: &LtlParseReport) -> Option<LtlDiagnosticCode> {
    report.refusal.map(|refusal| refusal.code)
}

fn span(report: &LtlParseReport) -> (u32, u32) {
    let refusal = report.refusal.expect("refusal");
    (refusal.span.start(), refusal.span.end())
}

const LEAVES: [&str; 4] = ["p0", "p1", "true", "false"];
const BINARY: [&str; 6] = ["U", "R", "&", "|", "->", "<->"];

/// The exhaustive depth-one population: 4 leaves, every unary operator over
/// every leaf, every binary operator over every leaf pair. 120 for LTLf
/// (unary `! F G Xs Xw`), 116 for the infinite profile (unary `! F G X`).
fn population(profile: LtlProfile) -> Vec<(String, String, usize)> {
    let unary: &[&str] = match profile {
        LtlProfile::LtlfFiniteTrace => &["!", "F", "G", "Xs", "Xw"],
        LtlProfile::InfiniteTrace => &["!", "F", "G", "X"],
    };
    let mut cases = Vec::new();
    for leaf in LEAVES {
        cases.push((leaf.to_owned(), leaf.to_owned(), 1));
    }
    for operator in unary {
        for leaf in LEAVES {
            let canonical = if *operator == "!" {
                format!("!{leaf}")
            } else {
                format!("{operator} {leaf}")
            };
            // Compact spellings exercise maximal munch (`Gp0`, `Xsp0`).
            cases.push((format!("{operator}{leaf}"), canonical, 2));
        }
    }
    for operator in BINARY {
        for left in LEAVES {
            for right in LEAVES {
                cases.push((
                    format!("{left}  {operator}\t{right}"),
                    format!("{left} {operator} {right}"),
                    3,
                ));
            }
        }
    }
    cases
}

fn node_count(report: &LtlParseReport) -> usize {
    report.stats.nodes
}

// ------------------------------------------------------------------ FR-058

// Trace: TC-112, FR-058-AC-1
#[test]
fn the_depth_one_population_parses_under_its_profile() {
    assert_eq!(population(LTLF).len(), 120);
    assert_eq!(population(INFINITE).len(), 116);
    for profile in LtlProfile::ALL {
        for (source, canonical, nodes) in population(profile) {
            let report = v6(&source, profile);
            assert!(report.document.is_some(), "{source}: {:?}", report.diagnostics);
            assert_eq!(report.profile, profile);
            assert_eq!(report.document.as_ref().unwrap().profile(), profile);
            assert_eq!(node_count(&report), nodes, "{source}");
            assert_eq!(text(&report), canonical, "{source}");
        }
    }
    // Under the infinite profile F is F[0,) and X is F[1,1].
    let Some(LtlDocument::Infinite(document)) = v6("F p0", INFINITE).document else {
        panic!("infinite edition");
    };
    assert_eq!(
        document.nodes()[1].kind,
        InfiniteNodeKind::Future {
            interval: TemporalInterval::Unbounded(UnboundedInterval::new(0)),
            operand: NodeId(0),
        }
    );
    let Some(LtlDocument::Infinite(document)) = v6("X p0", INFINITE).document else {
        panic!("infinite edition");
    };
    assert_eq!(
        document.nodes()[1].kind,
        InfiniteNodeKind::Future {
            interval: TemporalInterval::Closed(Interval::new(1, 1).unwrap()),
            operand: NodeId(0),
        }
    );
}

// Trace: TC-113, FR-058-AC-2
#[test]
fn intervals_and_out_of_dialect_operators_have_their_own_codes() {
    for profile in LtlProfile::ALL {
        for (source, start, end) in [
            ("F[0,2] p0", 1, 6),
            ("G[0,) p0", 1, 5),
            ("p0 U[1,3] p1", 4, 9),
            ("p0 R[0,5] p1", 4, 9),
        ] {
            let report = v6(source, profile);
            assert!(report.document.is_none());
            assert_eq!(code(&report), Some(LtlDiagnosticCode::IntervalNotInDialect), "{source}");
            assert_eq!(span(&report), (start, end), "{source}");
            assert!(report.diagnostics[0].message.contains("v1 to v4"));
        }
        for (source, start) in [
            ("p0 W p1", 3),
            ("p0 M p1", 3),
            ("O p0", 0),
            ("H p0", 0),
            ("Y p0", 0),
            ("p0 S p1", 3),
            ("p0 T p1", 3),
        ] {
            let report = v6(source, profile);
            assert_eq!(code(&report), Some(LtlDiagnosticCode::UnsupportedOperator), "{source}");
            assert_eq!(span(&report), (start, start + 1), "{source}");
        }
    }
}

// ------------------------------------------------------------------ FR-059

// Trace: TC-114, FR-059-AC-1, StR-044-VC-1
#[test]
fn one_text_two_profiles_two_graphs() {
    let ltlf = v6("F p0", LTLF);
    let infinite = v6("F p0", INFINITE);
    match (ltlf.document.as_ref(), infinite.document.as_ref()) {
        (Some(LtlDocument::Ltlf(first)), Some(LtlDocument::Infinite(second))) => {
            assert_eq!(first.semantic_profile(), SemanticProfile::LtlfFiniteTraceV1);
            assert_eq!(second.semantic_profile(), SemanticProfile::InfiniteTraceV1);
        }
        other => panic!("unexpected editions {other:?}"),
    }
    assert_eq!(ltlf.profile, LTLF);
    assert_eq!(infinite.profile, INFINITE);
}

// Trace: TC-115, FR-059-AC-2
#[test]
fn bare_next_under_ltlf_names_both_strengths() {
    let report = v6("X p0", LTLF);
    assert!(report.document.is_none());
    assert_eq!(code(&report), Some(LtlDiagnosticCode::NextStrengthRequired));
    assert_eq!(span(&report), (0, 1));
    let message = &report.diagnostics[0].message;
    assert!(message.contains("Xs") && message.contains("Xw"), "{message}");
}

// Trace: TC-116, FR-059-AC-2
#[test]
fn strong_and_weak_next_are_refused_under_the_infinite_profile() {
    for source in ["Xs p0", "Xw p0"] {
        let report = v6(source, INFINITE);
        assert!(report.document.is_none());
        assert_eq!(code(&report), Some(LtlDiagnosticCode::NextStrengthUndefined));
        assert_eq!(span(&report), (0, 2));
        assert!(v6(source, LTLF).document.is_some());
    }
}

trait NoImpl {
    const DEFAULT: bool = false;
    const FROM_STR: bool = false;
    const PARSE: bool = false;
}
impl<T> NoImpl for Probe<T> {}
struct Probe<T>(PhantomData<T>);
impl<T: Default> Probe<T> {
    const DEFAULT: bool = true;
}
impl<T: for<'a> From<&'a str>> Probe<T> {
    const FROM_STR: bool = true;
}
impl<T: std::str::FromStr> Probe<T> {
    const PARSE: bool = true;
}

// Trace: TC-117, FR-059-AC-3, StR-044-VC-1
#[test]
fn no_v6_entry_point_lacks_a_profile_and_the_profile_has_no_default() {
    // Inherent consts shadow the trait fallbacks only when the bound holds.
    assert!(!Probe::<LtlProfile>::DEFAULT);
    assert!(!Probe::<LtlProfile>::FROM_STR);
    assert!(!Probe::<LtlProfile>::PARSE);
    let _parse: fn(&str, LtlProfile, ParseLimits) -> LtlParseReport = parse_clean_ascii_v6;
    let _format: fn(&LtlDocument, LtlProfile, FormatLimits) -> tl_parse::LtlFormatReport =
        format_clean_ascii_v6;
    // Every public v6 function names its profile (typed or explicit identity).
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/ltl");
    for file in ["mod.rs", "format.rs"] {
        let source = fs::read_to_string(root.join(file)).unwrap();
        let mut rest = source.as_str();
        while let Some(at) = rest.find("pub fn ") {
            let signature_end = rest[at..].find('{').map_or(rest.len(), |end| at + end);
            let signature = &rest[at..signature_end];
            let takes_profile = signature.contains("profile: LtlProfile")
                || signature.contains("profile_identity: &str")
                || signature.contains("&self");
            assert!(takes_profile, "{file}: {signature}");
            rest = &rest[signature_end..];
        }
    }
    // The identity-string selector refuses anything but the two v6 identities.
    for identity in ["", "ltlf", "mltl.closed-trace/v1", "mltl.ltlf-finite-trace/v2"] {
        let refused =
            parse_clean_ascii_v6_with_profile_identity("p0", identity, ParseLimits::default())
                .unwrap_err();
        assert_eq!(refused.code, LtlDiagnosticCode::ProfileMismatch);
    }
    for profile in LtlProfile::ALL {
        let report = parse_clean_ascii_v6_with_profile_identity(
            "F p0",
            profile.identity(),
            ParseLimits::default(),
        )
        .unwrap();
        assert_eq!(report.profile, profile);
    }
}

// ------------------------------------------------------------------ FR-060

const PRECEDENCE: [(&str, &str); 10] = [
    ("p0 & p1 | p2", "p0 & p1 | p2"),
    ("p0 & (p1 | p2)", "p0 & (p1 | p2)"),
    ("p0 -> p1 -> p2", "p0 -> p1 -> p2"),
    ("(p0 -> p1) -> p2", "(p0 -> p1) -> p2"),
    ("p0 <-> p1 <-> p2", "p0 <-> p1 <-> p2"),
    ("p0 <-> (p1 <-> p2)", "p0 <-> (p1 <-> p2)"),
    ("p0 U p1 U p2", "p0 U p1 U p2"),
    ("p0 U (p1 R p2)", "p0 U (p1 R p2)"),
    ("G(p0 -> F p1)", "G (p0 -> F p1)"),
    ("!G p0 & F !p1 U p2", "!G p0 & F !p1 U p2"),
];

fn kinds(report: &LtlParseReport) -> String {
    // Graph identity without spans, via the canonical serialization of the
    // owner document with spans stripped by re-parsing canonical text.
    format!("{:?}", report.document.as_ref().map(LtlDocument::profile))
        + &text(report)
}

// Trace: TC-118, FR-060-AC-1
#[test]
fn every_population_and_precedence_formula_round_trips_to_a_fixed_point() {
    let mut checked = 0;
    for profile in LtlProfile::ALL {
        let mut sources: Vec<(String, String)> = population(profile)
            .into_iter()
            .map(|(source, canonical, _)| (source, canonical))
            .collect();
        sources.extend(
            PRECEDENCE
                .iter()
                .map(|(source, canonical)| ((*source).to_owned(), (*canonical).to_owned())),
        );
        for (source, canonical) in sources {
            let first = v6(&source, profile);
            let formatted = text(&first);
            assert_eq!(formatted, canonical, "{source}");
            let second = v6(&formatted, profile);
            // Canonical text re-parses to itself, so its graph (spans included)
            // is a fixed point.
            assert_eq!(text(&second), formatted, "{source}");
            assert_eq!(v6(&formatted, profile).document, second.document);
            assert_eq!(kinds(&first), kinds(&second));
            checked += 1;
        }
    }
    assert_eq!(checked, 236 + 2 * PRECEDENCE.len());
}

fn infinite_document(kind: InfiniteNodeKind) -> LtlDocument {
    LtlDocument::Infinite(
        InfiniteFormulaDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            NodeId(1),
            vec![
                InfiniteNode::new(InfiniteNodeKind::Proposition {
                    proposition: PropositionId(0),
                }),
                InfiniteNode::new(kind),
            ],
        )
        .unwrap(),
    )
}

// Trace: TC-119, FR-060-AC-2
#[test]
fn graphs_outside_the_image_are_not_expressible() {
    let closed = TemporalInterval::Closed(Interval::new(0, 3).unwrap());
    let one = TemporalInterval::Closed(Interval::new(1, 1).unwrap());
    let late = TemporalInterval::Unbounded(UnboundedInterval::new(2));
    for kind in [
        InfiniteNodeKind::Future {
            interval: closed,
            operand: NodeId(0),
        },
        InfiniteNodeKind::Globally {
            interval: one,
            operand: NodeId(0),
        },
        InfiniteNodeKind::Globally {
            interval: late,
            operand: NodeId(0),
        },
        InfiniteNodeKind::StrongPrevious { operand: NodeId(0) },
    ] {
        let report =
            format_clean_ascii_v6(&infinite_document(kind), INFINITE, FormatLimits::default());
        assert_eq!(report.text, None, "{kind:?}");
        assert_eq!(
            report.error.map(|error| error.code),
            Some(LtlFormatErrorCode::NotExpressible),
            "{kind:?}"
        );
    }
    // `Xs` under the infinite profile: an LTLf graph formatted as infinite.
    let strong = v6("Xs p0", LTLF);
    let report = format_clean_ascii_v6(
        strong.document.as_ref().unwrap(),
        INFINITE,
        FormatLimits::default(),
    );
    assert_eq!(
        report.error.map(|error| error.code),
        Some(LtlFormatErrorCode::NotExpressible)
    );
}

// ------------------------------------------------------------------ FR-061

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    revision: String,
    family: String,
    dialect: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Limits {
    max_tokens: Option<usize>,
    max_source_bytes: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    canonical: Option<String>,
    code: Option<String>,
    span: Option<[u32; 2]>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    id: String,
    class: String,
    path: String,
    limits: Option<Limits>,
    ltlf: Outcome,
    infinite: Outcome,
    reason: String,
}

fn corpus() -> (PathBuf, Manifest) {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/v6");
    let manifest = serde_json::from_slice(&fs::read(directory.join("manifest.json")).unwrap())
        .unwrap();
    (directory, manifest)
}

fn case_limits(case: &Case) -> ParseLimits {
    let defaults = ParseLimits::default();
    match case.limits.as_ref() {
        None => defaults,
        Some(limits) => ParseLimits {
            max_tokens: limits.max_tokens.unwrap_or(defaults.max_tokens),
            max_source_bytes: limits.max_source_bytes.unwrap_or(defaults.max_source_bytes),
            ..defaults
        },
    }
}

/// Replays one case under one profile; returns whether it parsed.
fn replay(directory: &PathBuf, case: &Case, profile: LtlProfile) -> (bool, String) {
    let source = fs::read_to_string(directory.join(&case.path)).unwrap();
    let outcome = match profile {
        LtlProfile::LtlfFiniteTrace => &case.ltlf,
        LtlProfile::InfiniteTrace => &case.infinite,
    };
    let report = parse_clean_ascii_v6(&source, profile, case_limits(case));
    assert!(!case.reason.is_empty());
    match (&outcome.canonical, &outcome.code) {
        (Some(canonical), None) => {
            assert_eq!(&text(&report), canonical, "{} {profile:?}", case.id);
        }
        (None, Some(expected)) => {
            assert!(report.document.is_none(), "{} {profile:?}", case.id);
            assert_eq!(
                code(&report).map(LtlDiagnosticCode::as_str),
                Some(expected.as_str()),
                "{} {profile:?}",
                case.id
            );
            let [start, end] = outcome.span.expect("span");
            assert_eq!(span(&report), (start, end), "{} {profile:?}", case.id);
        }
        _ => panic!("{} has no single outcome under {profile:?}", case.id),
    }
    (report.document.is_some(), report.to_json().unwrap())
}

// Trace: TC-120, TC-122, FR-061-AC-1
#[test]
fn the_hostile_corpus_matches_its_manifest_under_each_profile() {
    let (directory, manifest) = corpus();
    assert_eq!(manifest.schema_version, "tl-parse.corpus/v1");
    assert_eq!(manifest.revision, tl_parse::CORPUS_REVISION);
    assert_eq!(manifest.family, "v6");
    assert_eq!(manifest.dialect, DIALECT_V6_REVISION);
    assert_eq!(manifest.cases.len(), 29);
    for profile in LtlProfile::ALL {
        let parsed = manifest
            .cases
            .iter()
            .filter(|case| replay(&directory, case, profile).0)
            .count();
        // 3 exact-limit cases parse under both profiles; next-strength cases
        // parse under the profile that admits them (SPEC-GAP in PLAN-045).
        let expected = match profile {
            LtlProfile::LtlfFiniteTrace => 5,
            LtlProfile::InfiniteTrace => 4,
        };
        assert_eq!(parsed, expected, "{profile:?}");
    }
    let classes: std::collections::BTreeSet<&str> =
        manifest.cases.iter().map(|case| case.class.as_str()).collect();
    assert_eq!(classes.len(), 14);
}

// Trace: TC-121, FR-061-AC-2
#[test]
fn the_fuzz_target_and_its_twenty_nine_seeds_are_checked_in() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join("fuzz/fuzz_targets/clean_ascii_v6.rs").is_file());
    let seeds = fs::read_dir(root.join("fuzz/corpus/clean_ascii_v6"))
        .unwrap()
        .count();
    assert_eq!(seeds, 29);
}

// Trace: TC-122, FR-061-AC-1
#[test]
fn the_report_reader_is_strict() {
    let report = v6("G (p0 -> F p1)", INFINITE);
    let bytes = report.to_json().unwrap().into_bytes();
    let limits = ParseArtifactLimits::default();
    assert_eq!(LtlParseReport::from_json_bytes(&bytes, limits).unwrap(), report);
    let text = String::from_utf8(bytes).unwrap();
    let unknown = text.replacen('{', "{\"extra\":0,", 1);
    assert!(LtlParseReport::from_json_bytes(unknown.as_bytes(), limits).is_err());
    let flipped = text.replace("mltl.infinite-trace/v1\",\"limits", "mltl.ltlf-finite-trace/v1\",\"limits");
    assert!(LtlParseReport::from_json_bytes(flipped.as_bytes(), limits).is_err());
}

// ------------------------------------------------------------------ FR-062

// Trace: TC-123, FR-062-AC-1, StR-044-VC-2
#[test]
fn dialect_identities_v1_to_v4_are_unchanged() {
    // Byte identity of v1..v4 records, documents and corpora: this change edits
    // none of them, and their existing suites (corpus, format,
    // versioned_dialects, clean_ascii_v2/v3, infinite_v4) are unchanged.
    // Decision 7 forbids registering new digest pins, so no digest literal is
    // added here.
    assert_eq!(tl_parse::DIALECT_REVISION, "tl-parse.clean-ascii/v1");
    assert_eq!(tl_parse::DIALECT_V2_REVISION, "tl-parse.clean-ascii/v2");
    assert_eq!(tl_parse::DIALECT_V3_REVISION, "tl-parse.clean-ascii/v3");
    assert_eq!(tl_parse::DIALECT_V4_REVISION, "tl-parse.clean-ascii/v4");
    assert_ne!(DIALECT_V6_REVISION, tl_parse::DIALECT_V5_REVISION);
}

// Trace: TC-124, FR-062-AC-2
#[test]
fn dialects_v1_to_v4_still_refuse_bracket_free_temporal_text() {
    let limits = ParseLimits::default();
    for source in ["G p0", "p0 U p1"] {
        for profile in [SemanticProfile::ClosedTraceV1, SemanticProfile::OnlinePrefixV1] {
            assert!(parse(source, profile, limits).document.is_none(), "v1 {source}");
            assert!(
                parse_clean_ascii_v2(source, profile, limits).document.is_none(),
                "v2 {source}"
            );
        }
        assert!(parse_clean_ascii_v3(source, limits).document.is_none(), "v3 {source}");
        assert!(
            parse_clean_ascii_v4(source, SemanticProfile::InfiniteTraceV1, "event_position", limits)
                .document
                .is_none(),
            "v4 {source}"
        );
    }
}

// ------------------------------------------------------------------ NFR-046

// Trace: TC-125, NFR-046-AC-1
#[test]
fn each_ceiling_accepts_at_its_value_and_refuses_one_past_it() {
    let base = ParseLimits::default();
    for profile in LtlProfile::ALL {
        let at = |source: &str, limits: ParseLimits| parse_clean_ascii_v6(source, profile, limits);
        let source = ParseLimits {
            max_source_bytes: 5,
            ..base
        };
        assert!(at("p0&p1", source).document.is_some());
        assert_eq!(code(&at("p0&p11", source)), Some(LtlDiagnosticCode::SourceLimit));
        let tokens = ParseLimits {
            max_tokens: 3,
            ..base
        };
        assert!(at("p0&p1", tokens).document.is_some());
        assert_eq!(code(&at("!p0&p1", tokens)), Some(LtlDiagnosticCode::TokenLimit));
        let nodes = ParseLimits {
            max_nodes: 3,
            ..base
        };
        assert!(at("p0&p1", nodes).document.is_some());
        assert_eq!(code(&at("!p0&p1", nodes)), Some(LtlDiagnosticCode::NodeLimit));
        let depth = ParseLimits {
            max_depth: 4,
            ..base
        };
        assert!(at("(((p0)))", depth).document.is_some());
        assert_eq!(code(&at("((((p0))))", depth)), Some(LtlDiagnosticCode::DepthLimit));
        let diagnostics = ParseLimits {
            max_diagnostics: 2,
            ..base
        };
        let two = at("p0 # #", diagnostics);
        assert_eq!(two.diagnostics.len(), 2);
        assert!(!two.stats.diagnostics_truncated);
        let three = at("p0 # # #", diagnostics);
        assert_eq!(three.diagnostics.len(), 2);
        assert!(three.stats.diagnostics_truncated);
        let output = FormatLimits {
            max_output_bytes: 7,
            ..FormatLimits::default()
        };
        let report = at("p0 & p1", base);
        let document = report.document.as_ref().unwrap();
        assert!(format_clean_ascii_v6(document, profile, output).text.is_some());
        let smaller = FormatLimits {
            max_output_bytes: 6,
            ..output
        };
        assert_eq!(
            format_clean_ascii_v6(document, profile, smaller)
                .error
                .map(|error| error.code),
            Some(LtlFormatErrorCode::OutputLimit)
        );
    }
}

// Trace: TC-126, NFR-046-AC-2
#[test]
fn two_runs_produce_byte_equal_reports() {
    let (directory, manifest) = corpus();
    for profile in LtlProfile::ALL {
        for case in &manifest.cases {
            assert_eq!(
                replay(&directory, case, profile).1,
                replay(&directory, case, profile).1,
                "{}",
                case.id
            );
        }
        for (source, _, _) in population(profile) {
            assert_eq!(
                v6(&source, profile).to_json().unwrap(),
                v6(&source, profile).to_json().unwrap()
            );
        }
    }
}
