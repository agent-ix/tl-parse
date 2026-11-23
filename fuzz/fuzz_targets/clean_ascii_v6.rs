#![no_main]
#![forbid(unsafe_code)]

//! `tl-parse.clean-ascii/v6` (spec name v5) parse/format round trip (FR-061). UNBUILT.

use libfuzzer_sys::fuzz_target;
use tl_parse::{format_clean_ascii_v6, parse_clean_ascii_v6, FormatLimits, LtlProfile, ParseLimits};

// Trace: TC-121, FR-061-AC-2
fuzz_target!(|data: &[u8]| {
    let Some((selector, rest)) = data.split_first() else { return; };
    let Ok(source) = std::str::from_utf8(rest) else { return; };
    // The profile bit is the caller's explicit choice, never read from the text.
    let profile = if selector & 1 == 0 {
        LtlProfile::LtlfFiniteTrace
    } else {
        LtlProfile::InfiniteTrace
    };
    let limits = ParseLimits {
        max_source_bytes: 4096, max_tokens: 512, max_nodes: 256,
        max_depth: 64, max_diagnostics: 16, max_work: 16384,
    };
    let report = parse_clean_ascii_v6(source, profile, limits);
    assert!(report.document.is_some() != report.refusal.is_some());
    assert!(report.stats.tokens <= limits.max_tokens);
    assert!(report.stats.nodes <= limits.max_nodes);
    assert!(report.stats.work <= limits.max_work);
    assert!(report.diagnostics.len() <= limits.max_diagnostics);
    let Some(document) = report.document.as_ref() else { return; };
    let formatted = format_clean_ascii_v6(document, profile, FormatLimits::default());
    let text = formatted.text.expect("every parsed graph is in the v6 image");
    let reparsed = parse_clean_ascii_v6(&text, profile, ParseLimits::default());
    let again = format_clean_ascii_v6(reparsed.document.as_ref().expect("canonical text parses"),
        profile, FormatLimits::default());
    assert_eq!(again.text.as_deref(), Some(text.as_str()));
});
