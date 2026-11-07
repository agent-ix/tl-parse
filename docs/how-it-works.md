# How it works

## From text to a document

A parse runs three stages over the source string.

1. The lexer splits the text into tokens under the selected dialect. It accepts only the closed vocabulary of that dialect: `false`, `true`, propositions written `p` followed by a canonical unsigned decimal, the operators, brackets, parentheses and ASCII whitespace. Any other character or identifier is a diagnostic.
2. The parser builds tl-syntax nodes directly, using the binding powers of the dialect. Every node carries a half-open byte span into the source.
3. The finished node table is admitted through tl-syntax. A result is a `FormulaDocument` that has passed the tl-syntax validator, so every document the parser returns is also valid for the other TL crates.

A report holds either a document or diagnostics. When any diagnostic is raised, including a resource diagnostic, the document is withheld. The parser continues after most errors by skipping a token or treating a missing token as inserted, so one parse can report several diagnostics; each records the recovery it took.

The caller chooses the dialect by calling one of four entry points and chooses the semantic profile by argument. Nothing in the text selects either.

## Canonical text

`format_document` renders a document as canonical text: no whitespace, the spellings of the dialect, and parentheses only where removing them would change the tree. Prefix operands are parenthesized only when they are infix expressions. `format_clean_ascii_v3` and `format_clean_ascii_v4` do the same for the past and infinite-trace dialects, and `format_formula` takes a borrowed tl-syntax `Formula`.

Formatting is iterative and bounded, so deep graphs do not exhaust the stack. A graph that cannot be written in the dialect, or that exceeds the output limits, returns a `FormatError` and no text.

Parsing canonical text yields a graph with the same profile, root and node kinds as the document it came from. Source spans differ, because the text is different. `examples/roundtrip_sweep.rs` checks this over 20,000 generated sources per profile by default, and also removes parentheses from canonical text to confirm that the check can fail.

## Bounds

Every parse and format call takes limits. `ParseLimits` bounds source bytes, tokens, nodes, nesting depth, retained diagnostics and parser work. `FormatLimits` bounds output bytes and formatter work. Callers can lower each value and cannot raise it past a hard maximum; the effective values are recorded in the report. The values and the diagnostics they raise are in [diagnostics.md](diagnostics.md).

## Reports and strict readers

Each dialect has its own report type with a schema identity. `ParseReport` (v1) serializes with `report_json`. `DerivedParseReport` (v2) adds the lowering records. `PastParseReport` (v3) and `InfiniteParseReport` (v4) have `from_json_bytes`, a strict reader that takes `ParseArtifactLimits` and refuses unknown or duplicate fields, trailing data, non-canonical JSON, wrong identities, counters or spans that do not fit the report, and invalid embedded documents. `parse_with_context` returns a `ContextualParseReport` that binds each free proposition to a signal in a shared tl-syntax signal catalog.

## Where the crate sits

A formula passes through several crates on its way from text to a Quire outcome. tl-parse is the first stage.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="images/pipeline-dark.svg">
  <img alt="A formula moves from text to a Quire outcome. tl-parse turns MLTL text into a formula document, tl-rewrite returns a rewritten document, and tl-mltl evaluates it against a trace. In tests, tl-oracle evaluates the same formula and trace independently and the verdicts are compared. quire-mltl maps the result to a Boolean Value or a typed NonValue reason. All documents use the vocabulary of tl-syntax." src="images/pipeline-light.svg">
</picture>

The calling program moves documents from one stage to the next. The dependency structure between the crates is in the [README](../README.md#the-tl-stack).
