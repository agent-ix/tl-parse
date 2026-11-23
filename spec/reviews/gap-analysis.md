---
id: SR-1501
title: "gap-analysis review of TL-150 numeric specs (TL-150-spec-tl-parse)"
type: SpecReview
analysis: gap-analysis
scope: "TL-421, TL-424 (parse half), TL-425 (parse half)"
review_set: all
---

## Summary

TL-421 and parse halves of TL-424/TL-425 map fully: TL-421 AC1-3 to FR-031/028/033 and TC-1549..1557; TL-424 to FR-032 TC-1552/1553; TL-425 to FR-032-AC-3 TC-1554. No unmapped ACs. Matrix rows are all planned; StR-014-VC rows are demonstrations only.

Method note: independent fresh-context review, no build tools run. Skill lenses were applied manually by the reviewer in one pass; the criterion-strength lens normally uses the external Jev service, which was not used. All specs are unbuilt and ADRs are proposed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unmapped acceptance criteria in tl-parse scope. | TM-014 |
| FND-002 | low | No AC covers a contract plus aggregation mix or temporal template inside a quantifier through the formatter (aggregation is not recovered from a graph, so round-trip of group requirements is only via expansion). | FR-031; FR-032 |
