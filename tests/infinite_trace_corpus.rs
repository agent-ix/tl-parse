//! Check the TL-208 fixture pin and its independently counted byte loci.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

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
        } else {
            assert!(
                case["expectedCanonical"].as_str().is_some(),
                "missing oracle {id}"
            );
        }
    }
}
