//! Replay the checked hostile-input corpus through the real crate (FR-006-AC-2, FR-006-AC-6).
//!
//! This is a producer. It runs the actual lexer, parser and formatter over the
//! actual corpus bytes and writes one JSON object per fixture to stdout. It
//! computes no aggregate verdict, retains nothing, and knows nothing about
//! Quoin, Quire or attestations — the assurance chain reads the rows this
//! writes and reports what they say.
//!
//! The outcome vocabulary is the point of this file. A malformed fixture that
//! the parser rejects with its declared diagnostic is reported as `malformed`,
//! which is a *correct* outcome and is deliberately neither `pass` nor `fail`:
//!
//!   * it is not `fail`, because the parser did exactly what it must do, and a
//!     corpus that is six-sevenths malformed by design would otherwise report a
//!     failing proof forever;
//!   * it is not `pass`, because collapsing it into `pass` would make a parser
//!     that silently accepted every malformed input indistinguishable from one
//!     that rejected them all correctly.
//!
//! `fail` is reserved for genuine disagreement: a malformed fixture that parsed,
//! a valid fixture that did not, or a diagnostic code or canonical rendering
//! that is not the declared one.

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use serde::Deserialize;
use tl_parse::{format_document, parse, DiagnosticCode, FormatLimits, ParseLimits};
use tl_syntax::{FormulaDocument, NodeKind, SemanticProfile};

const PROTOCOL: &str = "tl-parse.parser-conformance/v1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    revision: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Case {
    path: String,
    profile: String,
    max_tokens: Option<usize>,
    max_depth: Option<usize>,
    expected_code: Option<String>,
    expected_canonical: Option<String>,
}

/// One fixture's result, in the shape the assurance chain transcribes.
struct Row {
    symbol: String,
    outcome: &'static str,
    detail: String,
}

fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", other as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn profile_of(name: &str) -> Option<SemanticProfile> {
    match name {
        "closed" => Some(SemanticProfile::ClosedTraceV1),
        "online" => Some(SemanticProfile::OnlinePrefixV1),
        _ => None,
    }
}

fn code_name(code: DiagnosticCode) -> &'static str {
    code.as_str()
}

/// The graph identity the round-trip property is about: profile, root, and the
/// ordered node kinds. Source spans are deliberately excluded — see the call
/// site.
fn structural(document: &FormulaDocument) -> (SemanticProfile, u32, Vec<NodeKind>) {
    (
        document.semantic_profile(),
        document.root().0,
        document.nodes().iter().map(|node| node.kind).collect(),
    )
}

/// Classify one fixture by running it, never by consulting a stored answer.
fn evaluate(corpus: &Path, case: &Case) -> Row {
    let symbol = format!("corpus/v1/{}", case.path);

    let Some(profile) = profile_of(&case.profile) else {
        return Row {
            symbol,
            outcome: "fail",
            detail: format!("unknown profile {}", escape(&case.profile)),
        };
    };

    // A fixture whose bytes cannot be read has not been evaluated. That is
    // `unavailable`, which is a different fact from a fixture that was read and
    // disagreed, and it must not be reported as either a pass or a failure.
    let source = match fs::read_to_string(corpus.join(&case.path)) {
        Ok(text) => text,
        Err(error) => {
            return Row {
                symbol,
                outcome: "unavailable",
                detail: format!("fixture unreadable: {}", escape(&error.to_string())),
            };
        }
    };

    let defaults = ParseLimits::default();
    let limits = ParseLimits {
        max_tokens: case.max_tokens.unwrap_or(defaults.max_tokens),
        max_depth: case.max_depth.unwrap_or(defaults.max_depth),
        ..defaults
    };
    let report = parse(&source, profile, limits);

    match (&case.expected_code, &case.expected_canonical) {
        // A fixture that declares nothing has asserted nothing. Reporting it as
        // a pass would let an empty declaration buy coverage.
        (None, None) => Row {
            symbol,
            outcome: "vacuous",
            detail: "the manifest declares neither expectedCode nor expectedCanonical".to_owned(),
        },

        (Some(_), Some(_)) => Row {
            symbol,
            outcome: "fail",
            detail: "the manifest declares both expectedCode and expectedCanonical".to_owned(),
        },

        // The malformed class. The parser must reject, and must reject with the
        // declared code, at a span inside the source.
        (Some(expected), None) => {
            let Some(first) = report.diagnostics.first() else {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: format!(
                        "declared {} but the source parsed with no diagnostic",
                        escape(expected)
                    ),
                };
            };
            let observed = code_name(first.code);
            if observed != expected.as_str() {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: format!(
                        "declared {} but observed {}",
                        escape(expected),
                        escape(observed)
                    ),
                };
            }
            if report.document.is_some() {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: "a diagnostic was reported and a document was still produced"
                        .to_owned(),
                };
            }
            Row {
                symbol,
                outcome: "malformed",
                detail: format!(
                    "rejected as {} at {}..{}",
                    escape(observed),
                    first.span.start(),
                    first.span.end()
                ),
            }
        }

        // The well-formed class. Parse, format to exactly the declared canonical
        // text, and re-parse to the same graph.
        (None, Some(expected)) => {
            if !report.diagnostics.is_empty() {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: format!(
                        "a valid fixture reported {}",
                        escape(code_name(report.diagnostics[0].code))
                    ),
                };
            }
            let Some(document) = report.document.as_ref() else {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: "a diagnostic-free parse produced no document".to_owned(),
                };
            };
            let formatted = format_document(document, FormatLimits::default());
            let Some(text) = formatted.text.as_ref() else {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: "the document did not format".to_owned(),
                };
            };
            if text != expected {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: format!(
                        "canonical text is {} not the declared {}",
                        escape(text),
                        escape(expected)
                    ),
                };
            }
            let reparsed = parse(text, profile, limits);
            let Some(second) = reparsed.document.as_ref() else {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: "the canonical text did not re-parse at all".to_owned(),
                };
            };
            // Structural identity, which is what the round-trip property means
            // and what tests/property.rs compares. Whole-document equality
            // would also compare source spans, and those legitimately differ
            // between the original source and its canonical rendering — so
            // asserting it would report a drift that is not one.
            if structural(document) != structural(second) {
                return Row {
                    symbol,
                    outcome: "fail",
                    detail: "the canonical text re-parsed to a different graph".to_owned(),
                };
            }
            Row {
                symbol,
                outcome: "pass",
                detail: format!("canonical {} round-trips", escape(text)),
            }
        }
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    let mut manifest_path = None;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--manifest" if index + 1 < arguments.len() => {
                manifest_path = Some(PathBuf::from(&arguments[index + 1]));
                index += 2;
            }
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(manifest_path) = manifest_path else {
        eprintln!("usage: corpus_conformance --manifest <path>");
        return ExitCode::from(2);
    };

    let raw = match fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };
    let manifest: Manifest = match serde_json::from_slice(&raw) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("cannot parse {}: {error}", manifest_path.display());
            return ExitCode::from(2);
        }
    };
    if manifest.schema_version != "tl-parse.corpus/v1" {
        eprintln!("unsupported corpus schema {}", manifest.schema_version);
        return ExitCode::from(2);
    }
    if manifest.revision != tl_parse::CORPUS_REVISION {
        eprintln!(
            "corpus revision {} is not the compiled {}",
            manifest.revision,
            tl_parse::CORPUS_REVISION
        );
        return ExitCode::from(2);
    }
    // A corpus that declares no cases would emit an empty stream, which the
    // chain refuses. Saying so here names the cause instead of leaving the
    // reader to infer it from a downstream refusal.
    if manifest.cases.is_empty() {
        eprintln!("the corpus manifest declares no cases");
        return ExitCode::from(2);
    }

    let corpus = manifest_path
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut failed = 0usize;
    for case in &manifest.cases {
        let row = evaluate(&corpus, case);
        if row.outcome == "fail" {
            failed += 1;
        }
        println!(
            "{{\"protocol\":\"{PROTOCOL}\",\"symbol\":\"{}\",\"outcome\":\"{}\",\
             \"traceIds\":[\"TC-018\",\"FR-005-AC-1\"],\"detail\":\"{}\"}}",
            escape(&row.symbol),
            row.outcome,
            escape(&row.detail)
        );
    }
    // A producer that reported a failing row must exit non-zero. `make conformance`
    // is listed as a CI gate, and a gate whose command always returns 0 is not a
    // gate — it is a print statement. The rows remain the authority for the chain;
    // this is the exit status for the shell.
    if failed > 0 {
        eprintln!("{failed} corpus fixture(s) disagreed with the manifest");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
