use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
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
    ParserKind,
    fn(&str, ParseLimits) -> usize,
);

#[derive(Clone, Copy)]
enum ParserKind {
    Bounded,
    Past,
    Infinite,
}

fn parse_only(kind: ParserKind, source: &str, limits: ParseLimits) {
    match kind {
        ParserKind::Bounded => {
            black_box(parse(source, SemanticProfile::ClosedTraceV1, limits));
        }
        ParserKind::Past => {
            black_box(parse_clean_ascii_v3(source, limits));
        }
        ParserKind::Infinite => {
            black_box(parse_clean_ascii_v4(
                source,
                SemanticProfile::InfiniteTraceV1,
                InfiniteClock::EventPosition.as_str(),
                limits,
            ));
        }
    }
}

fn parser_benchmarks(c: &mut Criterion) {
    let cases: [BenchmarkCase; 10] = [
        (
            "bounded_small",
            BOUNDED_SMALL,
            false,
            ParserKind::Bounded,
            bounded,
        ),
        ("past_small", PAST_SMALL, false, ParserKind::Past, past),
        (
            "infinite_small",
            INFINITE_SMALL,
            false,
            ParserKind::Infinite,
            infinite,
        ),
        (
            "infinite_fairness",
            INFINITE_FAIRNESS,
            false,
            ParserKind::Infinite,
            infinite,
        ),
        (
            "bounded_median",
            SHARED_MEDIAN,
            false,
            ParserKind::Bounded,
            bounded,
        ),
        ("past_median", SHARED_MEDIAN, false, ParserKind::Past, past),
        (
            "infinite_median",
            SHARED_MEDIAN,
            false,
            ParserKind::Infinite,
            infinite,
        ),
        (
            "bounded_near_node_cap",
            SHARED_NEAR_NODE_CAP,
            true,
            ParserKind::Bounded,
            bounded,
        ),
        (
            "past_near_node_cap",
            SHARED_NEAR_NODE_CAP,
            true,
            ParserKind::Past,
            past,
        ),
        (
            "infinite_near_node_cap",
            SHARED_NEAR_NODE_CAP,
            true,
            ParserKind::Infinite,
            infinite,
        ),
    ];
    let mut parse_group = c.benchmark_group("parser_only");
    parse_group.sample_size(20);
    parse_group.warm_up_time(Duration::from_millis(500));
    parse_group.measurement_time(Duration::from_secs(1));
    for (name, source, near_cap, kind, run) in cases {
        let parse_limits = limits(near_cap);
        assert!(
            run(source, parse_limits) > 0,
            "{name} produced empty output"
        );
        parse_group.throughput(Throughput::Bytes(source.len() as u64));
        parse_group.bench_function(name, |b| {
            b.iter(|| parse_only(kind, black_box(source), parse_limits));
        });
    }
    parse_group.finish();

    let mut roundtrip_group = c.benchmark_group("parser_roundtrip");
    roundtrip_group.sample_size(20);
    roundtrip_group.warm_up_time(Duration::from_millis(500));
    roundtrip_group.measurement_time(Duration::from_secs(1));
    for (name, source, near_cap, _, run) in cases {
        let parse_limits = limits(near_cap);
        roundtrip_group.throughput(Throughput::Bytes(source.len() as u64));
        roundtrip_group.bench_function(name, |b| {
            b.iter(|| black_box(run(black_box(source), parse_limits)));
        });
    }
    roundtrip_group.finish();
}

criterion_group!(benches, parser_benchmarks);
criterion_main!(benches);
