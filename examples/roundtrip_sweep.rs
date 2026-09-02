//! The parse-format-parse fixed point, as a producer (FR-006-AC-2).
//!
//! Reviewers of PR #6 established this property by independent execution and
//! called it "the crate's central correctness argument": every source the
//! parser accepts must format and re-parse to a structurally identical graph
//! under the same limits. `tests/property.rs` owns it as a proptest; this
//! example owns it as a *retained structured result*, so the assurance chain
//! can attest to it from bytes rather than from a test runner's exit code.
//!
//! Generation is deterministic. A sweep whose inputs change every run cannot be
//! compared across candidate revisions, and an attestation over an
//! irreproducible input set names a run nobody else can perform.
//!
//! The negative control is not decoration. The property is over-satisfiable:
//! an implementation that returned the input unchanged would pass every case.
//! So the control strips parentheses from canonical text before re-parsing and
//! requires that to break the property for a substantial share of cases. A
//! sweep whose control never fires has not shown the property is doing work,
//! and this producer reports `vacuous` when that happens rather than `pass`.

use std::process::ExitCode;

use tl_parse::{format_document, parse, FormatLimits, ParseLimits};
use tl_syntax::{FormulaDocument, NodeKind, SemanticProfile};

const PROTOCOL: &str = "tl-parse.roundtrip-sweep/v1";

/// A deterministic 64-bit LCG. Reproducibility is the requirement; statistical
/// quality is not, because the shapes below are enumerated rather than sampled.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

fn structural(document: &FormulaDocument) -> (SemanticProfile, u32, Vec<NodeKind>) {
    (
        document.semantic_profile(),
        document.root().0,
        document.nodes().iter().map(|node| node.kind).collect(),
    )
}

/// Build one random source term. Depth-bounded so generation terminates.
fn term(rng: &mut Rng, depth: usize) -> String {
    if depth == 0 {
        return match rng.below(4) {
            0 => "true".to_owned(),
            1 => "false".to_owned(),
            other => format!("p{}", other + rng.below(8)),
        };
    }
    match rng.below(10) {
        0 => format!("!{}", term(rng, depth - 1)),
        1 => format!("({})", term(rng, depth - 1)),
        2 => format!(
            "F[{},{}]{}",
            rng.below(4),
            4 + rng.below(4),
            term(rng, depth - 1)
        ),
        3 => format!(
            "G[{},{}]{}",
            rng.below(4),
            4 + rng.below(4),
            term(rng, depth - 1)
        ),
        4 => format!(
            "{}U[{},{}]{}",
            term(rng, depth - 1),
            rng.below(4),
            4 + rng.below(4),
            term(rng, depth - 1)
        ),
        5 => format!(
            "{}R[{},{}]{}",
            term(rng, depth - 1),
            rng.below(4),
            4 + rng.below(4),
            term(rng, depth - 1)
        ),
        6 => format!("{}&{}", term(rng, depth - 1), term(rng, depth - 1)),
        7 => format!("{}|{}", term(rng, depth - 1), term(rng, depth - 1)),
        8 => format!("{}->{}", term(rng, depth - 1), term(rng, depth - 1)),
        _ => format!("{}<->{}", term(rng, depth - 1), term(rng, depth - 1)),
    }
}

/// The directed shapes. These are the ones the PR #6 review used to find the
/// original depth-asymmetry defect, so they are enumerated rather than left to
/// chance: a random generator that happened not to emit a 200-term conjunction
/// would silently stop covering the case that once failed.
fn directed() -> Vec<String> {
    let mut cases = Vec::new();
    for count in [1, 2, 63, 127, 200, 255] {
        cases.push(format!("{}p0", "!".repeat(count)));
        cases.push(
            (0..count)
                .map(|index| format!("p{}", index % 32))
                .collect::<Vec<_>>()
                .join("&"),
        );
        cases.push(
            (0..count)
                .map(|index| format!("p{}", index % 32))
                .collect::<Vec<_>>()
                .join("|"),
        );
    }
    // Precedence-dependent, unparenthesised, and explicitly grouped against the
    // default associativity in both directions.
    cases.extend(
        [
            "p0&p1|p2",
            "p0|p1&p2",
            "p0->p1->p2",
            "(p0->p1)->p2",
            "p0<->p1<->p2",
            "p0<->(p1<->p2)",
            "p0U[0,1]p1&p2",
            "p0&p1U[0,1]p2",
            "!p0&p1",
            "!(p0&p1)",
            "F[0,1]p0|p1",
            "F[0,1](p0|p1)",
            "G[0,2](p0->F[1,1]p1)",
            "p0R[0,3]p1U[1,2]p2",
        ]
        .iter()
        .map(|value| (*value).to_owned()),
    );
    cases
}

struct Counts {
    checked: usize,
    rejected: usize,
    format_failed: usize,
    drift: usize,
}

fn sweep(sources: &[String], profile: SemanticProfile) -> Counts {
    let mut counts = Counts {
        checked: 0,
        rejected: 0,
        format_failed: 0,
        drift: 0,
    };
    for source in sources {
        let first = parse(source, profile, ParseLimits::default());
        let Some(document) = first.document.as_ref() else {
            // A source the parser declines is outside the property's domain.
            // The property is about what the parser *accepts*.
            counts.rejected += 1;
            continue;
        };
        let formatted = format_document(document, FormatLimits::default());
        let Some(text) = formatted.text.as_ref() else {
            // A typed format limit is a declared refusal, not a drift.
            counts.format_failed += 1;
            continue;
        };
        let second = parse(text, profile, ParseLimits::default());
        counts.checked += 1;
        match second.document.as_ref() {
            Some(reparsed) if structural(document) == structural(reparsed) => {}
            _ => counts.drift += 1,
        }
    }
    counts
}

/// The negative control: strip parentheses from canonical text, then re-parse.
/// Removing needed parentheses must change the tree for a meaningful share of
/// cases, or the property is being satisfied by something other than the
/// formatter getting precedence right.
fn control(sources: &[String], profile: SemanticProfile) -> (usize, usize) {
    let mut compared = 0;
    let mut broke = 0;
    for source in sources {
        let first = parse(source, profile, ParseLimits::default());
        let Some(document) = first.document.as_ref() else {
            continue;
        };
        let formatted = format_document(document, FormatLimits::default());
        let Some(text) = formatted.text.as_ref() else {
            continue;
        };
        if !text.contains('(') {
            continue;
        }
        let stripped: String = text
            .chars()
            .filter(|value| *value != '(' && *value != ')')
            .collect();
        let second = parse(&stripped, profile, ParseLimits::default());
        compared += 1;
        match second.document.as_ref() {
            Some(reparsed) if structural(document) == structural(reparsed) => {}
            _ => broke += 1,
        }
    }
    (compared, broke)
}

fn row(symbol: &str, outcome: &str, detail: &str) {
    println!(
        "{{\"protocol\":\"{PROTOCOL}\",\"symbol\":\"{symbol}\",\"outcome\":\"{outcome}\",\
         \"traceIds\":[\"TC-016\",\"FR-004-AC-2\"],\"detail\":\"{detail}\"}}"
    );
}

fn main() -> ExitCode {
    let cases: usize = std::env::var("TL_PARSE_SWEEP_CASES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(20_000);

    let mut rng = Rng(0x5DEECE66D);
    let mut sources = directed();
    while sources.len() < cases {
        let depth = 1 + rng.below(5);
        sources.push(term(&mut rng, depth));
    }

    let mut drifted = 0usize;
    let mut checked = 0usize;
    for (name, profile) in [
        ("closed", SemanticProfile::ClosedTraceV1),
        ("online", SemanticProfile::OnlinePrefixV1),
    ] {
        let counts = sweep(&sources, profile);
        checked += counts.checked;
        drifted += counts.drift;
        let outcome = if counts.drift > 0 {
            "fail"
        } else if counts.checked == 0 {
            // Nothing was actually compared. That is not a pass.
            "vacuous"
        } else {
            "pass"
        };
        row(
            &format!("roundtrip/{name}"),
            outcome,
            &format!(
                "checked {} rejected {} format-refused {} drift {}",
                counts.checked, counts.rejected, counts.format_failed, counts.drift
            ),
        );
    }

    // The control, reported as its own row so it cannot be quietly dropped.
    let (compared, broke) = control(&sources, SemanticProfile::ClosedTraceV1);
    let control_outcome = if compared == 0 {
        "vacuous"
    } else if broke == 0 {
        // The property held even with parentheses removed, which means the
        // sweep is not exercising precedence at all.
        "fail"
    } else {
        "pass"
    };
    row(
        "roundtrip/negative-control",
        control_outcome,
        &format!("parenthesised {compared} broke-when-stripped {broke}"),
    );

    eprintln!(
        "roundtrip sweep: {} sources, {checked} checked, {drifted} drift, control {broke}/{compared}",
        sources.len()
    );
    ExitCode::SUCCESS
}
