---
id: NFR-003
title: Make qualification controls explicit and fail closed
type: NFR
quality_attribute: reliability
---

# NFR-003: Make qualification controls explicit and fail closed

## Statement

Candidate qualification shall bind executable identities, execute a complete
local-gate census, retain positive outcome evidence, and distinguish active,
inconclusive, failed, and explicitly retracted records without granting release
authority.

## Scope

This requirement owns `tools.lock`, the local-CI runner and propagation probes,
the collection/finalization/verifier scripts, the evidence retraction registry,
and their traceability and behavior tests.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Mandatory tools with selected-profile, source-locked SHA-256 identities | 9/9 | 9/9 | Test |
| Candidate local-CI gates with required positive signatures | 14/14 | 14/14 | Test |
| Active records lacking qualification-v2 | 0 | 0 | Test |
| Automatic release decisions | 0 | 0 | Inspection |

## Verification

Behavior tests exercise the tool lock, exact-path resolution, compiled Rust-test
census, gate census, transcript postcondition, profile/retraction/unsupported
states, clean-tree refusal, and retained-record checks. Git-ignored `target`,
`__pycache__`, and `*.pyc` paths are build/runtime caches rather than candidate
source and are outside the clean-tree source census.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every mandatory executable is source-locked by selected, versioned environment profile, absolute path, and SHA-256; Cargo and rustc identify the actual stable binaries while separately pinned rustup selects explicit auxiliary toolchains. Collection verifies the exact resolved path and live bytes, retains the selected profile, and re-derives records against that profile in the source revision's lock. Declared byte-identical aliases are accepted, while undeclared profiles, paths resolving elsewhere, and different bytes fail closed. | Test (TC-023) |
| NFR-003-AC-2 | Local CI independently verifies the clean-tree retained-evidence boundary before delegation, refuses ambient profile selection plus global, multi-target, pattern-scoped, imported, and dynamic Make execution controls, enumerates every mandatory gate, compares the requirement-tagged Rust tests with Cargo's compiled census, propagates every command failure, and refuses success unless its transcript names the selected profile and contains the exact positive gate census. | Test (TC-024) |
| NFR-003-AC-3 | Active evidence requires qualification-v2 and positive output for every non-silent retained lane; missing mandatory lanes and legacy qualification states are non-passing; an unsupported source tool-lock schema has a distinct individual verification state and exit while checksum-verified unsupported archives remain counted history without requiring retraction; explicitly retracted records remain checksum-verifiable but cannot pass qualification; and the assurance-bound record must be active v2 and cannot be retracted. | Test (TC-025) |
| NFR-003-AC-4 | Evidence verification requires a clean tree and checks record identity and append-only behavior relative to the presented Git history; no local digest claims external attestation or release authority. | Inspection (TC-022) |

## Qualification Boundary

These controls make a presented candidate and its retained artifacts
reproducible and reviewable. Branch protection and the remote review history,
not the local repository, establish resistance to history replacement.
