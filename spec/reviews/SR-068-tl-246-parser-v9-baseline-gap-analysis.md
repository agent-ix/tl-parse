---
id: SR-068
title: "TL-246 parser V9 baseline staging gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-parse@02e167e6728e0daf574dc637700d970a9dcb22e7; benches/parser_roundtrip.rs, benches/inputs/{SHA256SUMS,*.txt}, Cargo.toml, Cargo.lock; TL-246 PR #55 purpose and agent-ix/tl-mltl FR-051-AC-1, NFR-009-AC-1, TC-190, MP-062, MP-064, campaign/V9_CRITERION.md, campaign/v9_criterion.py, campaign/source-closure.json, campaign/test_v9_criterion.py, src/bin/tl_campaign_check/v9_replay.rs"
review_set: subset
---

## Summary

Ticket: TL-246. This is an acceptance-to-tests check of the bounded PR #55 purpose: publish a reproducible, equal-harness parser V9 baseline input from the feature-complete historical parser source. There is no tl-parse plan bundle for this staging-only PR, and the owning V9 Campaign requirement and plans live in tl-mltl. Evaluated PR head 02e167e6728e0daf574dc637700d970a9dcb22e7 against its historical parent 225a3300963199e523958f4abe4c16cc9dcca641 and current tl-parse benchmark blob 75e03a9e134ab69b85310f8be0c4b2cc591ae8c2.

## Verdict

**PASS for the bounded staging purpose.** The PR supplies an equal benchmark source and retained historical dependency graph, and the real benchmark target compiles and runs every named smoke case. It does not satisfy FR-051-AC-1 or TC-190's full paired-distribution acceptance by itself. The published tl-mltl Campaign source closure and definition still pin the prior parser baseline, and no full same-host pair or checker receipt is part of this PR. That known TL-246 integration gap remains open without changing this PR's bounded verdict.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No gaps found within the bounded PR #55 staging purpose (placeholder). | - |

## Coverage

| Bounded PR assertion | Evidence and test | Result |
| --- | --- | --- |
| Historical parser source remains exact | One-file diff from 225a330 to 02e167e; unchanged Cargo.toml and Cargo.lock digests; lock pins tl-syntax fed48a2 | PASS |
| Candidate and baseline harness bytes match | Reviewed head and current main benchmark blobs both 75e03a9e134ab69b85310f8be0c4b2cc591ae8c2; six benchmark inputs and SHA256SUMS unchanged from parent and match current main | PASS |
| Target and workload population are executable | `cargo bench --locked --offline --bench parser_roundtrip --no-run` on archived head; compiled Criterion `--test --noplot` executes ten parser_only and ten parser_roundtrip names, all pass | PASS |
| Timed V9 scope remains explicit | `v9_criterion.py` selects parser_roundtrip group and its ten cases; parser_only group is separately named; both loops preflight pinned inputs and use the same parse limits | PASS |
| Full V9 requirement, FR-051-AC-1 / NFR-009-AC-1 / TC-190 | Requires current source closure repinning, two paired same-host distributions, raw retention, and independent checker verdict. Published source closure still names 225a330 and TC-190 is planned in tl-mltl's matrix | OPEN in TL-246; outside PR #55 |

`quire coverage --scope . --json` for tl-parse completed: 124/128 backed matrix rows, with no unbacked rows or status lies; its remaining global symbols and suspicions are unrelated to this one-file benchmark staging. No changed production parser code, requirement, Test Matrix row, or test tag is introduced by PR #55. Direct Criterion smoke is the operative behavior check for the new benchmark file; existing tl-mltl V9 tests guard source and harness identity in the integration lane. Optional semantic review was not run because this targeted mechanical review did not receive a separate request for it.
