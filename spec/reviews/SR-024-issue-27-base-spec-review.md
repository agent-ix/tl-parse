---
id: SR-024
title: Base review of issue 27 provenance and hosted-workflow controls
type: SpecReview
analysis: base
scope: "NFR-002, NFR-003, TM-001 additions for tl-parse issue 27"
review_set: all
---

## Summary

The two new criteria are concrete, independently traceable to sequential planned
tests, and preserve the human provenance and release boundaries. The planned
rows are intentionally unbacked until implementation begins after acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2701 | low | IDs remain collision-free, and criteria and test rows cross-reference exactly. | NFR-002-AC-2, NFR-003-AC-5, TC-031, TC-032 |
| FND-2702 | medium | **FIXED after exact-head review of `362d997`:** NFR-003-AC-5 did not allocate YAML run selection, shell comment boundaries, npm `add`, or identity-bearing alternate specs precisely enough. The criterion and TC-032 now name those outcomes and the implementation supplies discriminating controls. | NFR-003-AC-5, TC-032, tl-parse#28 review |
| FND-2703 | medium | **FIXED after exact-head review of `cc24922`:** source-spelling recognition still omitted legal YAML separator spacing, empty quoted shell words reopened the hash bypass, and TC-032 depended on the canonical command using `npm install`. The criterion now allocates semantic job-step selection, the full documented npm-install alias family, literal nested shell commands, and a started-word comment boundary. | NFR-003-AC-5, TC-032, tl-parse#28 review |
