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
| Mandatory tools with source-locked SHA-256 identities | 8/8 | 8/8 | Test |
| Candidate local-CI gates with required positive signatures | 13/13 | 13/13 | Test |
| Active records lacking qualification-v2 | 0 | 0 | Test |
| Automatic release decisions | 0 | 0 | Inspection |

## Verification

Behavior tests exercise the tool lock, gate census, transcript postcondition,
profile/retraction states, clean-tree refusal, and retained-record checks.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every mandatory executable is source-locked by absolute path and SHA-256; collection verifies the live bytes and retained records are re-derived against the source revision's lock. | Test (TC-023) |
| NFR-003-AC-2 | Local CI independently verifies the clean-tree retained-evidence boundary before delegation, enumerates every mandatory gate, propagates every command failure, and refuses success unless its own transcript contains the exact positive gate census. | Test (TC-024) |
| NFR-003-AC-3 | Active evidence requires qualification-v2 and positive output for every non-silent retained lane; missing mandatory lanes are inconclusive; older records are explicitly retracted or inconclusive; and the assurance-bound record cannot be retracted. | Test (TC-025) |
| NFR-003-AC-4 | Evidence verification requires a clean tree and checks record identity and append-only behavior relative to the presented Git history; no local digest claims external attestation or release authority. | Inspection (TC-022) |

## Qualification Boundary

These controls make a presented candidate and its retained artifacts
reproducible and reviewable. Branch protection and the remote review history,
not the local repository, establish resistance to history replacement.
