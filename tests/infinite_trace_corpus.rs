//! Check the TL-208 fixture pin and its independently counted byte loci.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use tl_parse::tl_syntax::SemanticProfile;
use tl_parse::{format_clean_ascii_v4, parse_clean_ascii_v4, FormatLimits, ParseLimits};

// Trace: TC-064, FR-017-AC-2
#[test]
fn v4_fixture_bytes_and_loci_are_pinned() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/infinite-trace");
    let sums = fs::read_to_string(root.join("SHA256SUMS")).expect("read fixture checksums");
    let expected_files: BTreeSet<&str> = [
        "README.md",
        "manifest.json",
        "valid-unbounded.txt",
        "valid-fairness.txt",
        "malformed-leading-zero.txt",
        "malformed-missing-upper.txt",
    ]
    .into_iter()
    .collect();
    let mut pinned_files = BTreeSet::new();
    for line in sums.lines() {
        let (digest, file) = line.split_once("  ").expect("sha256sum line");
        assert!(
            expected_files.contains(file),
            "unexpected fixture path {file}"
        );
        assert!(pinned_files.insert(file), "duplicate pin for {file}");
        let bytes = fs::read(root.join(file)).expect("read pinned fixture");
        assert_eq!(digest, format!("{:x}", Sha256::digest(bytes)), "{file}");
    }
    assert_eq!(pinned_files, expected_files);

    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).expect("manifest bytes"))
            .expect("manifest JSON");
    assert_eq!(
        manifest["schemaVersion"],
        "tl-parse.infinite-trace-corpus/v1"
    );
    assert_eq!(manifest["profile"], "mltl.infinite-trace/v1");
    assert_eq!(manifest["clock"], "event_position");
    assert_eq!(manifest["dialect"], "tl-parse.clean-ascii/v4");
    let cases = manifest["cases"].as_array().expect("case array");
    assert_eq!(cases.len(), 4);
    let mut identities = BTreeSet::new();
    for case in cases {
        let id = case["id"].as_str().expect("case identity");
        assert!(identities.insert(id), "duplicate case identity {id}");
        let file = case["path"].as_str().expect("case path");
        assert!(pinned_files.contains(file), "unpinned source {file}");
        assert!(
            !case["reason"].as_str().unwrap_or_default().is_empty(),
            "missing human rationale {id}"
        );
        let source = fs::read_to_string(root.join(file)).expect("source fixture");
        let input = source.strip_suffix('\n').expect("one fixture newline");
        let parsed = parse_clean_ascii_v4(
            input,
            SemanticProfile::InfiniteTraceV1,
            "event_position",
            ParseLimits::default(),
        );
        assert!(!input.ends_with('\n'), "extra fixture newline {id}");
        for span in case["intervalSpans"].as_array().into_iter().flatten() {
            let from = usize::try_from(span[0].as_u64().expect("span start"))
                .expect("span start fits usize");
            let to =
                usize::try_from(span[1].as_u64().expect("span end")).expect("span end fits usize");
            let interval = input.get(from..to).expect("valid interval span");
            assert!(interval.starts_with('[') && interval.ends_with(')'), "{id}");
        }
        for span in case["premiseSpans"].as_array().into_iter().flatten() {
            let from = usize::try_from(span[0].as_u64().expect("span start"))
                .expect("span start fits usize");
            let to =
                usize::try_from(span[1].as_u64().expect("span end")).expect("span end fits usize");
            let premise = input.get(from..to).expect("valid premise span");
            assert!(premise.starts_with('G') || premise.starts_with('O'), "{id}");
        }
        if let Some(span) = case["expectedSpan"].as_array() {
            let from = usize::try_from(span[0].as_u64().expect("span start"))
                .expect("span start fits usize");
            let to =
                usize::try_from(span[1].as_u64().expect("span end")).expect("span end fits usize");
            assert!(input.get(from..to).is_some(), "invalid refusal locus {id}");
            assert!(case["expectedCode"].as_str().is_some(), "missing code {id}");
            assert!(
                parsed.document.is_none(),
                "malformed fixture admitted: {id}"
            );
            assert_eq!(
                parsed.diagnostics[0].code.as_str(),
                case["expectedCode"].as_str().unwrap(),
                "{id}"
            );
            assert_eq!(
                (
                    parsed.diagnostics[0].span.start(),
                    parsed.diagnostics[0].span.end()
                ),
                (u32::try_from(from).unwrap(), u32::try_from(to).unwrap()),
                "{id}"
            );
        } else {
            assert!(
                case["expectedCanonical"].as_str().is_some(),
                "missing oracle {id}"
            );
            let document = parsed
                .document
                .as_ref()
                .unwrap_or_else(|| panic!("valid fixture refused {id}: {:?}", parsed.diagnostics));
            let formatted =
                format_clean_ascii_v4(document, parsed.fairness.as_ref(), FormatLimits::default());
            assert_eq!(
                formatted.text.as_deref(),
                case["expectedCanonical"].as_str(),
                "{id}"
            );
            let intervals: Vec<_> = parsed
                .interval_spans
                .iter()
                .map(|span| [span.start(), span.end()])
                .collect();
            let expected_intervals: Vec<[u32; 2]> = case["intervalSpans"]
                .as_array()
                .unwrap()
                .iter()
                .map(|span| {
                    [
                        span[0].as_u64().unwrap() as u32,
                        span[1].as_u64().unwrap() as u32,
                    ]
                })
                .collect();
            assert_eq!(intervals, expected_intervals, "{id}");
            let premises: Vec<_> = parsed
                .premise_spans
                .iter()
                .map(|span| [span.start(), span.end()])
                .collect();
            let expected_premises: Vec<[u32; 2]> = case["premiseSpans"]
                .as_array()
                .unwrap()
                .iter()
                .map(|span| {
                    [
                        span[0].as_u64().unwrap() as u32,
                        span[1].as_u64().unwrap() as u32,
                    ]
                })
                .collect();
            assert_eq!(premises, expected_premises, "{id}");
        }
    }
}

// Trace: TC-064, FR-017-AC-2
#[test]
fn one_axis_fixture_mutations_break_the_pinned_oracle() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/infinite-trace");
    let source = fs::read(root.join("valid-unbounded.txt")).unwrap();
    let digest = format!("{:x}", Sha256::digest(&source));
    let pinned = fs::read_to_string(root.join("SHA256SUMS")).unwrap();
    assert!(pinned.contains(&digest));
    let mut changed_source = source.clone();
    changed_source[0] = b'G';
    assert_ne!(format!("{:x}", Sha256::digest(&changed_source)), digest);
    let mut changed_digest = digest.clone();
    changed_digest.replace_range(0..1, if &digest[..1] == "a" { "b" } else { "a" });
    assert_ne!(changed_digest, digest);

    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let valid = &manifest["cases"][0];
    let text = std::str::from_utf8(&source).unwrap().trim_end_matches('\n');
    let parsed = parse_clean_ascii_v4(
        text,
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        ParseLimits::default(),
    );
    let formatted = format_clean_ascii_v4(
        parsed.document.as_ref().unwrap(),
        None,
        FormatLimits::default(),
    )
    .text
    .unwrap();
    assert_eq!(valid["expectedCanonical"], formatted);
    assert_ne!(format!("G{}", &formatted[1..]), formatted);
    let actual_span = parsed.interval_spans[0];
    assert_eq!(
        valid["intervalSpans"][0][1].as_u64().unwrap(),
        u64::from(actual_span.end())
    );
    assert_ne!(
        valid["intervalSpans"][0][1].as_u64().unwrap() + 1,
        u64::from(actual_span.end())
    );

    let malformed = &manifest["cases"][2];
    let malformed_source = fs::read_to_string(root.join("malformed-leading-zero.txt")).unwrap();
    let refused = parse_clean_ascii_v4(
        malformed_source.trim_end_matches('\n'),
        SemanticProfile::InfiniteTraceV1,
        "event_position",
        ParseLimits::default(),
    );
    assert_eq!(
        malformed["expectedCode"],
        refused.diagnostics[0].code.as_str()
    );
    assert_ne!("unexpected_token", refused.diagnostics[0].code.as_str());
}
