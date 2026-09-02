---
id: NFR-002
title: Preserve clean-room provenance and qualification boundaries
type: NFR
quality_attribute: compliance
---

# NFR-002: Preserve clean-room provenance and qualification boundaries

## Statement

The source shall remain independently authored from the permitted tl-syntax
operator model and repository requirements, with exact revision pins and no
claim that automated checks replace human review.

## Scope

The requirement covers grammar authorship, dependency/dialect/corpus pins,
qualification language, and release authority.

## Rationale

Textual compatibility and assurance claims are not reviewable if their source,
version, license, or decision owner can drift silently.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Unpinned governed inputs | 0 | 0 | Inspection |
| Automated release approvals | 0 | 0 | Inspection |

## Verification

Dialect, wire, and corpus tests inspect exact identities. No automated check
grants review or release authority; that remains a human's and is established by
inspection rather than by a gate.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | The dialect record names its authorship basis, license boundary, exact compiled tl-syntax pin, stable revision, and digest, with the authorship basis and the compiled revision recorded as separate facts. The compiled pin is enforced by `Cargo.toml` and `Cargo.lock` rather than by a repository-local digest table. | Inspection (TC-020) |

## Qualification Boundary

Passing evidence supports review of one candidate. It does not qualify a
consumer, certify semantic correctness, or authorize publication.
