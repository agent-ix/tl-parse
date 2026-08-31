---
id: NFR-002
title: Preserve clean-room provenance and qualification boundaries
type: NFR
quality_attribute: compliance
---

# NFR-002: Preserve clean-room provenance and qualification boundaries

## Statement

The source shall remain independently authored from the permitted tl-syntax
operator model and repository requirements, with exact revision and evidence
pins and no claim that automated checks replace human review.

## Scope

The requirement covers grammar authorship, dependency/dialect/corpus pins,
candidate evidence, qualification language, and release authority.

## Rationale

Textual compatibility and assurance claims are not reviewable if their source,
version, license, or decision owner can drift silently.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Unpinned governed inputs | 0 | 0 | inspection and evidence validation |
| Automated release approvals | 0 | 0 | assurance-boundary test |

## Verification

Dialect, wire, corpus, and evidence tests inspect exact identities and require
human review/release state to remain pending.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | The dialect record names its authorship basis, license boundary, exact tl-syntax pin, stable revision, and digest. | Inspection (TC-020) |
| NFR-002-AC-2 | Retained evidence identifies exact source, dependency, corpus, schemas, limits, and toolchain while leaving review/release authority pending. | Test (TC-022) |

## Qualification Boundary

Passing evidence supports review of one candidate. It does not qualify a
consumer, certify semantic correctness, or authorize publication.
