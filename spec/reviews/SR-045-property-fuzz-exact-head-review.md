---
id: SR-045
title: "Exact-head review — tl-parse property and fuzz baseline"
type: SpecReview
analysis: code-review
scope: "candidate 19e48c540360d24e8779e760f98d9a8e5898a73b and tl-parse#25"
review_set: subset
---

## Summary

The complete local gate passes at the exact candidate containing SR-042 through
SR-044. The candidate changes only validated review artifacts; the reviewed
Rust/property/fuzz implementation remains byte-identical to current main
`9ca856b`. No hosted workflow was dispatched.

`make ci` passes 57 Rust tests, the 20,000-source/two-profile round-trip sweep
(40,000 checks, zero drift, negative control 2390/2390), both bounded cargo-fuzz
targets with LeakSanitizer enabled, corpus/checksum validation, strict Clippy,
cargo-deny, unsafe audit, Quire validation/coverage, Rust 1.75 MSRV, rustdoc,
shared pin admission and the complete assurance chain.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4501 | low | Exact-head verification is clean: 91/91 documents validate, all 93 matrix rows and 59 Rust symbols bind, and both 64-run fuzz targets complete without a crash artifact. | `19e48c5`, TM-001, SR-042 through SR-044 |
| FND-4502 | medium | The run remains review evidence rather than normalized retained Quoin evidence until a released structured libFuzzer producer-result/adapter boundary exists. No local substitute was introduced. | SR-042 FND-4203, SR-043 FND-4302, SR-044 FND-4401 |

## Exact-head fuzz observations

| Target | Checked seeds | Runs | Final coverage/features | Peak RSS | Outcome |
| --- | ---: | ---: | --- | ---: | --- |
| `parser` | 4 | 64 | 988 / 2248 | 39 MiB | pass; no crash artifact |
| `clean_ascii_v2` | 5 | 64 | 1257 / 3530 | 45 MiB | pass; no crash artifact |

These nondeterministic smoke observations are not compared as a trend or called
a plateau. The later measured campaign and any Kani/concolic or mutation choice
remain with the parent epic.
