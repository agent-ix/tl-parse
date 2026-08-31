---
id: SR-002
title: Code review of tl-parse v0.1 implementation
type: SpecReview
analysis: code-review
scope: src/**/*.rs, tests/**/*.rs, corpus/**/*, fuzz/**/*, schemas/**/*.json, scripts/**/*, Cargo.toml, Makefile, .github/workflows/ci.yml
review_set: all
---

# Code review of tl-parse v0.1 implementation

## Summary

The code review examined exact dependency/profile identities, lexical token
boundaries, UTF-8 byte spans, canonical-number handling, precedence and
associativity, direct topological graph construction, fail-closed recovery,
all parser/formatter limits, iterative graph traversal, diagnostic and JSON
stability, CLI exit classes, corpus/fuzz integrity, evidence classification,
and CI triggers. Follow-up review additionally exercised deep formatter fixed
points, actual cargo-fuzz compilation/execution, JSON deserialization, the
source-size/UTF-8 boundary, CLI misuse paths, behavioral Make targets, evidence
artifact tampering, and clean-room provenance bindings. Review corrected
canonical adjacency tokenization, exact amended tl-syntax pins, limit-test
construction, corpus integrity gates, formatter depth growth, fuzz-target
dependency drift, evidence false-success handling, streamed source-size
reporting, per-node formatter budgeting, failure propagation, strict
traceability completeness, and evidence-anchor cross-checking. No unresolved code
defect or blocking requirement was found. Independent human review remains
mandatory.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-201 | high | Resolved: all package, lockfile, API, dialect, assurance, and evidence identities now pin amended tl-syntax revision `740182f13b84858008d6f176f75136737d405c1b`. | FR-002, NFR-002 |
| FND-202 | medium | Resolved: atoms followed immediately by canonical `U[...]`/`R[...]` operators tokenize consistently with whitespace-free formatter output. | FR-001, FR-004 |
| FND-203 | medium | Resolved: source, token, node, depth, diagnostic, parse-work, output, and format-work failures suppress partial success and have direct tests. | FR-003, FR-004 |
| FND-204 | medium | Resolved: retained corpus and fuzz seeds are checksum-gated and successful seeds must parse-format-parse without drift. | FR-005 |
| FND-205 | medium | Resolved: evidence tooling distinguishes failure, skipped validation, provisional/final non-self-attestation, and sealed validation; immutable checksums are in the complete local gate. | FR-005, NFR-002 |
| FND-206 | medium | Resolved: hosted CI retains only `workflow_dispatch`; local iteration uses `make ci`, and a machine-checkable regression rejects push/PR triggers. | FR-005, MP-001 |
| FND-207 | high | The temporary exact git-source exception expires on 2026-09-07 or upstream merge and blocks source release until repin and evidence regeneration. | AA-001 |
| FND-208 | high | Resolved: precedence-aware minimal-parentheses rendering makes deep prefix and same-associativity chains structural fixed points within the declared parser depth budget. | FR-004 |
| FND-209 | high | Resolved: the actual cargo-fuzz target imports the public re-export, has an exact lockfile, compiles under nightly, and executes all retained seeds in a bounded local smoke gate. | FR-005, TC-019 |
| FND-210 | medium | Resolved: wire types deserialize with closed fields; diagnostic and format failures implement standard display/error contracts; CLI tests cover JSON format, diagnostic truncation, ambiguous paths, unknown options, and the byte-limit/UTF-8 boundary. | FR-003, FR-005 |
| FND-211 | medium | Resolved: retained evidence re-derivation, per-artifact manifest verification, behavioral Make probes, whole-document dialect/provenance digests, and exact consulted-file hashes reject previously viable false-positive mutations. | FR-005, NFR-002 |
| FND-212 | low | Defensive graph-validation errors remain intentionally typed at internal invariant boundaries even though safe public constructors prevent callers from manufacturing those states. | FR-003, FR-004 |
| FND-213 | medium | The dialect deliberately associates unparenthesized `U`/`R` chains to the left; the normative record now calls out the interoperability risk and requires explicit parentheses across unlike dialects. | FR-001, NFR-002 |
| FND-214 | medium | Resolved: oversized seekable files use metadata for the exact count, while streams stop after the first byte beyond the limit; neither path parses fabricated bytes or waits for an unbounded producer to close. | FR-005, NFR-001, TC-021 |
| FND-215 | medium | Resolved after follow-up: checked recipes use fixed tool names; local CI refuses ambient Make controls, tool overrides, optimized Python, and sanitizer overrides; static, expanded-recipe, and per-command probes reject false-success controls at every recipe position. Qualification-v2 collection applies a clean allowlisted environment to every retained command and requires positive Rust, corpus, policy, traceability, attribution, and sanitizer output. Direct and conditional ignored Rust tests, trace references/statuses/diagnostics, evidence membership/history/symlinks/records, and required JSON formats are behavior-tested. | FR-004, FR-005, NFR-001, NFR-002, TC-017, TC-022 |
| FND-216 | high | Resolved: canonical formatting uses one iterative output buffer and charges expanded nodes plus emitted bytes, removing quadratic retained strings while preserving the accepted node boundary. The public source-limit report is total at below/equal/above-limit inputs. | FR-004, FR-005, NFR-001, TC-017, TC-021 |
| FND-217 | high | Resolved: exact-candidate evidence must positively prove at least 25 passing Rust tests across eight result groups plus the complete local-gate transcript census; the assurance-bound record must have every outcome passed and cover the current non-evidence tree. Parameter digests, tool identities, and envelope results are re-derived from the named source revision. Record-history checks are explicitly relative to the presented Git history; remote branch protection and review remain the non-local control. | FR-005, NFR-002, NFR-003, TC-022, TC-023, TC-024, TC-025 |
| FND-218 | high | Resolved during final QA: `make ci` now performs the independent clean-tree retained-evidence verification before delegating to the transcript runner, so an uncommitted early return in the runner cannot become a single-file success key. | NFR-003, TC-024 |
| FND-219 | medium | Resolved during final QA: the collector obtains locked tool fields through the `tool_identity.py` CLI, which is executable under the exact `env -i` collection environment; all eight path and digest lookups have a clean-environment behavior test. | NFR-003, TC-023 |
| FND-220 | medium | Resolved during final QA: the source-locked qualification environment carries `CARGO_TARGET_DIR`, and failed collection removes only its exact `mktemp` staging root, preventing cargo-fuzz output from dirtying the candidate tree. | NFR-003, TC-023, TC-025 |
| FND-221 | medium | Resolved during final QA: the standalone retained MSRV lane invokes `make msrv` and must contain `msrv gate passed`; exit code zero without that signature is classified as failed. | NFR-003, TC-024, TC-025 |
