---
id: SR-610
title: Shared CLI encoding and writer Rust review
type: SpecReview
relationships:
  - target: "ix://agent-ix/tl-parse/FR-005"
    type: reviews
---
# SR-610: Shared CLI encoding and writer Rust review

## Summary

Reviewed the shared generic utility adoption against the Rust review checklist and repository conventions. The local output boundary now owns only domain error classification; newline writing and compact format-report encoding delegate directly to ix-cli-kit.

## Scope

Revision 2a4f0dec01dc58865693a354dd6a1d1072bc9c39. Files examined: Cargo.toml, Cargo.lock, fuzz/Cargo.lock, deny.toml, src/bin/tl-parse.rs, tests/cli.rs, spec/requirements/FR-005-corpus-cli-evidence.md and reviews/2026-10-02-cli-kit-output-spec-review.md. The workflow diff is empty.

## Verdict

The initial runtime diff has no findings; fix FND-002 before delivery. Input/resource policy, domain report serialization, diagnostic rendering and exit classes are preserved. The dependency allowance names the exact shared package; no generic license allowance, credential operation, source copy, fallback reader or compatibility shim is introduced. Existing real-process closed-pipe coverage and injected writer failures assert the downstream policy; framing assertions observe actual emitted bytes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The finalized hosted gates cannot fetch private ix-cli-kit without repository read authentication, so all three build/license jobs fail before compilation. Configure the existing organization read credential for this dependency only and preserve every gate. | .github/workflows/ci.yml:12 at 6ee8c3e14b6c85aa0b26945ab595e0d122cc1c01; hosted run 37073809423 |
| FND-001 | low | No findings (placeholder). | FR-005-AC-4 |

## Validation

Complete make ci passed: formatting, clippy, tests, conformance, round trips, compiled test census, both fuzz-target builds and sanitizer smoke runs, dependency checks, unsafe audit, execution-control guard tests, strict specification/coverage checks, MSRV and rustdoc. The computed matrix tags FR-005-AC-4 to the injected writer and real CLI tests. The full gate runs again on the final head before merge. Quoin workflow tl-parse-cli-kit-output records the validated review; human acceptance is not acknowledged.

## Finalization scope

The hosted dependency-resolution failure was observed on 6ee8c3e14b6c85aa0b26945ab595e0d122cc1c01 after both complete local gates passed. Examined the entire existing manual-only CI workflow and the scoped authentication amendment. The existing organization REGISTRY_TOKEN is available to this repository; only its name was inspected. The amendment scopes its Git HTTP header to ix-cli-kit, masks the derived authorization header and selects the Git CLI. All three jobs and every check are retained. No credential is created, copied into source or changed.

## Dispositions

- FND-002: fixed d3d108e81364f9d4e4ecb391a57aac869e09abae. All three jobs configure masked read authorization for the ix-cli-kit URL through the existing organization credential and use Git CLI fetch. No job or check is removed. Fresh complete local gates and finalized hosted validation are required before merge.
