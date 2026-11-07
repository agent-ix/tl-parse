# tl-parse documentation

tl-parse is a parser, canonical formatter and diagnostic reporter for Mission-time Linear Temporal Logic (MLTL) in ASCII text. This directory documents the dialects it reads, the command-line tool, the diagnostics and the public API. The [README](../README.md) has the overview and the smallest example.

| Page | What it covers |
| --- | --- |
| [how-it-works.md](how-it-works.md) | The lexer, parser and formatter, resource bounds, strict documents, and where the crate sits in the pipeline from text to a Quire outcome, and its step in a debugging session. |
| [dialects.md](dialects.md) | The four dialects: spellings, precedence, profiles and report types. |
| [cli.md](cli.md) | The `tl-parse` binary: commands, options, output and exit codes. |
| [diagnostics.md](diagnostics.md) | Diagnostic codes, spans, recovery actions, limits and format errors. |
| [examples.md](examples.md) | Runnable examples for each dialect, limits, context binding and strict readers. |
| [api-reference.md](api-reference.md) | Public functions, types and constants. |

The normative dialect documents are kept as written:
[v1](DIALECT-001-clean-room-mltl-v1.md), [v2](DIALECT-002-clean-ascii-v2.md), [v3](DIALECT-003-clean-ascii-v3.md) and [v4](DIALECT-004-clean-ascii-v4.md). [ATTRIBUTION.md](ATTRIBUTION.md) records the clean-room authorship boundary.
