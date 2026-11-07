# Examples

Every Rust example here compiles and passes against the crate. The first example is the one in the README.

## Parse and format

```rust
use tl_parse::{format_document, parse, FormatLimits, ParseLimits};
use tl_parse::tl_syntax::SemanticProfile;

let report = parse(
    "G[0,2](p0 -> F[1,1]p1)",
    SemanticProfile::ClosedTraceV1,
    ParseLimits::default(),
);
assert!(report.diagnostics.is_empty());
let document = report.document.expect("the source parses");
assert_eq!(document.nodes().len(), 5);

let formatted = format_document(&document, FormatLimits::default());
assert_eq!(formatted.text.as_deref(), Some("G[0,2](p0->F[1,1]p1)"));
```

The fixture `corpus/v1/valid-canonical.txt` holds this source. Whitespace in the source does not appear in the canonical text.

## Read a diagnostic

```rust
use tl_parse::{parse, report_json, DiagnosticCode, ParseLimits};
use tl_parse::tl_syntax::SemanticProfile;

let report = parse("p0 U[3,1] p1", SemanticProfile::ClosedTraceV1, ParseLimits::default());
assert!(report.document.is_none());

let diagnostic = &report.diagnostics[0];
assert_eq!(diagnostic.code, DiagnosticCode::InvalidInterval);
assert_eq!((diagnostic.span.start(), diagnostic.span.end()), (4, 9));
assert_eq!(
    diagnostic.to_string(),
    "invalid_interval at 4..9: inclusive interval start 3 exceeds end 1"
);

// The whole report serializes as one JSON line with stable key order.
let json = report_json(&report).unwrap();
assert!(json.starts_with(r#"{"schema_version":"tl-parse.diagnostics/v1""#));
```

## Lower the limits

```rust
use tl_parse::{parse, DiagnosticCode, ParseLimits};
use tl_parse::tl_syntax::SemanticProfile;

let limits = ParseLimits { max_depth: 2, ..ParseLimits::default() };
let report = parse("!!!!p0", SemanticProfile::ClosedTraceV1, limits);
assert!(report.document.is_none());
assert_eq!(report.diagnostics[0].code, DiagnosticCode::DepthLimit);
```

`corpus/v1/depth-limit.txt` and `token-limit.txt` do the same with `max_depth` and `max_tokens`.

## Derived operators (v2)

```rust
use tl_parse::{format_document, parse_clean_ascii_v2, DerivedOperator, FormatLimits, ParseLimits};
use tl_parse::tl_syntax::SemanticProfile;

let report = parse_clean_ascii_v2(
    "p0 W[0,4] p1",
    SemanticProfile::ClosedTraceV1,
    ParseLimits::default(),
);
assert!(report.diagnostics.is_empty());

// One use of W: two operand nodes plus three generated nodes.
let document = report.document.expect("the source parses");
assert_eq!(document.nodes().len(), 5);
assert_eq!(report.lowerings.len(), 1);
assert_eq!(report.lowerings[0].kind, DerivedOperator::WeakUntil);

let text = format_document(&document, FormatLimits::default()).text.unwrap();
assert_eq!(text, "p0U[0,4]p1|G[0,4]p0");
```

## Past operators (v3)

```rust
use tl_parse::{format_clean_ascii_v3, parse_clean_ascii_v3, FormatLimits, ParseLimits};

let report = parse_clean_ascii_v3("O[0,2]p0 S[1,3] Yp1", ParseLimits::default());
assert!(report.diagnostics.is_empty());
let document = report.document.expect("the source parses");

let text = format_clean_ascii_v3(&document, FormatLimits::default()).text.unwrap();
assert_eq!(text, "O[0,2]p0S[1,3]Yp1");
```

A future operator is refused: `parse_clean_ascii_v3("F[0,1]p0", ...)` returns `unsupported_operator`.

## Infinite traces and fairness (v4)

```rust
use tl_parse::{
    format_clean_ascii_v4, parse_clean_ascii_v4, FormatLimits, InfiniteDisposition, ParseLimits,
};
use tl_parse::tl_syntax::SemanticProfile;

let report = parse_clean_ascii_v4(
    "fair {G[0,)p0; O[1,)p1}: F[0,)p2",
    SemanticProfile::InfiniteTraceV1,
    "event_position",
    ParseLimits::default(),
);
assert!(report.diagnostics.is_empty());
assert_eq!(report.disposition(), InfiniteDisposition::NoTemporalVerdict);
assert_eq!(report.interval_spans.len(), 3);
assert_eq!(report.premise_spans.len(), 2);

let text = format_clean_ascii_v4(
    report.document.as_ref().unwrap(),
    report.fairness.as_ref(),
    FormatLimits::default(),
)
.text
.unwrap();
assert_eq!(text, "fair {G[0,)p0; O[1,)p1}: F[0,)p2");

// Any profile other than the infinite-trace profile is refused.
let refused = parse_clean_ascii_v4(
    "F[0,)p0",
    SemanticProfile::ClosedTraceV1,
    "event_position",
    ParseLimits::default(),
);
assert_eq!(refused.disposition(), InfiniteDisposition::Unsupported);
```

The fixture `corpus/infinite-trace/valid-fairness.txt` holds the same formula with extra whitespace. `tests/infinite_v4.rs` covers refusals, spans and the strict reader.

## Strict reports

```rust
use tl_parse::{parse_clean_ascii_v3, ParseArtifactLimits, ParseLimits, PastParseReport};

let report = parse_clean_ascii_v3("O[0,1]p0", ParseLimits::default());
let bytes = report.canonical_json_bytes().unwrap();

// The canonical bytes read back to the same report.
let read = PastParseReport::from_json_bytes(&bytes, ParseArtifactLimits::default()).unwrap();
assert_eq!(read, report);

// Valid JSON that is not the canonical encoding is refused.
let mut spaced = bytes.clone();
spaced.insert(1, b' ');
assert!(PastParseReport::from_json_bytes(&spaced, ParseArtifactLimits::default()).is_err());
```

`InfiniteParseReport::from_json_bytes` works the same way for v4.

## Bind propositions to signals

```rust
use tl_parse::{parse_with_context, ParseLimits};
use tl_parse::tl_syntax::{
    OwnedSignalDeclaration, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId,
};

let catalog = SignalCatalogDocument::new(
    vec![
        OwnedSignalDeclaration::new(SignalId(10), "brake".into(), SignalDomain::Boolean),
        OwnedSignalDeclaration::new(SignalId(11), "door".into(), SignalDomain::Boolean),
    ],
    vec![
        PropositionBinding::new(PropositionId(1), SignalId(10)),
        PropositionBinding::new(PropositionId(2), SignalId(11)),
    ],
)
.unwrap();

let report = parse_with_context(
    "G[0,2](p1 -> F[1,1]p2)",
    SemanticProfile::ClosedTraceV1,
    ParseLimits::default(),
    &catalog,
    None,
)
.unwrap();
// One binding per free proposition, in order of first appearance.
assert_eq!(report.bindings.len(), 2);
assert_eq!(report.bindings[0].signal, SignalId(10));

// A proposition with no binding is refused with its source span.
let error = parse_with_context(
    "p3",
    SemanticProfile::ClosedTraceV1,
    ParseLimits::default(),
    &catalog,
    None,
)
.unwrap_err();
assert_eq!(error.to_string(), "missing signal binding for proposition 3 at 0..2");
```

`tests/contextual.rs` covers this path, including a requirement context.

## Replay the corpus and sweep the round trip

```sh
cargo run --quiet --example corpus_conformance -- --manifest corpus/v1/manifest.json
cargo run --quiet --release --example roundtrip_sweep
TL_PARSE_SWEEP_CASES=1000 cargo run --quiet --example roundtrip_sweep
cargo run --quiet --example fuzz_campaign -- parser
```

`corpus_conformance` and `roundtrip_sweep` print one JSON row per check and exit nonzero on a failing row. `roundtrip_sweep` reads the number of generated sources from `TL_PARSE_SWEEP_CASES` and defaults to 20,000. `fuzz_campaign` stages the checked seeds for one cargo-fuzz target (`parser` or `clean_ascii_v2`), runs 64 executions under a 300-second deadline with LeakSanitizer, and exits 0 for a pass, 1 for a failure or crash artifact and 2 when the run could not complete. It needs a nightly toolchain with cargo-fuzz.
