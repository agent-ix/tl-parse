//! Replay the owner's one corpus through the v4 parser boundary in place.

use std::{fs, path::Path};

use serde_json::Value;
use sha2::{Digest, Sha256};
use tl_parse::tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, NodeId, SemanticProfile,
    CORPUS_DIR,
};
use tl_parse::{
    format_clean_ascii_v4, parse_clean_ascii_v4, FormatErrorCode, FormatLimits, ParseLimits,
};

// Trace: TC-063, FR-017-AC-1, TC-064, FR-017-AC-2
#[test]
fn pinned_owner_corpus_round_trips_or_refuses_unrepresentable_topology() {
    let directory = Path::new(CORPUS_DIR).join("infinite-trace");
    let manifest: Value =
        serde_json::from_slice(&fs::read(directory.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["corpus"], "tl-syntax.infinite-trace-corpus/v1");
    for pin in manifest["files"].as_array().unwrap() {
        let path = pin["path"].as_str().unwrap();
        let bytes = fs::read(directory.join(path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            pin["sha256"],
            "{path}"
        );
    }
    let cases: Value =
        serde_json::from_slice(&fs::read(directory.join("cases.json")).unwrap()).unwrap();
    assert_eq!(cases["cases"].as_array().unwrap().len(), 13);
    let mut admitted = 0;
    let mut unrepresentable = 0;
    for case in cases["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let formula: InfiniteFormulaDocument = match serde_json::from_value(case["formula"].clone())
        {
            Ok(formula) => formula,
            Err(_)
                if matches!(
                    id,
                    "finite-profile-refuses-unbounded" | "previous-with-unbounded-interval-refuses"
                ) =>
            {
                continue
            }
            Err(error) => panic!("owner formula {id} refused unexpectedly: {error}"),
        };
        let roots: Vec<NodeId> = serde_json::from_value(case["fairness"]["roots"].clone()).unwrap();
        let fairness = if roots.is_empty() {
            None
        } else {
            let identity = formula.content_identity().unwrap();
            Some(
                FairnessPremisesDocument::new(
                    &formula,
                    identity,
                    InfiniteClock::EventPosition,
                    roots,
                )
                .unwrap(),
            )
        };
        let first = format_clean_ascii_v4(&formula, fairness.as_ref(), FormatLimits::default());
        let Some(text) = first.text else {
            assert!(
                matches!(
                    id,
                    "fair-loop-satisfies-premise" | "fairness-on-non-lasso-refuses"
                ),
                "unexpected unrepresentable owner case {id}: {:?}",
                first.error
            );
            assert_eq!(
                first.error.unwrap().code,
                FormatErrorCode::UnrepresentableGraph
            );
            unrepresentable += 1;
            continue;
        };
        let parsed = parse_clean_ascii_v4(
            &text,
            SemanticProfile::InfiniteTraceV1,
            "event_position",
            ParseLimits::default(),
        );
        let document = parsed
            .document
            .as_ref()
            .unwrap_or_else(|| panic!("v4 refused owner case {id}: {:?}", parsed.diagnostics));
        assert_eq!(
            document.content_identity().unwrap(),
            formula.content_identity().unwrap(),
            "{id}"
        );
        assert_eq!(
            parsed.fairness.as_ref().map(|f| f.roots()),
            fairness.as_ref().map(|f| f.roots()),
            "{id}"
        );
        let again =
            format_clean_ascii_v4(document, parsed.fairness.as_ref(), FormatLimits::default());
        assert_eq!(again.text.as_deref(), Some(text.as_str()), "{id}");
        admitted += 1;
    }
    assert_eq!(admitted, 9);
    assert_eq!(unrepresentable, 2);
}
