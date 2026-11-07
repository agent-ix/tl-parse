# Command-line tool

`tl-parse` is a thin wrapper over the v1 parser and formatter. It validates a formula or rewrites it to canonical text. The v2, v3 and v4 dialects are available only through the library.

```text
tl-parse <validate|format> [--profile <closed|online>] [--json] [FILE|-]
```

Run it from a checkout with `cargo run --bin tl-parse --`, or install the binary built by `cargo build --release`.

## Commands

| Command | Behavior |
| --- | --- |
| `validate` | Parses the input. Prints `valid` and exits 0, or prints the diagnostics and exits 1. |
| `format` | Parses the input and prints canonical text, or prints the diagnostics and exits 1. |

## Options

| Option | Meaning |
| --- | --- |
| `--profile closed` | Semantic profile `mltl.closed-trace/v1`. This is the default. |
| `--profile online` | Semantic profile `mltl.online-prefix/v1`. |
| `--json` | Machine-readable output, described below. |
| `FILE` or `-` | The input. Without an argument, or with `-`, the tool reads standard input. One input only. |

The input must be UTF-8 and at most 1,048,576 bytes. A longer input is refused with a `source_limit` diagnostic without being read in full.

## Output

Without `--json`, diagnostics go to standard error, one per line:

```text
error[invalid_interval] 4..9: inclusive interval start 3 exceeds end 1
```

The numbers are the half-open byte span of the diagnostic. If more diagnostics were raised than the retained limit, a final `error[diagnostic_limit]` line says so.

With `--json`:

- `validate` prints the whole parse report as one JSON line, on success and on failure. The schema is `tl-parse.diagnostics/v1`.
- `format` prints the format report as one JSON line on success. When parsing fails it prints the parse report instead.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | The input is valid, and for `format` the text was written. |
| 1 | The input was refused. The diagnostics are in the output. |
| 2 | Bad usage, an unreadable file, invalid UTF-8 or a formatting failure. A message beginning `tl-parse:` goes to standard error. |

A closed standard output pipe is not an error.

## Examples

```sh
printf 'p0 U[1,2] true' | tl-parse format --profile online -
# p0U[1,2]true

printf 'G[0,2](p0 -> F[1,1] p1)' | tl-parse validate -
# valid

printf 'p0 U[3,1] p1' | tl-parse validate -
# error[invalid_interval] 4..9: inclusive interval start 3 exceeds end 1
# error[trailing_input] 10..12: unexpected trailing token "p1"; expected end of input
```

The last command exits 1. `tests/cli.rs` covers these paths and the exit classes.
