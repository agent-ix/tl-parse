#![no_main]
#![forbid(unsafe_code)]

//! `tl-parse.clean-ascii/v5` parse/format fixed point (FR-033). UNBUILT.

use libfuzzer_sys::fuzz_target;
use tl_parse::tl_syntax::{
    FixedDecimalSignalDomain, IntegerSignalDomain, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};
use tl_parse::{
    format_clean_ascii_v5, parse_clean_ascii_v5, FormatLimits, ParseLimits, HARD_MAX_DEPTH,
    HARD_MAX_DIAGNOSTICS, HARD_MAX_NODES, HARD_MAX_PARSE_WORK, HARD_MAX_SOURCE_BYTES,
    HARD_MAX_TOKENS,
};

/// Selector byte: bit 0 picks the catalog (with or without reserved names),
/// bit 1 picks the profile.
fn catalog(selector: u8) -> Option<SignalCatalogDocument> {
    let mut signals = vec![
        ("speed", SignalDomain::Integer(IntegerSignalDomain::new(-1_000, 1_000).ok()?)),
        ("temp", SignalDomain::FixedDecimal(FixedDecimalSignalDomain::new(-10_000, 10_000, 2).ok()?)),
        ("door", SignalDomain::Boolean),
        ("tenth", SignalDomain::FixedDecimal(FixedDecimalSignalDomain::new(-1_000, 1_000, 1).ok()?)),
    ];
    if selector & 1 == 1 {
        signals.push(("each", SignalDomain::Integer(IntegerSignalDomain::new(0, 9).ok()?)));
        signals.push(("p17", SignalDomain::Integer(IntegerSignalDomain::new(0, 9).ok()?)));
    }
    let declarations = signals
        .into_iter()
        .enumerate()
        .map(|(index, (name, domain))| {
            OwnedSignalDeclaration::new(SignalId(index as u32), name.to_owned(), domain)
        })
        .collect();
    SignalCatalogDocument::new(
        declarations,
        vec![PropositionBinding::new(PropositionId(0), SignalId(2))],
    )
    .ok()
}

// Trace: TC-1556, FR-033-AC-2
fuzz_target!(|data: &[u8]| {
    let Some((selector, rest)) = data.split_first() else { return; };
    let Ok(source) = std::str::from_utf8(rest) else { return; };
    let Some(catalog) = catalog(*selector) else { return; };
    let profile = if selector & 2 == 0 {
        SemanticProfile::ClosedTraceV1
    } else {
        SemanticProfile::OnlinePrefixV1
    };
    // Resource caps equal to the hard maxima.
    let limits = ParseLimits {
        max_source_bytes: HARD_MAX_SOURCE_BYTES, max_tokens: HARD_MAX_TOKENS,
        max_nodes: HARD_MAX_NODES, max_depth: HARD_MAX_DEPTH,
        max_diagnostics: HARD_MAX_DIAGNOSTICS, max_work: HARD_MAX_PARSE_WORK,
    };
    let report = parse_clean_ascii_v5(source, &catalog, profile, limits);
    assert!(report.document.is_some() != report.refusal.is_some());
    assert!(report.stats.work <= limits.max_work);
    assert!(report.diagnostics.len() <= limits.max_diagnostics);
    let Some(document) = report.document.as_ref() else { return; };
    let formatted = format_clean_ascii_v5(document, report.predicate_table.as_ref(), &catalog,
        FormatLimits::default());
    let Some(text) = formatted.text else { return; };
    let reparsed = parse_clean_ascii_v5(&text, &catalog, profile, limits);
    assert!(reparsed.document.is_some(), "{text}: {:?}", reparsed.diagnostics);
    let again = format_clean_ascii_v5(reparsed.document.as_ref().unwrap(),
        reparsed.predicate_table.as_ref(), &catalog, FormatLimits::default());
    assert_eq!(again.text.as_deref(), Some(text.as_str()));
});
