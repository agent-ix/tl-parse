# tl-parse numeric v5 hostile corpus

Independently authored for this repository from tl-parse FR-029 to FR-033
(TL-421) and licensed under `MIT OR Apache-2.0`, matching the crate. No third
party grammar or corpus text is copied.

`manifest.json` (`tl-parse.numeric-corpus/v1`) lists each case's id, class,
path, catalog reference (`standard` or `reserved`, built in
`tests/numeric_v5.rs`), profile, optional limits, and either the expected
canonical text or the expected code with its half-open byte span.
`generate.py` regenerates the fixtures and computes each span from its locus.

Status: UNBUILT. Expectations were derived from the specification and the
parser source and have not yet been confirmed by a run (PLAN-015).
