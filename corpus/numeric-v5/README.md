# tl-parse numeric v5 hostile corpus

Independently authored for this repository from tl-parse FR-029 to FR-033
(TL-421) and licensed under `MIT OR Apache-2.0`, matching the crate. No third
party grammar or corpus text is copied.

`manifest.json` (`tl-parse.numeric-corpus/v1`) lists each case's id, class,
path, catalog reference (`standard` or `reserved`, built in
`tests/numeric_v5.rs`), profile, optional limits, and either the expected
canonical text or the expected code with its half-open byte span.
The fixtures and manifest are hand-maintained data (the repository is
Rust-only and keeps no generator). When a case is added or edited, its span is
checked by `tests/numeric_v5.rs`, which replays every case against the parser
and fails on any mismatch; the long fixtures (4,096-digit literal, nesting and
chain cases) are plain repetitions described by their `reason`.

Status: UNBUILT. Expectations were derived from the specification and the
parser source and have not yet been confirmed by a run (PLAN-015).
