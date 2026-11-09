---
id: SR-231
title: "spec-correctness - cargo-mutants configuration, kill-rate thresholds and coverage reporting (tl-parse, TL-231, TL-234)"
type: SpecReview
analysis: spec-correctness
scope: "mutation configuration, thresholds, scripts and Makefile verification targets of tl-parse"
review_set: all
---

## Summary

TL-231 and TL-234 add a reproducible cargo-mutants configuration, a kill-rate gate, a diff gate and llvm-cov
line and branch coverage reporting to tl-parse, as separate Make targets outside `make ci`. This record is the dated
evidence (2026-11-08 and 2026-11-09); `docs/verification-tooling.md` carries no figures. tl-parse core is at 87.50%, below the 95% target, with 28 survivors and timeouts and no tests added; the crate-wide rate (81.98%) is a core plus shard 0/3 sample, not a full run, and the floors (core 87, crate 78) are the sampled rate minus a margin.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | Mutation measurements (cargo-mutants 27.0.0, `--no-shuffle`, taken on the tree before the final rebase (pre-rebase)): Core (`src/parser.rs`, `src/lexer.rs`): 196 caught, 16 missed, 12 timeout, 16 unviable (87.50%). Crate sample (core plus `--shard 0/3`, 308 mutants): 414 caught, 77 missed, 14 timeout, 43 unviable (81.98%). Timeouts count against the rate. | `mutation-thresholds.toml` |
| FND-2312 | low | The core survivors include the `parse_dialect` conditions, the interval-parsing `!=` mutants, `charge_work` and `advance`, and lexer boundary mutants; the 12 timeouts loop `advance` or the identifier scanner. `minimum` in `src/diagnostic.rs` (`<` to `<=`) is equivalent and is listed in `exclude_re`, not a test gap; it is counted as missed in the figures above. 44 of the missed mutants are in `src/diagnostic.rs`. | `docs/verification-tooling.md` |
| FND-2313 | low | The enforced mutation floors are the measured sampled rates minus a margin, not the 95% and 85% targets; groups below target have open survivors, listed below, that are test gaps to kill. | `mutation-thresholds.toml` |
| FND-2314 | low | Every gate was shown able to fail: `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate, on an empty population or a failed baseline, and exits 2 on unreadable input; `make coverage COVERAGE_MIN_LINES=99` exits non-zero. `scripts/run_mutants.py` accepts cargo-mutants exit 0, 2 and 3 so the gate always prints its table. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage re-measured from a clean `target/llvm-cov-target` on 2026-11-09, all features, `ci_guard` excluded. Stable `make coverage`: 87.78% lines / 86.43% regions / 84.42% functions. Nightly `make coverage-branch`: 83.27% lines / 58.99% regions / 83.90% functions / 86.27% branches. The two toolchains report different line totals and the nightly region figure is much lower, so `COVERAGE_MIN_LINES` is 80, below the lower line figure; both targets pass at that floor. A reviewer measured lower totals on a fresh clone than these; the difference could not be reproduced here, so the floor sits below both readings that were available. | `Makefile` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro), heavily loaded (load average 35 to 45). Toolchain: rustc 1.98.1,
nightly rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3
(engine 0.52.1). Mutation order is fixed by `--no-shuffle`; there is no seed in this mode. The per-mutant timeout
is the cargo-mutants automatic value (three times the unmutated baseline, minimum 60 s in this crate's
`.cargo/mutants.toml`). Wall times are not equality inputs. The crate-wide rates are core plus `--shard 0/3` file
samples (cargo-mutants shards by file), not full runs.

## Open survivors and timeouts of the measured populations

Each is a test gap to kill, unless listed as equivalent above.

- `src/bin/tl-parse.rs:157:25: replace > with >= in read_bounded`
- `src/bin/tl-parse.rs:175:21: replace > with >= in read_bounded_reader`
- `src/context.rs:131:9: replace <impl fmt::Display for ContextualParseError>::fmt -> fmt::Result with Ok(Default::default())`
- `src/diagnostic.rs:120:13: replace < with <= in minimum`
- `src/diagnostic.rs:147:9: replace <impl fmt::Display for StrictParseArtifactReadError>::fmt -> fmt::Result with Ok(Default::default())`
- `src/diagnostic.rs:206:26: replace > with == in preflight_parse_artifact`
- `src/diagnostic.rs:206:26: replace > with >= in preflight_parse_artifact`
- `src/diagnostic.rs:213:13: delete match arm b'}' | b']' in preflight_parse_artifact`
- `src/diagnostic.rs:226:5: replace scan_json_string -> Result<usize, StrictParseArtifactReadError> with Ok(0)`
- `src/diagnostic.rs:226:5: replace scan_json_string -> Result<usize, StrictParseArtifactReadError> with Ok(1)`
- `src/diagnostic.rs:227:17: replace < with <= in scan_json_string`
- `src/diagnostic.rs:230:13: delete match arm b'\\' in scan_json_string`
- `src/diagnostic.rs:232:26: replace >= with < in scan_json_string`
- `src/diagnostic.rs:235:33: replace == with != in scan_json_string`
- `src/diagnostic.rs:245:26: replace > with == in scan_json_string`
- `src/diagnostic.rs:245:26: replace > with >= in scan_json_string`
- `src/diagnostic.rs:257:5: replace decoded_unicode_escape -> (usize, usize) with (0, 0)`
- `src/diagnostic.rs:257:5: replace decoded_unicode_escape -> (usize, usize) with (0, 1)`
- `src/diagnostic.rs:257:5: replace decoded_unicode_escape -> (usize, usize) with (1, 0)`
- `src/diagnostic.rs:257:5: replace decoded_unicode_escape -> (usize, usize) with (1, 1)`
- `src/diagnostic.rs:25:53: replace * with +`
- `src/diagnostic.rs:25:60: replace * with +`
- `src/diagnostic.rs:261:49: replace == with != in decoded_unicode_escape`
- `src/diagnostic.rs:261:9: replace && with || in decoded_unicode_escape`
- `src/diagnostic.rs:262:49: replace == with != in decoded_unicode_escape`
- `src/diagnostic.rs:262:9: replace && with || in decoded_unicode_escape`
- `src/diagnostic.rs:263:9: replace && with || in decoded_unicode_escape`
- `src/diagnostic.rs:269:9: delete match arm 0x0000..= 0x007F in decoded_unicode_escape`
- `src/diagnostic.rs:270:9: delete match arm 0x0080..= 0x07FF in decoded_unicode_escape`
- `src/diagnostic.rs:277:5: replace parse_hex_quad -> Option<u16> with None`
- `src/diagnostic.rs:277:5: replace parse_hex_quad -> Option<u16> with Some(0)`
- `src/diagnostic.rs:277:5: replace parse_hex_quad -> Option<u16> with Some(1)`
- `src/diagnostic.rs:280:13: delete match arm b'0'..= b'9' in parse_hex_quad`
- `src/diagnostic.rs:280:44: replace - with + in parse_hex_quad`
- `src/diagnostic.rs:280:44: replace - with / in parse_hex_quad`
- `src/diagnostic.rs:281:13: delete match arm b'a'..= b'f' in parse_hex_quad`
- `src/diagnostic.rs:281:44: replace - with + in parse_hex_quad`
- `src/diagnostic.rs:281:44: replace - with / in parse_hex_quad`
- `src/diagnostic.rs:281:51: replace + with - in parse_hex_quad`
- `src/diagnostic.rs:282:13: delete match arm b'A'..= b'F' in parse_hex_quad`
- `src/diagnostic.rs:282:44: replace - with + in parse_hex_quad`
- `src/diagnostic.rs:282:44: replace - with / in parse_hex_quad`
- `src/diagnostic.rs:282:51: replace + with - in parse_hex_quad`
- `src/diagnostic.rs:29:60: replace * with +`
- `src/diagnostic.rs:31:53: replace * with +`
- `src/diagnostic.rs:31:60: replace * with +`
- `src/diagnostic.rs:419:31: replace match guard previous.code.v4_refusal_priority() >= code.v4_refusal_priority() with false in InfiniteRefusal::accumulate`
- `src/dialect/mod.rs:104:9: replace Dialect::permits_lowering -> bool with true`
- `src/dialect/mod.rs:81:9: replace Dialect::permits_temporal_prefix -> bool with true`
- `src/formatter.rs:13:9: delete match arm (FormulaSchemaVersion::V2, SemanticProfile::OriginCompleteHistoryV1) in format_document`
- `src/formatter.rs:37:9: replace || with && in format_clean_ascii_v3`
- `src/infinite.rs:149:44: replace > with >= in <impl TryFrom<InfiniteParseReportWire> for InfiniteParseReport>::try_from`
- `src/infinite.rs:150:41: replace > with >= in <impl TryFrom<InfiniteParseReportWire> for InfiniteParseReport>::try_from`
- `src/infinite.rs:224:21: replace match guard self.document.is_some() with true in InfiniteParseReport::disposition`
- `src/infinite.rs:482:37: replace < with <= in parse_clean_ascii_v4`
- `src/infinite.rs:482:37: replace < with == in parse_clean_ascii_v4`
- `src/infinite.rs:482:37: replace < with > in parse_clean_ascii_v4`
- `src/lexer.rs:258:29: replace += with *= in Lexer<'_>::scan_token`
- `src/lexer.rs:307:27: replace match guard !byte.is_ascii_alphanumeric() && *byte != b'_' with false in Lexer<'_>::atom_keyword_boundary`
- `src/lexer.rs:307:66: replace != with == in Lexer<'_>::atom_keyword_boundary`
- `src/lexer.rs:328:29: replace += with *= in Lexer<'_>::scan_identifier`
- `src/lexer.rs:340:16: delete ! in Lexer<'_>::scan_identifier`
- `src/lexer.rs:340:35: replace && with || in Lexer<'_>::scan_identifier`
- `src/lexer.rs:378:27: replace < with <= in Lexer<'_>::scan_number`
- `src/lexer.rs:381:25: replace += with *= in Lexer<'_>::scan_number`
- `src/lexer.rs:65:35: replace > with == in Token::found`
- `src/lexer.rs:65:35: replace > with >= in Token::found`
- `src/lib.rs:110:5: replace dialect_digest -> String with "xyzzy".into()`
- `src/lib.rs:110:5: replace dialect_digest -> String with String::new()`
- `src/lib.rs:117:5: replace dialect_v2_digest -> String with "xyzzy".into()`
- `src/lib.rs:117:5: replace dialect_v2_digest -> String with String::new()`
- `src/lib.rs:124:5: replace dialect_v3_digest -> String with "xyzzy".into()`
- `src/lib.rs:124:5: replace dialect_v3_digest -> String with String::new()`
- `src/parser.rs:119:39: replace || with && in parse_dialect`
- `src/parser.rs:119:59: replace || with && in parse_dialect`
- `src/parser.rs:142:25: replace match guard admitted == document with true in strictly_admit_document`
- `src/parser.rs:149:17: delete match arm StrictDocumentReadError::ResourceLimitExceeded{resource:"formula nodes", ..} in strictly_admit_document`
- `src/parser.rs:158:17: delete match arm StrictDocumentReadError::WorkLimitExceeded{..} in strictly_admit_document`
- `src/parser.rs:342:27: replace != with == in Parser<'_>::parse_prefix`
- `src/parser.rs:457:35: replace == with != in Parser<'_>::parse_prefix`
- `src/parser.rs:464:31: replace != with == in Parser<'_>::parse_prefix`
- `src/parser.rs:504:33: replace != with == in Parser<'_>::parse_interval`
- `src/parser.rs:539:31: replace != with == in Parser<'_>::parse_interval`
- `src/parser.rs:66:51: replace && with || in parse_dialect`
- `src/parser.rs:781:9: replace Parser<'_>::charge_work -> bool with false`
- `src/parser.rs:803:44: replace > with >= in Parser<'_>::charge_work_units`
- `src/parser.rs:827:31: replace && with || in Parser<'_>::advance`
- `src/parser.rs:827:46: replace + with * in Parser<'_>::advance`
- `src/parser.rs:827:50: replace < with <= in Parser<'_>::advance`
- `src/parser.rs:827:9: replace Parser<'_>::advance with ()`
- `src/parser.rs:92:26: replace && with || in parse_dialect`
