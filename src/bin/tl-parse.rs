use std::{
    env,
    fs::File,
    io::{self, Read, Write},
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
        BoundedInput::SourceLimit {
            source_bytes,
            exact,
        } => {
            let limits = ParseLimits::default();
            let mut report = source_limit_report(source_bytes, profile, limits);
            if !exact {
                if let Some(diagnostic) = report.diagnostics.first_mut() {
                    diagnostic.message = format!(
                        "source length is at least {source_bytes}, exceeding effective limit {}",
                        limits.max_source_bytes
                    );
                }
            }
            report
        }
    };
    if json && (command == "validate" || report.document.is_none()) {
        write_stdout(
            &report_json(&report).map_err(|error| format!("cannot serialize report: {error}"))?,
        )?;
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
            write_stdout("valid")?;
        }
        return Ok(ExitCode::SUCCESS);
    }

    let formatted = format_document(document, FormatLimits::default());
    if let Some(text) = formatted.text.as_ref() {
        if json {
            write_stdout(
                &serde_json::to_string(&formatted)
                    .map_err(|error| format!("cannot serialize format report: {error}"))?,
            )?;
        } else {
            write_stdout(text)?;
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

fn write_stdout(text: &str) -> Result<(), String> {
    write_output(&mut io::stdout().lock(), text)
}

fn write_output(writer: &mut impl Write, text: &str) -> Result<(), String> {
    match writeln!(writer, "{text}") {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(error) => Err(format!("cannot write stdout: {error}")),
    }
}

enum BoundedInput {
    Source(String),
    SourceLimit { source_bytes: usize, exact: bool },
}

fn read_bounded(path: &str) -> Result<BoundedInput, String> {
    let byte_limit = ParseLimits::default().max_source_bytes;
    if path == "-" {
        read_bounded_reader(io::stdin().lock(), byte_limit)
            .map_err(|error| format!("cannot read stdin: {error}"))
    } else {
        let file = File::open(path).map_err(|error| format!("cannot open {path:?}: {error}"))?;
        let source_bytes = file
            .metadata()
            .map_err(|error| format!("cannot inspect {path:?}: {error}"))?
            .len();
        if source_bytes > byte_limit as u64 {
            return Ok(BoundedInput::SourceLimit {
                source_bytes: usize::try_from(source_bytes).unwrap_or(usize::MAX),
                exact: true,
            });
        }
        read_bounded_reader(file, byte_limit)
            .map_err(|error| format!("cannot read {path:?}: {error}"))
    }
}

fn read_bounded_reader(reader: impl Read, byte_limit: usize) -> io::Result<BoundedInput> {
    let read_limit = byte_limit.saturating_add(1);
    let mut retained = Vec::with_capacity(read_limit.min(64 * 1024));
    reader.take(read_limit as u64).read_to_end(&mut retained)?;
    let source_bytes = retained.len();
    if source_bytes > byte_limit {
        return Ok(BoundedInput::SourceLimit {
            source_bytes,
            exact: false,
        });
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

#[cfg(test)]
mod tests {
    use std::io::{self, Read, Write};

    use super::{read_bounded_reader, write_output, BoundedInput};

    struct NonClosingReader {
        reads: usize,
    }

    impl Read for NonClosingReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            self.reads += 1;
            buffer.fill(b'p');
            Ok(buffer.len())
        }
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Other, "fixture read failure"))
        }
    }

    struct FailingWriter(io::ErrorKind);

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(self.0, "fixture write failure"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // Trace: TC-021, FR-005-AC-3, NFR-001-AC-1
    #[test]
    fn bounded_reader_stops_without_waiting_for_eof() {
        let input = read_bounded_reader(NonClosingReader { reads: 0 }, 4_096).unwrap();
        match input {
            BoundedInput::SourceLimit {
                source_bytes,
                exact,
            } => {
                assert_eq!(source_bytes, 4_097);
                assert!(!exact);
            }
            BoundedInput::Source(_) => panic!("non-closing oversized input was accepted"),
        }
    }

    // Trace: TC-021, FR-005-AC-3, NFR-001-AC-1
    #[test]
    fn reader_and_writer_errors_keep_their_fail_closed_classes() {
        let error = match read_bounded_reader(FailingReader, 32) {
            Err(error) => error,
            Ok(_) => panic!("failing reader unexpectedly succeeded"),
        };
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(error.to_string(), "fixture read failure");

        assert_eq!(
            write_output(&mut FailingWriter(io::ErrorKind::BrokenPipe), "p0"),
            Ok(())
        );
        let error = write_output(&mut FailingWriter(io::ErrorKind::Other), "p0").unwrap_err();
        assert_eq!(error, "cannot write stdout: fixture write failure");
    }
}
