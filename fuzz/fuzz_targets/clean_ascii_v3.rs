#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use tl_parse::{format_clean_ascii_v3, parse_clean_ascii_v3, FormatLimits, ParseLimits};

// Trace: TC-065, FR-017-AC-3
fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else { return; };
    let limits = ParseLimits {
        max_source_bytes: 4096, max_tokens: 512, max_nodes: 256,
        max_depth: 64, max_diagnostics: 16, max_work: 16384,
    };
    let report = parse_clean_ascii_v3(source, limits);
    assert!(report.stats.tokens <= limits.max_tokens);
    assert!(report.stats.nodes <= limits.max_nodes);
    assert!(report.stats.work <= limits.max_work);
    if let Some(document) = report.document.as_ref() {
        let formatted = format_clean_ascii_v3(document,
            FormatLimits { max_output_bytes: 16384, max_work: 65536 });
        if let Some(text) = formatted.text {
            let reparsed = parse_clean_ascii_v3(&text, ParseLimits::default());
            assert!(reparsed.document.is_some());
        }
    }
});
