---
id: SR-004
title: tl-parse merged PGM-01 reconciliation
type: SpecReview
analysis: base
scope: PGM-01 requirements and the tl-parse v0.1 candidate
review_set: all
---

# tl-parse merged PGM-01 reconciliation

## Summary

Merged PGM-01 policy: `agent-ix/quire-contract-ir#12` at
`7dac9d8c19952412b56a0347387666e2ca81e01d`.

Envelope schema: `quire.derivation-evidence/v1`, SHA-256
`0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256`.

The collector architecture is adapted under MIT OR Apache-2.0 from the
same-program tl-rewrite collector at immutable revision
`b9cd1764ef70d7508603049e10e56ce5ceae40e2`. The adaptation records exact
source, dependency, dialect, corpus, limits, complete command outcomes,
canonical validation, overwrite refusal, external checksums, and a separate
post-seal summary for finalized-envelope validation. Human review remains
pending.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-401 | medium | PGM-01 is merged and exactly reconciled; the retained exact-candidate record passes all 13 collection/post-seal checks, while the independent human release decision remains open. | PGM-01, AP-001, AA-001 |

## Policy mapping

| Policy requirement | tl-parse disposition | Evidence or remaining gate |
|---|---|---|
| PGM-01-R01 schema compatibility | Dialect, diagnostic, corpus, local evidence input, and manifest records use explicit closed v1 identities and reject unknown schema fields. | FR-001, FR-003, FR-005; serde and evidence schemas |
| PGM-01-R02 exact pins | Candidate evidence records source, merged policy/schema, toolchain, syntax dependency, dialect/corpus records, parameters, inputs, and outputs. | Collection input, manifest, canonical envelope |
| PGM-01-R03 release order | tl-parse pins the exact reviewed tl-syntax candidate; downstream repositories must pin the eventual human-reviewed tl-parse revision. | Cargo.toml, Cargo.lock; human decision remains open |
| PGM-01-R04 licensing and provenance | Crate, dialect, schemas, collector adaptation, corpora, and seeds are MIT OR Apache-2.0 with explicit provenance. | Cargo.toml, dialect/corpus/evidence documentation |
| PGM-01-R05 clean-room boundary | The dialect is independently authored from the allowed tl-syntax vocabulary and no third-party grammar text is copied. | DIALECT-001, MRS-001, repository inspection |
| PGM-01-R06 human authority | Agent-produced implementation/evidence remain separate from independent human review and release. | AP-001, AA-001, envelope provenance |
| PGM-01-R07 classification | tl-parse is a text-boundary component and requires consuming-project verification. | AP-001, CAC-001, README.md |
| PGM-01-R08 common envelope | Revision-scoped evidence emits every canonical field and is gated by the exact PGM-01 Draft 7 schema and validator. | Collector and retained validator outputs |
| PGM-01-R09 retention and decision | New runs refuse overwrite, retain stdout/stderr/status/digests/limitations, and record no automated release decision. | Collector, external checksum file, AA-001 |
| PGM-01-R10 qualification boundary | Parser, formatter, generated, fuzz, and corpus evidence confer no semantic proof, consuming-project validation, accreditation, or certification. | AP-001, AA-001, README.md |

The merged policy requires no additional public parser API. Independent
review, protected-branch checks, and the human source-release decision remain
external workflow gates.
