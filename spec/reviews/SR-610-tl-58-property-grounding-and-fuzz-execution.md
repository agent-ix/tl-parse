---
id: SR-610
title: "spec-correctness and spec-fuzz — tl-parse property additions and fuzz execution evidence (TL-58)"
type: SpecReview
analysis: spec-correctness
scope: "tests/property.rs and the four fuzz/fuzz_targets"
review_set: all
---

## Summary

TL-58 adds four independently grounded property tests to `tests/property.rs`
and records a fixed-budget execution of all four cargo-fuzz targets. The
existing parse-format-parse property is unchanged. All four fuzz runs
finished with exit status 0 and no crash, leak or timeout artifact.

The criterion classification baseline is SR-051 (49 criteria, 18 extractable,
one candidate). Quire 0.38.3 now counts 51 criteria, 21 extractable and one
candidate. This review grounds the span, malformed-input and
precedence/associativity laws that SR-051 left as finite witnesses. It does
not re-census the other criteria.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6101 | low | Four generated properties are new: span correctness, malformed-tail rejection with exact code and span, operator associativity, and adjacent-level precedence. Each oracle is independent of the parser: source slicing, the declared span of an appended tail, and the declared grammar in FR-001. | FR-001-AC-1, FR-001-AC-2, FR-002-AC-1, FR-002-AC-3, `tests/property.rs` |
| FND-6102 | low | The remaining extractable criteria keep their finite, corpus or inspection witnesses as recorded in SR-051. They are not recast as generated properties. Marking each criterion property, example-only or excluded inside the requirement documents themselves is not done here: it edits 51 requirement rows and belongs to a specify cycle. | SR-051 FND-4202 |
| FND-6103 | medium | The fuzz runs are not retained as normalized structured campaign evidence. Quoin 0.29.4 was not checked for a libFuzzer adapter in this change, and no raw-output scraping envelope is added. The execution record below is the durable record until a structured producer boundary is used. | tl-syntax#26 |
| FND-6104 | low | LeakSanitizer reports one 48-byte allocation owned by the libFuzzer driver thread (`__thread_struct`). It is suppressed by the existing, narrowly scoped `fuzz/lsan_suppressions.txt` (TL-195), identical to `make fuzz-smoke`. | `fuzz/lsan_suppressions.txt` |

## Property additions

| Test | Law | Criteria |
| --- | --- | --- |
| `generated_formula_spans_are_in_bounds_nested_and_name_their_source_text` | every node has a span; spans are ordered and within the source; each child span lies inside its parent; each proposition span slices to exactly `p<id>` | FR-002-AC-1 |
| `appended_malformed_tails_are_rejected_with_the_right_code_and_span` | a valid formula plus `@`, `)`, ` p0` or `&` yields no document and the declared diagnostic code at the declared half-open byte span | FR-001-AC-2, FR-002-AC-3 |
| `chained_binary_operators_associate_as_declared` | 3 to 7 operand chains of `&`, `\|`, `<->`, `U`, `R` group left and `->` groups right | FR-001-AC-1 |
| `adjacent_precedence_levels_bind_the_tighter_operator_first` | each adjacent pair in `<->`, `->`, `\|`, `&`, `U` binds the tighter operator first in both orders | FR-001-AC-1 |

## Fuzz execution record

Candidate: branch `issue/tl-58-property-fuzz-evidence` on `origin/main`
plus the PR 77 spec fixes. Host: Darwin 25.4.0, arm64, 2026-11-08. Toolchain: rustc
`1.100.0-nightly (1303417c4 2026-09-21)`, cargo-fuzz 0.13.1, libFuzzer via
libfuzzer-sys 0.4.13, quire 0.38.3 (engine 0.52.1), quoin 0.29.4.

Each target ran with `cargo fuzz run <target> -- -seed=20261108
-max_total_time=180`, seeded from the committed `fuzz/corpus/<target>`
directory into an empty generated-corpus directory, with
`ASAN_OPTIONS=detect_leaks=1` and the repository LeakSanitizer suppression.

| Target | Seed | Wall time | Executions | Final cov / features | Generated corpus | Outcome |
| --- | ---: | ---: | ---: | --- | ---: | --- |
| `parser` | 20261108 | 181 s | 2,747,131 | 2126 / 9073 | 922 units | pass, exit 0, no artifact |
| `clean_ascii_v2` | 20261108 | 181 s | 1,457,340 | 2406 / 12564 | 1033 units | pass, exit 0, no artifact |
| `clean_ascii_v3` | 20261108 | 181 s | 2,610,758 | 1778 / 7992 | 1002 units | pass, exit 0, no artifact |
| `unbounded_parse_roundtrip` | 20261108 | 181 s | 3,008,853 | 2654 / 10458 | 1225 units | pass, exit 0, no artifact |

The generated corpora are scratch output and are not committed; the committed
seed corpus under `fuzz/corpus` is unchanged. No crash occurred, so no
regression test was added. Coverage and feature counts are observations from
one 180-second run and do not show a plateau.

## Run report

| Route | Count |
| --- | ---: |
| Property tests emitted | 4 |
| Fuzz targets executed | 4 |
| Crashes turned into regression tests | 0 |
| Criteria not marked in the requirement documents | 51 |
