---
id: SR-044
title: "Gap analysis — tl-parse property grounding and fuzz evidence"
type: SpecReview
analysis: gap-analysis
scope: "tl-parse#25, spec/requirements, spec/test-matrix.md, tests, fuzz, and SR-042 through SR-043"
review_set: subset
---

## Summary

The property-grounding portion of tl-parse#25 is complete. All 49 binding
criteria have an explicit grounding disposition in SR-042, all 93 matrix rows
are backed, and all 59 Rust candidates are compiled, tagged and bound. No
duplicate generated test was added over existing hand-written evidence.

The bounded execution portion is demonstrated but not durably normalized: both
64-run fuzz targets passed at `9ca856b`, while the shared stack exposes no
libFuzzer result adapter and the repository is forbidden to invent a generic
collector or stdout-derived evidence envelope. The ticket is therefore ready to
merge as an exact review/baseline increment but is not ready to close as a
retained-campaign deliverable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-4401 | medium | Durable campaign retention remains open: the exact run is recorded in SR-042, but there is no released structured libFuzzer producer-result contract or Quoin adapter that can retain it without a local substitute. | tl-parse#25, tl-syntax#26, SR-042 FND-4203, SR-043 FND-4302 |
| FND-4402 | low | Traceability closes exactly: Quire reports 93/93 rows backed, 49/49 criteria backed, and 59/59/59 Rust bound/tagged/candidates with no status lie, unbacked row, or untracked symbol. | TM-001, `quire coverage --scope . --strict` |

## Reconciliation

| Item | Complete | Outstanding |
| --- | ---: | ---: |
| Binding criteria grounded | 49 | 0 |
| Extractable/candidate criteria dispositioned | 19 | 0 |
| Matrix rows backed | 93 | 0 |
| Rust trace symbols bound | 59 | 0 |
| Checked fuzz targets executed at the candidate | 2 | 0 |
| Fuzz campaigns retained through a released structured shared contract | 0 | 2 |

The outstanding count is a representation/retention gap, not a claim that the
two fuzz executions failed. The parent epic owns the later plateau, Kani or
concolic decision, and mutation campaign; this issue does not pre-empt those
measurements.
