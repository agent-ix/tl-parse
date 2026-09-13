#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use tl_parse::tl_syntax::SemanticProfile;
use tl_parse::{
    format_document, parse, parse_clean_ascii_v2, DiagnosticCode, FormatLimits, ParseLimits,
    DIALECT_REVISION,
};

// Trace: TC-044, FR-008-AC-5
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
    let report = parse_clean_ascii_v2(source, SemanticProfile::ClosedTraceV1, limits);
    assert!(report.stats.tokens <= limits.max_tokens);
    assert!(report.stats.nodes <= limits.max_nodes);
    assert!(report.stats.work <= limits.max_work);
    assert!(report.stats.diagnostics <= limits.max_diagnostics);
    let json = serde_json::to_string(&report).expect("derived report serializes");
    assert!(!json.contains(DIALECT_REVISION));
    if !report.diagnostics.is_empty() || report.stats.diagnostics_truncated {
        assert!(report.document.is_none());
    }
    let Some(document) = report.document.as_ref() else {
        assert!(report.lowerings.is_empty());
        return;
    };
    for record in &report.lowerings {
        assert!(record.first_generated.0 > record.left.0.max(record.right.0));
        assert_eq!(record.root.0, record.first_generated.0 + 2);
        assert!(record.expression_span.start() <= record.operator_span.start());
        assert!(record.operator_span.end() <= record.expression_span.end());
    }
    let formatted = format_document(
        document,
        FormatLimits {
            max_output_bytes: 16_384,
            max_work: 65_536,
        },
    );
    assert_eq!(formatted.text.is_some(), formatted.error.is_none());
    let Some(text) = formatted.text else {
        return;
    };
    // Lowering shares the left operand, so primitive text repeats it and the
    // reparsed graph is larger; chained left operands grow it exponentially.
    // v1 must accept the text or refuse it only through a resource limit, and
    // an accepted text must reach a canonical-text fixed point.
    let reparse_limits = ParseLimits::default();
    let primitive = parse(&text, SemanticProfile::ClosedTraceV1, reparse_limits);
    let Some(reparsed) = primitive.document else {
        assert!(!primitive.diagnostics.is_empty());
        assert!(primitive.diagnostics.iter().all(|diagnostic| matches!(
            diagnostic.code,
            DiagnosticCode::SourceLimit
                | DiagnosticCode::TokenLimit
                | DiagnosticCode::NodeLimit
                | DiagnosticCode::DepthLimit
                | DiagnosticCode::WorkLimit
        )));
        return;
    };
    let again = format_document(
        &reparsed,
        FormatLimits {
            max_output_bytes: 16_384,
            max_work: 1_048_576,
        },
    );
    assert_eq!(again.text.as_deref(), Some(text.as_str()));
});
