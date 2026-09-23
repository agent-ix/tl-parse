use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use sha2::{Digest, Sha256};
use tl_parse::tl_syntax::{InfiniteClock, SemanticProfile};
use tl_parse::{
    format_clean_ascii_v3, format_clean_ascii_v4, format_document, parse, parse_clean_ascii_v3,
    parse_clean_ascii_v4, FormatLimits, ParseLimits,
};

const BOUNDED_SMALL: &str = include_str!("inputs/bounded-small.txt");
const PAST_SMALL: &str = include_str!("inputs/past-small.txt");
const INFINITE_SMALL: &str = include_str!("inputs/infinite-small.txt");
const INFINITE_FAIRNESS: &str = include_str!("inputs/infinite-fairness.txt");
const SHARED_MEDIAN: &str = include_str!("inputs/shared-median.txt");
const SHARED_NEAR_NODE_CAP: &str = include_str!("inputs/shared-near-node-cap.txt");
const INPUT_SHA256SUMS: &str = include_str!("inputs/SHA256SUMS");

fn verify_inputs() {
    let files = [
        ("bounded-small.txt", BOUNDED_SMALL),
        ("infinite-fairness.txt", INFINITE_FAIRNESS),
        ("infinite-small.txt", INFINITE_SMALL),
        ("past-small.txt", PAST_SMALL),
        ("shared-median.txt", SHARED_MEDIAN),
        ("shared-near-node-cap.txt", SHARED_NEAR_NODE_CAP),
    ];
    let actual = files
        .iter()
        .map(|(name, source)| format!("{:x}  {name}\n", Sha256::digest(source.as_bytes())))
        .collect::<String>();
    assert_eq!(actual, INPUT_SHA256SUMS, "benchmark input digests changed");
}

fn limits(near_cap: bool) -> ParseLimits {
    ParseLimits {
        max_nodes: if near_cap {
            128
        } else {
            ParseLimits::default().max_nodes
        },
        ..ParseLimits::default()
    }
}

fn bounded(source: &str, limits: ParseLimits) -> usize {
    let report = parse(source, SemanticProfile::ClosedTraceV1, limits);
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let document = report.document.expect("bounded input is pinned and valid");
    let formatted = format_document(&document, FormatLimits::default());
    formatted.text.expect("bounded format succeeds").len()
}

fn past(source: &str, limits: ParseLimits) -> usize {
    let report = parse_clean_ascii_v3(source, limits);
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let document = report.document.expect("past input is pinned and valid");
    let formatted = format_clean_ascii_v3(&document, FormatLimits::default());
    formatted.text.expect("past format succeeds").len()
}

fn infinite(source: &str, limits: ParseLimits) -> usize {
    let report = parse_clean_ascii_v4(
        source,
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition.as_str(),
        limits,
    );
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    let document = report.document.expect("infinite input is pinned and valid");
    let formatted =
        format_clean_ascii_v4(&document, report.fairness.as_ref(), FormatLimits::default());
    formatted.text.expect("infinite format succeeds").len()
}

type BenchmarkCase = (
    &'static str,
    &'static str,
    bool,
    fn(&str, ParseLimits) -> usize,
);

fn parser_roundtrip(c: &mut Criterion) {
    verify_inputs();
    let mut group = c.benchmark_group("parser_roundtrip");
    group.sample_size(20);
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
    let cases: [BenchmarkCase; 10] = [
        ("bounded_small", BOUNDED_SMALL, false, bounded),
        ("past_small", PAST_SMALL, false, past),
        ("infinite_small", INFINITE_SMALL, false, infinite),
        ("infinite_fairness", INFINITE_FAIRNESS, false, infinite),
        ("bounded_median", SHARED_MEDIAN, false, bounded),
        ("past_median", SHARED_MEDIAN, false, past),
        ("infinite_median", SHARED_MEDIAN, false, infinite),
        ("bounded_near_node_cap", SHARED_NEAR_NODE_CAP, true, bounded),
        ("past_near_node_cap", SHARED_NEAR_NODE_CAP, true, past),
        (
            "infinite_near_node_cap",
            SHARED_NEAR_NODE_CAP,
            true,
            infinite,
        ),
    ];
    for (name, source, near_cap, run) in cases {
        let parse_limits = limits(near_cap);
        assert!(
            run(source, parse_limits) > 0,
            "{name} produced empty output"
        );
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_function(name, |b| {
            b.iter(|| black_box(run(black_box(source), parse_limits)));
        });
    }
    group.finish();
}

criterion_group!(benches, parser_roundtrip);
criterion_main!(benches);
