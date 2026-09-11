---
id: SR-032
title: "Code review — issue 27 provenance and hosted-workflow controls"
type: SpecReview
analysis: code-review
scope: "docs/ATTRIBUTION.md, docs/DIALECT-001-clean-room-mltl-v1.md, README.md, deny.toml, assurance/**, .github/workflows/ci.yml, tests/cli.rs, tests/shared_assurance.rs, spec/test-matrix.md"
review_set: subset
---

## Summary

This authorial review traced the issue 27 implementation against
NFR-002-AC-2, NFR-003-AC-5, TC-031, and TC-032. The candidate restores the
enumerated clean-room compiled-pin delta, describes the exact revision as a
reviewed-history commit rather than a moving head, and makes the hosted
workflow use exactly the released scoped ix-flow package under its sole manual
trigger. The controls are implemented in Rust and include executable mutation
probes rather than source-text assertions alone.

## Verdict

**AUTHORIAL / CONDITIONAL** — all grounded implementation findings are resolved
in the candidate, its 31 matrix test cases have backing trace symbols, and the
new behavior remains inside the two accepted criteria. This record grants no
independent exact-head pull-request clearance.

## Assurance Context

The reviewed baseline is `ad15ffec5434fbd77c69251eaf046994c65a47f5`.
SR-024 through SR-031 are the accepted specification review set. Source
inspection covered the exact tl-syntax range
`953ee825e5060335b4c79682f5f41a78c5a1bfae..26b801d4a68ebfe720062cfdb3c66b070ab60e92`.
The pinned local tools are Quire 0.31.0, Quoin 0.23.1, and
`@agent-ix/ix-flow@0.0.4`. Hosted CI remains `workflow_dispatch` only and was
not dispatched.

The gap-analysis procedure was not repeated for the whole active
`spec/plans/PLAN-002-shared-assurance-migration/` bundle: that broader plan and
its remaining work are already reviewed by SR-009 and SR-011, while issue 27 is
a bounded correction within it. Its applicable mechanical check was run
directly: `quire coverage --scope . --json` reports no unbacked test-case rows,
status lies, or untracked symbols. The four unbacked suite-registry rows remain
the deliberately non-source-symbol suites documented in
`spec/evidence/suites.md` and SR-007.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3201 | medium | Resolved: the attribution record again enumerates the exact source-inspected compiled-pin delta and distinguishes the new upstream API families tl-parse consumes from those it does not. | NFR-002-AC-2, TC-031, docs/ATTRIBUTION.md, tl-parse#26 FND-2601 |
| FND-3202 | medium | Resolved: provenance prose no longer calls the fixed compiled revision the current branch head; it identifies an exact commit reachable from reviewed main history when admitted. | NFR-002-AC-2, TC-031, README.md, deny.toml, docs/DIALECT-001-clean-room-mltl-v1.md, assurance/pins.json |
| FND-3203 | medium | Resolved: the manual-only hosted workflow installs exactly `@agent-ix/ix-flow@0.0.4`; TC-032 rejects unscoped, alias, unversioned duplicate, automatic-trigger, and wrong-runtime mutations while ignoring comment-only spellings. | NFR-003-AC-5, TC-032, .github/workflows/ci.yml, tests/shared_assurance.rs |
| FND-3204 | low | Carried forward: the active traceability declaration expects `Status`, while the validated TestMatrix archetype requires `Coverage Status`; changing the local header makes strict structural validation fail. The program-wide module mismatch remains tracked by quire-contract-ir#21 and is not safely repairable in this repository. | spec/test-matrix.md, TM-001, agent-ix/quire-contract-ir#21, SR-007 FND-703 |
| FND-3205 | medium | **FIXED after independent review of `362d997`:** whole-workflow token scanning missed executable alternate package specifications after a word-internal shell `#`, including GitHub npm specs. The scanner now isolates YAML run scalars before applying shell comment rules and covers npm `add` plus git/GitHub identity-bearing arguments. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3206 | low | **FIXED after independent review of `362d997`:** whole-workflow scanning counted inert `name:` metadata. TC-032 now proves metadata is outside the executable population and that quoted YAML `run` keys remain in scope. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3207 | medium | **FIXED after independent review of `cc24922`:** textual `run:` recognition omitted legal `run :` YAML and other decoded key spellings. TC-032 now parses YAML and selects only semantic `jobs.*.steps[*].run` string scalars. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3208 | medium | **FIXED after independent review of `cc24922`:** an empty quoted word before `#` was incorrectly treated as no word, reopening the executable hash bypass. The shell tokenizer now retains started empty words and TC-032 exercises that exact mutation. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3209 | low | **FIXED after independent review of `cc24922`:** the mutation fixture assumed the canonical command was spelled `npm install`. Controls now locate any documented install alias and independently prove the complete alias family, including `npm add`, remains admitted. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3210 | medium | **FIXED after independent review of `0093009`:** parenthesized commands and path-qualified npm executables could bypass the package census. Shell grouping is now a command boundary and executable matching uses the final path component, with a grouped `/usr/bin/npm in` control. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3211 | low | **FIXED after independent review of `0093009`:** npm- and shell-shaped text in inert arguments was counted as executable. The scanner now resolves one command position after assignments and `env`, and the control proves inert `printf` arguments do not change the package population. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3212 | medium | **SUPERSEDED by the closed grammar after independent review of #30:** leading shell redirection cannot occupy a partially interpreted command position because every unquoted redirection-bearing run script is rejected. TC-032 covers attached, separate, fd-duplication, and chained forms. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review, tl-parse#30 review |
| FND-3213 | medium | **FIXED after independent review of `085783e`:** a `sh`/`bash` command without `-c` returned from the whole run-scalar scan and suppressed later commands. It now ends only that command's nested-shell inspection, and TC-032 retains the later-alternate mutation. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#28 review |
| FND-3214 | high | **FIXED by closing the admitted grammar:** the scanner no longer attempts to tokenize `2>&1` or any other unquoted redirection. It rejects the full run script with its observed identities, and TC-032 includes the reviewer-discovered `2>&1> /dev/null` chain. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-rewrite#34 review, tl-parse#30 review |
| FND-3215 | high | **FIXED after independent review of `afda8c1`:** double shell quotes do not suppress command-substitution grammar, so executable npm and redirection inside `$(...)` were treated as inert outer arguments. The closed grammar now rejects every unescaped `$` or backtick outside single quotes, while single-quoted and escaped spellings remain inert controls. | NFR-003-AC-5, TC-032, tests/shared_assurance.rs, tl-parse#30 review |

## Review Evidence

- Quire validation: 68/68 documents grammar-clean before this review artifact,
  with zero grammar findings.
- Quire coverage: 71/75 declared rows backed; all 31/31 test-case rows and all
  35/35 requirement criteria backed, with zero status lies and zero untracked
  symbols. The remaining 4 rows are the declared non-source-symbol suites.
- TC-020 preserved the normative dialect digest and matched the refreshed
  whole-document dialect and attribution digests.
- TC-031 passed against the exact enumerated range and consumed/unconsumed API
  boundary.
- TC-032 passed under the pinned ix-flow 0.0.4 executable; the ambient 0.2.3
  executable was rejected as intended.
- Corrective TC-032 probes preserve word-internal shell hashes, ignore actual
  shell comments and step metadata, recognize quoted YAML `run` keys, and make
  an npm-add GitHub alternate specification turn the gate red.
