use std::{
    env,
    fs::File,
    io::{self, Read},
    process::ExitCode,
};

use tl_parse::{format_document, parse, report_json, Diagnostic, FormatLimits, ParseLimits};
use tl_syntax::SemanticProfile;

const EXIT_INVALID: u8 = 1;
const EXIT_USAGE_OR_IO: u8 = 2;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("tl-parse: {message}");
            ExitCode::from(EXIT_USAGE_OR_IO)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut arguments = env::args().skip(1);
    let command = arguments.next().ok_or_else(usage)?;
    if command != "validate" && command != "format" {
        return Err(usage());
    }

    let mut profile = SemanticProfile::ClosedTraceV1;
    let mut json = false;
    let mut path: Option<String> = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--profile" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| "--profile requires closed or online".to_owned())?;
                profile = match value.as_str() {
                    "closed" => SemanticProfile::ClosedTraceV1,
                    "online" => SemanticProfile::OnlinePrefixV1,
                    _ => return Err("--profile requires closed or online".to_owned()),
                };
            }
            "--json" => json = true,
            "-" => {
                if path.replace(argument).is_some() {
                    return Err("only one input path may be supplied".to_owned());
                }
            }
            _ if argument.starts_with('-') => {
                return Err(format!("unknown option {argument:?}"));
            }
            _ => {
                if path.replace(argument).is_some() {
                    return Err("only one input path may be supplied".to_owned());
                }
            }
        }
    }

    let source = read_bounded(path.as_deref().unwrap_or("-"))?;
    let report = parse(&source, profile, ParseLimits::default());
    if json && (command == "validate" || report.document.is_none()) {
        println!(
            "{}",
            report_json(&report).map_err(|error| format!("cannot serialize report: {error}"))?
        );
    } else if !report.diagnostics.is_empty() {
        for diagnostic in &report.diagnostics {
            eprintln!("{}", render_diagnostic(diagnostic));
        }
        if report.stats.diagnostics_truncated {
            eprintln!("error[diagnostic_limit]: additional diagnostics were suppressed");
        }
    }

    let Some(document) = report.document.as_ref() else {
        return Ok(ExitCode::from(EXIT_INVALID));
    };
    if command == "validate" {
        if !json {
            println!("valid");
        }
        return Ok(ExitCode::SUCCESS);
    }

    let formatted = format_document(document, FormatLimits::default());
    if let Some(text) = formatted.text.as_ref() {
        if json {
            println!(
                "{}",
                serde_json::to_string(&formatted)
                    .map_err(|error| format!("cannot serialize format report: {error}"))?
            );
        } else {
            println!("{text}");
        }
        Ok(ExitCode::SUCCESS)
    } else {
        let error = formatted.error.map_or_else(
            || "unknown formatting failure".to_owned(),
            |error| error.message,
        );
        Err(error)
    }
}

fn read_bounded(path: &str) -> Result<String, String> {
    let byte_limit = (ParseLimits::default().max_source_bytes as u64).saturating_add(1);
    let mut source = String::new();
    if path == "-" {
        io::stdin()
            .lock()
            .take(byte_limit)
            .read_to_string(&mut source)
            .map_err(|error| format!("cannot read UTF-8 stdin: {error}"))?;
    } else {
        File::open(path)
            .map_err(|error| format!("cannot open {path:?}: {error}"))?
            .take(byte_limit)
            .read_to_string(&mut source)
            .map_err(|error| format!("cannot read UTF-8 {path:?}: {error}"))?;
    }
    Ok(source)
}

fn render_diagnostic(diagnostic: &Diagnostic) -> String {
    format!(
        "error[{}] {}..{}: {}",
        diagnostic.code.as_str(),
        diagnostic.span.start(),
        diagnostic.span.end(),
        diagnostic.message
    )
}

fn usage() -> String {
    "usage: tl-parse <validate|format> [--profile <closed|online>] [--json] [FILE|-]".to_owned()
}
