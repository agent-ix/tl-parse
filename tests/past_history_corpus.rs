use std::{collections::BTreeMap, fs, path::Path};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_parse::{
    format_clean_ascii_v3, parse, parse_clean_ascii_v2, parse_clean_ascii_v3, FormatLimits,
    ParseLimits,
};
use tl_syntax::{FormulaDocument, SemanticProfile};

const DIRECTORY: &str = "corpus/past-history";
const MANIFEST_SHA256: &str = "59b86e7c888bf850cdd4e49cf86b01ffb64a99887d7fd051dca6a7d9f0a56393";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    corpus: String,
    revision: u64,
    role: String,
    formula_schema: String,
    operator_profile: String,
    semantic_profile: String,
    history_schema: String,
    dialect: String,
    implementation_revisions: Revisions,
    files: Vec<Pin>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Revisions {
    tl_syntax: String,
    tl_parse: String,
    tl_mltl: String,
    tl_rewrite: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pin {
    path: String,
    sha256: String,
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

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn load() -> (Manifest, Cases) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIRECTORY);
    let manifest_bytes = fs::read(root.join("manifest.json")).unwrap();
    assert_eq!(digest(&manifest_bytes), MANIFEST_SHA256);
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes).unwrap();
    let pins: BTreeMap<_, _> = manifest
        .files
        .iter()
        .map(|pin| (&pin.path, &pin.sha256))
        .collect();
    assert_eq!(pins.len(), 3);
    for pin in &manifest.files {
        assert_eq!(
            digest(&fs::read(root.join(&pin.path)).unwrap()),
            pin.sha256,
            "{}",
            pin.path
        );
    }
    let cases = serde_json::from_slice(&fs::read(root.join("cases.json")).unwrap()).unwrap();
    (manifest, cases)
}

// Trace: TC-056, FR-013-AC-2, FR-013-AC-3
#[test]
fn exact_shared_corpus_replays_through_clean_ascii_v3() {
    let (manifest, cases) = load();
    assert_eq!(manifest.corpus, "tl-syntax.past-history-corpus/v1");
    assert_eq!(manifest.revision, 1);
    assert_eq!(manifest.role, "evidence-input");
    assert_eq!(manifest.formula_schema, "tl-syntax.formula/v2");
    assert_eq!(manifest.operator_profile, "tl-syntax.past-operators/v1");
    assert_eq!(manifest.semantic_profile, "mltl.origin-complete-history/v1");
    assert_eq!(manifest.history_schema, "tl-mltl.position-history/v1");
    assert_eq!(manifest.dialect, "tl-parse.clean-ascii/v3");
    assert_eq!(
        manifest.implementation_revisions.tl_syntax,
        "e70f2379a752117c79603bc399a86c26feed7716"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_parse,
        "f82b0c724675c0f774415aa696c360959da30481"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_mltl,
        "b346cd0902794633e862f644a5575fc9776c34fb"
    );
    assert_eq!(
        manifest.implementation_revisions.tl_rewrite,
        "22b9cadcb1692cec8d3a97768f4f3b38fc654a5e"
    );
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
fn manifest_or_source_mutation_breaks_exact_replay() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(DIRECTORY);
    let mut manifest = fs::read(root.join("manifest.json")).unwrap();
    manifest[0] ^= 1;
    assert_ne!(digest(&manifest), MANIFEST_SHA256);

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
