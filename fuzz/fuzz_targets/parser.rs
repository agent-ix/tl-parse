#![no_main]

use libfuzzer_sys::fuzz_target;
use tl_parse::tl_syntax::SemanticProfile;
use tl_parse::{format_document, parse, report_json, FormatLimits, ParseLimits};

// Trace: TC-019, FR-005-AC-2, StR-002-VC-2
fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let limits = ParseLimits {
        max_source_bytes: 4_096,
        max_tokens: 512,
        max_nodes: 256,
        max_depth: 64,
        max_diagnostics: 16,
        max_work: 16_384,
    };
    let report = parse(source, SemanticProfile::ClosedTraceV1, limits);
    assert!(report.stats.tokens <= limits.max_tokens);
    assert!(report.stats.nodes <= limits.max_nodes);
    assert!(report.stats.work <= limits.max_work);
    assert!(report.stats.diagnostics <= limits.max_diagnostics);
    assert!(report_json(&report).is_ok());
    if !report.diagnostics.is_empty() || report.stats.diagnostics_truncated {
        assert!(report.document.is_none());
    }
    if let Some(document) = report.document.as_ref() {
        let formatted = format_document(
            document,
            FormatLimits {
                max_output_bytes: 16_384,
                max_work: 65_536,
            },
        );
        assert_eq!(formatted.text.is_some(), formatted.error.is_none());
        let text = formatted.text.expect("default-sized valid graph formats");
        let reparsed = parse(&text, SemanticProfile::ClosedTraceV1, limits);
        let reparsed = reparsed.document.expect("canonical text reparses");
        assert_eq!(document.semantic_profile(), reparsed.semantic_profile());
        assert_eq!(document.root(), reparsed.root());
        assert_eq!(document.nodes().len(), reparsed.nodes().len());
        assert!(document
            .nodes()
            .iter()
            .zip(reparsed.nodes())
            .all(|(left, right)| left.kind == right.kind));
    }
});
