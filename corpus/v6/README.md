# tl-parse v6 hostile corpus

Independently authored for this repository from tl-parse FR-061 (TL-429) and
licensed under `MIT OR Apache-2.0`. The specification names the dialect v5;
the TL program decision of 2026-11-23 renumbered it to `tl-parse.clean-ascii/v6`.

`manifest.json` lists 29 cases, each with an expected outcome per profile
(`ltlf`, `infinite`): canonical text or code with half-open byte span.
The fixtures, manifest and the 58 `clean_ascii_v6` fuzz seeds (each case once
per profile bit: `<id>-ltlf.txt` starts with byte 0, `<id>-infinite.txt` with
byte 1) are hand-maintained data; `tests/clean_ascii_v6.rs` checks that every
seed equals its case and that every case matches the parser.

Status: UNBUILT. Expectations were derived from the specification and the
parser source and have not yet been confirmed by a run (PLAN-045).
