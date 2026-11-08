//! TL-220: byte loci for every v4 unbounded and fairness form, checked against a
//! hand-counted manifest and carried through the canonical-text round trip.

use std::fs;
use std::path::Path;

use serde_json::Value;
use tl_parse::tl_syntax::SemanticProfile;
use tl_parse::{format_clean_ascii_v4, parse_clean_ascii_v4, FormatLimits, ParseLimits};

fn pairs(value: &Value) -> Vec<[u32; 2]> {
    value
        .as_array()
        .expect("span array")
        .iter()
        .map(|span| {
            [
                u32::try_from(span[0].as_u64().unwrap()).unwrap(),
                u32::try_from(span[1].as_u64().unwrap()).unwrap(),
            ]
        })
        .collect()
}

fn slices<'a>(text: &'a str, spans: &[[u32; 2]]) -> Vec<&'a str> {
    spans
        .iter()
        .map(|[from, to]| &text[usize::try_from(*from).unwrap()..usize::try_from(*to).unwrap()])
        .collect()
}

fn parse(source: &str) -> tl_parse::InfiniteParseReport {
    parse_clean_ascii_v4(
        source,
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        ParseLimits::default(),
    )
}

// Trace: TC-064, FR-017-AC-2
#[test]
fn every_unbounded_and_fairness_form_keeps_its_byte_loci_through_the_round_trip() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/infinite-loci/cases.json");
    let manifest: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(
        manifest["schemaVersion"],
        "tl-parse.infinite-loci-corpus/v1"
    );
    let (mut valid, mut refused) = (0, 0);
    let mut operators = std::collections::BTreeSet::new();
    for case in manifest["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let source = case["source"].as_str().unwrap();
        let report = parse(source);
        if let Some(code) = case["expectedCode"].as_str() {
            refused += 1;
            assert!(report.document.is_none(), "{id}: refusal admitted a graph");
            let diagnostic = &report.diagnostics[0];
            assert_eq!(diagnostic.code.as_str(), code, "{id}");
            let span = pairs(&Value::Array(vec![case["expectedSpan"].clone()]))[0];
            assert_eq!(
                [diagnostic.span.start(), diagnostic.span.end()],
                span,
                "{id}"
            );
            assert!(usize::try_from(span[1]).unwrap() <= source.len(), "{id}");
            continue;
        }
        valid += 1;
        let document = report
            .document
            .as_ref()
            .unwrap_or_else(|| panic!("{id}: {:?}", report.diagnostics));
        let intervals: Vec<[u32; 2]> = report
            .interval_spans
            .iter()
            .map(|s| [s.start(), s.end()])
            .collect();
        let premises: Vec<[u32; 2]> = report
            .premise_spans
            .iter()
            .map(|s| [s.start(), s.end()])
            .collect();
        assert_eq!(intervals, pairs(&case["intervalSpans"]), "{id}");
        assert_eq!(premises, pairs(&case["premiseSpans"]), "{id}");
        for text in slices(source, &intervals) {
            assert!(text.starts_with('[') && text.ends_with([')', ']']), "{id}");
        }
        for op in source.chars().filter(|c| "FGOHURST".contains(*c)) {
            operators.insert(op);
        }

        // Fixed point: canonical text is the manifest's, re-parses to itself,
        // and its own loci slice out the same interval and premise text.
        let canonical =
            format_clean_ascii_v4(document, report.fairness.as_ref(), FormatLimits::default())
                .text
                .unwrap();
        assert_eq!(
            canonical,
            case["expectedCanonical"].as_str().unwrap(),
            "{id}"
        );
        let again = parse(&canonical);
        let again_doc = again.document.as_ref().expect(id);
        let again_text =
            format_clean_ascii_v4(again_doc, again.fairness.as_ref(), FormatLimits::default())
                .text
                .unwrap();
        assert_eq!(again_text, canonical, "{id}");
        let again_intervals: Vec<[u32; 2]> = again
            .interval_spans
            .iter()
            .map(|s| [s.start(), s.end()])
            .collect();
        assert_eq!(
            slices(&canonical, &again_intervals),
            slices(source, &intervals),
            "{id}: interval text changed across the round trip"
        );
        let again_premises: Vec<[u32; 2]> = again
            .premise_spans
            .iter()
            .map(|s| [s.start(), s.end()])
            .collect();
        let normalize = |text: &str| text.split_whitespace().collect::<String>();
        assert_eq!(
            slices(&canonical, &again_premises)
                .into_iter()
                .map(normalize)
                .collect::<Vec<_>>(),
            slices(source, &premises)
                .into_iter()
                .map(normalize)
                .collect::<Vec<_>>(),
            "{id}: premise text changed across the round trip"
        );
    }
    // Every unbounded-capable operator and a fairness envelope are exercised.
    for op in ['F', 'G', 'O', 'H', 'U', 'R', 'S', 'T'] {
        assert!(operators.contains(&op), "operator {op} has no locus case");
    }
    assert!(
        valid >= 13 && refused >= 8,
        "valid={valid} refused={refused}"
    );
}
