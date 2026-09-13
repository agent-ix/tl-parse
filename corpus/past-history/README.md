# Paired past/history corpus

`tl-syntax.past-history-corpus/v1` is the immutable interoperability corpus for
the origin-complete O/H/Y/S/T profile. It is evidence input, not an alternate
semantic authority and not qualification output.

`cases.json` pairs clean-ascii/v3 source with canonical formula-v2 graphs,
origin-complete event and fixed-sample histories, anchored outcomes,
required-history bounds, correction identities, rewrite expectations, refusal
classes, and closed external-target dispositions. `schema.json` is closed at
every record boundary. `manifest.json` pins the exact implementation revisions
and every replay file; `SHA256SUMS` provides the same file-integrity boundary to
non-Rust consumers.

The parser, evaluator, and rewrite repositories retain byte-identical copies
and pin the manifest digest. Their native replay tests consume only the fields
owned by that component and still validate the complete closed corpus shape.

FRET remains output-only. R2U2 and C2PO remain unavailable until an exact
reviewed past-profile correspondence exists. Isabelle/HOL remains an
intentionally unsupported future-only oracle. None is downloaded or executed.
