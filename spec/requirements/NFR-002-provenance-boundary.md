---
id: NFR-002
title: Preserve clean-room provenance and qualification boundaries
type: NFR
quality_attribute: compliance
---

# NFR-002: Preserve clean-room provenance and qualification boundaries

## Statement

The source shall remain independently authored from the permitted tl-syntax
operator model and repository requirements, with exact revision pins, an
enumerated source-inspected delta whenever the compiled pin advances, and no
claim that automated checks replace human review.

## Scope

The requirement covers grammar authorship, dependency/dialect/corpus pins,
the distinction between upstream API changes this crate consumes and does not
consume, qualification language, and release authority.

## Rationale

Textual compatibility and assurance claims are not reviewable if their source,
version, license, or decision owner can drift silently.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Unpinned governed inputs | 0 | 0 | Inspection |
| Compiled-pin advances without an enumerated source-inspected delta | 0 | 0 | Test |
| Automated release approvals | 0 | 0 | Inspection |

## Verification

Dialect, wire, and corpus tests inspect exact identities. A provenance test
requires the attribution record to identify the exact old-to-new compiled-pin
range, enumerate the changed public API families found by source inspection,
and say which of those families tl-parse directly consumes. No automated check
grants review or release authority; that remains a human's and is established by
inspection rather than by a gate.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | The dialect record names its authorship basis, license boundary, exact compiled tl-syntax pin, stable revision, and digest, with the authorship basis and the compiled revision recorded as separate facts. The compiled pin is enforced by `Cargo.toml` and `Cargo.lock` rather than by a repository-local digest table. | Test (TC-020) |
| NFR-002-AC-2 | The attribution record preserves the historical authorship basis and enumerates the source-inspected `953ee825e5060335b4c79682f5f41a78c5a1bfae..26b801d4a68ebfe720062cfdb3c66b070ab60e92` compiled-pin delta: caller-context documents, signal catalogs and bindings, span-free semantic formula identity, and assurance-only changes. It states that tl-parse directly consumes none of those new API families, continues to consume the pre-existing graph, interval, span, proposition, and semantic-profile contracts, and consulted no later grammar source. | Test (TC-031) |

## Qualification Boundary

Passing evidence supports review of one candidate. It does not qualify a
consumer, certify semantic correctness, or authorize publication.
