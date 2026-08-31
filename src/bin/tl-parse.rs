use std::{
    env,
    fs::File,
    io::{self, Read},
    process::ExitCode,
};

use tl_parse::{
    format_document, parse, report_json, source_limit_report, Diagnostic, FormatLimits, ParseLimits,
};
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

    let input = read_bounded(path.as_deref().unwrap_or("-"))?;
    let report = match input {
        BoundedInput::Source(source) => parse(&source, profile, ParseLimits::default()),
        BoundedInput::SourceLimit { source_bytes } => {
            source_limit_report(source_bytes, profile, ParseLimits::default())
        }
    };
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

enum BoundedInput {
    Source(String),
    SourceLimit { source_bytes: usize },
}

fn read_bounded(path: &str) -> Result<BoundedInput, String> {
    let byte_limit = ParseLimits::default().max_source_bytes;
    if path == "-" {
        read_bounded_reader(io::stdin().lock(), byte_limit)
            .map_err(|error| format!("cannot read stdin: {error}"))
    } else {
        let file = File::open(path).map_err(|error| format!("cannot open {path:?}: {error}"))?;
        read_bounded_reader(file, byte_limit)
            .map_err(|error| format!("cannot read {path:?}: {error}"))
    }
}

fn read_bounded_reader(mut reader: impl Read, byte_limit: usize) -> io::Result<BoundedInput> {
    let mut retained = Vec::with_capacity(byte_limit.min(64 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    let mut source_bytes = 0_usize;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        source_bytes = source_bytes.checked_add(read).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "input byte count overflowed usize",
            )
        })?;
        if source_bytes <= byte_limit {
            retained.extend_from_slice(&buffer[..read]);
        } else {
            retained.clear();
        }
    }
    if source_bytes > byte_limit {
        return Ok(BoundedInput::SourceLimit { source_bytes });
    }
    String::from_utf8(retained)
        .map(BoundedInput::Source)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
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
