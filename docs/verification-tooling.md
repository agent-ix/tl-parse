# Verification tooling

How this crate's mutation, coverage evidence is produced and gated (TL-231, TL-234). The measured numbers below come from a
single machine (Apple M5 Pro, Darwin 25.4.0, rustc 1.98.1, cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7,
nightly rustc 1.100.0-nightly 2026-09-21 for branch coverage) that was heavily loaded by other jobs.

## Targets

| Target | What it does | Budget |
| --- | --- | --- |
| `make mutants` | Full cargo-mutants run (`--no-shuffle`, so the population and order are fixed), then `scripts/mutation_gate.py` against `mutation-thresholds.toml` | hours; weekly, not part of `make ci` |
| `make mutants-diff` | cargo-mutants on `git diff $(MUTANTS_BASE)...HEAD` only (default `origin/main`); every mutant in the diff must be killed | minutes to an hour, per PR |
| `make coverage` | `cargo llvm-cov` line, function and region coverage on the pinned toolchain with a line floor (`COVERAGE_MIN_LINES`) | about 1 to 5 minutes |
| `make coverage-branch` | the same plus branch coverage (unstable, needs nightly with llvm-tools) | about 1 to 5 minutes |


`CI-plumbing` modules are excluded from every threshold (`.cargo/mutants.toml` `exclude_globs` and
`exclude` in `mutation-thresholds.toml`). Equivalent mutants are listed in `exclude_re` with a one-line
reason. `cargo mutants` tests every mutant against the whole test suite, so a full run costs
(mutants x suite time) divided by `MUTANTS_JOBS`; use `MUTANTS_JOBS` to trade memory for time.

## Mutation thresholds

Kill rate is `caught / (caught + missed + timeout)`; unviable mutants count on neither side. The campaign
targets are 95% for the core and 85% for the whole crate. The values enforced by the gate are the measured
floors (they only ratchet up).

| Group | Files | Enforced floor |
| --- | --- | --- |
| `core` | `["src/parser.rs", "src/lexer.rs"]` | 87.0% |
| `crate` | `["src/*.rs", "src/*/*.rs"]` | 78.0% |

Measured at 5654cf4 on this machine: core run: 240 mutants (196 caught, 16 missed, 12 timeout, 16 unviable = 87.50%). Crate sample: core plus `--shard 0/3` (308 mutants: `src/diagnostic.rs`, `src/infinite.rs`, `src/formatter.rs`, `src/bin/tl-parse.rs`, `src/dialect/mod.rs`, `src/context.rs`, `src/lib.rs`) = 414 caught, 77 missed, 14 timeout, 43 unviable, 81.98%.

The crate floor is the sampled rate minus a margin, because the sample covers only the files named
above; the first full `make mutants` run replaces the sample and the floor should then be raised to the
measured value.

The gate is shown to fail: with a floor above the measured rate it prints `FAIL` and exits 1.
Unresolved survivors and timeouts of the measured populations (each is a `test_gap` to kill with a test,
or an equivalent mutant to list in `exclude_re` with a reason):

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

## Coverage

`make coverage-branch` on this tree (all features, `ci_guard` excluded):

| Metric | Value | Metric | Value | Metric | Value | Metric | Value |
| --- | --- | --- | --- | --- | --- | --- | --- |
| lines | 86.37% | regions | 83.77% | functions | 81.86% | branches | 86.22% |

The stable-toolchain `make coverage` reports slightly different totals than the nightly branch run (it counts test sources too), so `COVERAGE_MIN_LINES` is set a point or two below the lower of the two. It is a floor, not a target; it fails the target when line coverage drops below it.
