# Diagnostics

A refused parse returns `Diagnostic` values in `ParseReport.diagnostics` (or the equivalent field of the other reports) and no document. A diagnostic has a stable `code`, a `severity`, a half-open byte `span` into the source, the `found` token, the `expected` token categories, the `recovery` the parser took and a `message`. Match on `code`; the message text is for people.

`severity` has one value, `error`. Spans are UTF-8 byte offsets, and `span.start()` and `span.end()` read them. `Diagnostic` implements `Display` and `Error`.

## Codes

`DiagnosticCode::as_str` gives the wire spelling, which is also the JSON value.

| Code | Raised when |
| --- | --- |
| `source_limit` | The source exceeds the byte limit. |
| `token_limit` | The token count exceeds the limit. |
| `node_limit` | The node count exceeds the limit. |
| `depth_limit` | Nesting exceeds the depth limit. |
| `work_limit` | Parser work exceeds the limit. |
| `unexpected_character` | A character has no meaning in the dialect. |
| `unknown_identifier` | An identifier is not in the closed vocabulary. |
| `non_canonical_number` | A number has a leading zero. |
| `integer_overflow` | A number exceeds `u32`. |
| `unexpected_token` | A token is not valid in this position. |
| `missing_token` | A required delimiter or operand is absent. |
| `invalid_interval` | An interval has start greater than end. |
| `trailing_input` | Input remains after a complete expression. |
| `validation_failure` | The parsed graph failed tl-syntax validation. |
| `unsupported_operator` | An operator is recognized but is outside the selected dialect. |
| `infinite_profile_mismatch` | v4 was called with a profile other than `mltl.infinite-trace/v1`. |
| `infinite_clock_mismatch` | v4 was called with a clock other than `event_position`. |
| `duplicate_fairness_premise` | Two fairness premises are the same formula. |

`expected` holds categories from a closed set: `expression`, `integer`, `left_bracket`, `right_bracket`, `right_parenthesis`, `left_brace`, `right_brace`, `semicolon`, `colon`, `comma` and `end_of_input`. `recovery` is `none`, `skipped_token`, `inserted_token` or `stopped`.

## Limits

`ParseLimits::default()` uses the hard maxima except for retained diagnostics. A caller can lower any field and cannot raise it past the hard maximum; the report records the effective values.

| Field | Default | Hard maximum |
| --- | ---: | ---: |
| `max_source_bytes` | 1,048,576 | 1,048,576 |
| `max_tokens` | 100,000 | 100,000 |
| `max_nodes` | 10,000 | 10,000 |
| `max_depth` | 256 | 256 |
| `max_diagnostics` | 32 | 64 |
| `max_work` | 1,000,000 | 1,000,000 |

`ParseStats` in each report records the observed source bytes, tokens, nodes, diagnostics, work and nesting depth, and whether diagnostics were truncated by `max_diagnostics`.

The formatter has its own limits.

| Field | Default | Hard maximum |
| --- | ---: | ---: |
| `FormatLimits::max_output_bytes` | 1,048,576 | 1,048,576 |
| `FormatLimits::max_work` | 4,194,304 | 4,194,304 |

## Format errors

`format_document` and its siblings return a `FormatReport` with `text` on success or an `error` on failure. `FormatError.code` is one of:

| Code | Raised when |
| --- | --- |
| `invalid_graph` | The document failed tl-syntax validation, or its schema and profile have no matching dialect. |
| `unrepresentable_graph` | The graph has a shape the dialect cannot write. |
| `output_limit` | Output exceeds the byte limit. |
| `work_limit` | Formatter work exceeds the limit. |

## Strict reader errors

The `from_json_bytes` readers return `StrictParseArtifactReadError`: `DocumentTooLarge`, `DepthLimitExceeded`, `StringTooLarge`, `WorkLimitExceeded`, `NonCanonicalDocument` or `InvalidDocument`. The enum is `#[non_exhaustive]`. `ParseArtifactLimits` can lower the ceilings (64 MiB document, JSON depth 64, 64 KiB string, 256 Mi work units) and cannot raise them.

## The checked corpus

`corpus/v1` holds one well-formed fixture and six refused ones, with the expected code or canonical text of each in `manifest.json`: `unexpected_character`, `non_canonical_number`, `invalid_interval`, `missing_token`, `token_limit` and `depth_limit`. `corpus/infinite-trace` holds two valid and two malformed v4 fixtures, with expected spans. `cargo run --example corpus_conformance -- --manifest corpus/v1/manifest.json` replays the first set. It prints one JSON row per fixture with the outcome `pass` for a well-formed fixture that formats to its declared text and re-parses to the same graph, `malformed` for a refusal with the declared code, `fail` for any disagreement, `vacuous` for a fixture that declares nothing and `unavailable` for an unreadable fixture. The exit status is nonzero when any row is `fail`.
