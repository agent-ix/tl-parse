---
id: NFR-002
title: Preserve clean-room provenance boundary
type: NFR
quality_attribute: compliance
---

# NFR-002: Preserve clean-room provenance boundary

## Statement

The source shall remain independently authored from the permitted tl-syntax
operator model and repository requirements, with no claim that automated checks
replace human review.

## Scope

The requirement covers grammar authorship, the license boundary, qualification
language, and release authority.

## Rationale

Compatibility and assurance claims are not reviewable if their source, license,
or decision owner can drift silently.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Automated release approvals | 0 | 0 | Inspection |

## Verification

The dialect record and attribution record are checked for their authorship and
license statements. No automated check grants review or release authority; that
remains a human's and is established by inspection rather than by a gate.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | The dialect record names its authorship basis and license boundary. | Test (TC-020) |

## Qualification Boundary

Passing evidence supports review of one candidate. It does not qualify a
consumer, certify semantic correctness, or authorize publication.
