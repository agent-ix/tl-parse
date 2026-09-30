use std::{fs, path::Path};

use serde::Deserialize;
use tl_parse::{
    format_clean_ascii_v3, parse, parse_clean_ascii_v2, parse_clean_ascii_v3, FormatLimits,
    ParseLimits,
};
use tl_syntax::{FormulaDocument, SemanticProfile};

const DIRECTORY: &str = "past-history";

#[derive(Deserialize)]
struct Manifest {
    corpus: String,
    revision: u64,
    role: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    history_schema: String,
    dialect: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cases {
    corpus: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    dialect: String,
    formulas: Vec<FormulaCase>,
    histories: Vec<serde_json::Value>,
    evaluations: Vec<serde_json::Value>,
    rewrites: Vec<serde_json::Value>,
    refusals: Vec<serde_json::Value>,
    target_dispositions: Vec<serde_json::Value>,
    mutation_axes: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FormulaCase {
    id: String,
    source: String,
    required_history: u64,
    document: FormulaDocument,
}

fn load() -> (Manifest, Cases) {
    // Read through the compiled tl-syntax dependency via `tl_syntax::CORPUS_DIR`,
    // not an in-repo path — TL-204 deletes `corpus/past-history` (5 files), which
    // was a byte-identical copy of tl-syntax's own corpus/past-history.
    let root = Path::new(tl_syntax::CORPUS_DIR).join(DIRECTORY);
    let manifest = serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let cases = serde_json::from_slice(&fs::read(root.join("cases.json")).unwrap()).unwrap();
    (manifest, cases)
}

// Trace: TC-056, FR-013-AC-2, FR-013-AC-3
#[test]
fn shared_corpus_replays_through_clean_ascii_v3() {
    let (manifest, cases) = load();
    assert_eq!(manifest.corpus, "tl-syntax.past-history-corpus/v1");
    assert_eq!(manifest.revision, 1);
    assert_eq!(manifest.role, "evidence-input");
    assert_eq!(manifest.formula_schema, "tl-syntax.formula/v2");
    assert_eq!(manifest.operator_profile, "tl-syntax.past-operators/v1");
    assert_eq!(manifest.semantic_profile, "mltl.origin-complete-history/v1");
    assert_eq!(manifest.history_schema, "tl-mltl.position-history/v1");
    assert_eq!(manifest.dialect, "tl-parse.clean-ascii/v3");
    assert_eq!(cases.corpus, manifest.corpus);
    assert_eq!(cases.formula_schema, manifest.formula_schema);
    assert_eq!(cases.operator_profile, manifest.operator_profile);
    assert_eq!(cases.semantic_profile, manifest.semantic_profile);
    assert_eq!(cases.dialect, manifest.dialect);
    assert_eq!(cases.formulas.len(), 8);
    assert_eq!(cases.histories.len(), 3);
    assert_eq!(cases.evaluations.len(), 9);
    assert_eq!(cases.rewrites.len(), 3);
    assert_eq!(cases.refusals.len(), 12);
    assert_eq!(cases.target_dispositions.len(), 4);
    assert_eq!(cases.mutation_axes.len(), 10);

    for case in cases.formulas {
        assert!(!case.id.is_empty());
        assert!(case.required_history <= u64::from(u32::MAX));
        let report = parse_clean_ascii_v3(&case.source, ParseLimits::default());
        assert!(
            report.diagnostics.is_empty(),
            "{}: {:?}",
            case.id,
            report.diagnostics
        );
        let observed = report.document.unwrap();
        assert_eq!(
            serde_json::to_value(observed.semantic_view()).unwrap(),
            serde_json::to_value(case.document.semantic_view()).unwrap(),
            "{}",
            case.id
        );
        let formatted = format_clean_ascii_v3(&observed, FormatLimits::default());
        assert_eq!(
            formatted.text.as_deref(),
            Some(case.source.as_str()),
            "{}",
            case.id
        );

        let v1 = parse(
            &case.source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert!(v1.document.is_none(), "v1 admitted {}", case.id);
        let v2 = parse_clean_ascii_v2(
            &case.source,
            SemanticProfile::ClosedTraceV1,
            ParseLimits::default(),
        );
        assert!(v2.document.is_none(), "v2 admitted {}", case.id);
    }
}

// Trace: TC-056, FR-013-AC-2, FR-013-AC-3
#[test]
fn source_mutation_breaks_exact_replay() {
    let (_, cases) = load();
    let case = &cases.formulas[0];
    let changed = format!("{} ", case.source);
    let parsed = parse_clean_ascii_v3(&changed, ParseLimits::default())
        .document
        .unwrap();
    assert_ne!(
        format_clean_ascii_v3(&parsed, FormatLimits::default())
            .text
            .as_deref(),
        Some(changed.as_str())
    );
}
