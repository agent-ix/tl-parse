---
id: SR-1500
title: "base review of TL-150 numeric specs (TL-150-spec-tl-parse)"
type: SpecReview
analysis: base
scope: "tl-parse FR-028..033, NFR-016, StR-014, TM-014, PLAN-015, dialect clean-ascii/v5"
review_set: all
---

## Summary

Base checklist: quoin validate no findings; 19 new ACs all mapped; matrix delta +19 untagged rows, +7 requirement-without-design (no ADR records the v5 dialect decision).

Method note: independent fresh-context review, no build tools run. Skill lenses were applied manually by the reviewer in one pass; the criterion-strength lens normally uses the external Jev service, which was not used. All specs are unbuilt and ADRs are proposed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No ADR for dialect v5 although it fixes reserved words, literal absorption of minus, proposition allocation and root conjunction for contracts; FR-028..033 and NFR-016 show as requirement-without-design. | FR-028..033; NFR-016 |
| FND-002 | medium | Lexing ambiguities: "-" absorbed into literal (so "x -5"), "(x)" formula versus "(arith cmp arith)" lookahead; reserved words invalidate existing catalog names. | FR-029 |
| FND-003 | medium | Nested quantifiers admitted with bound "product of member counts within tl-syntax bound", but tl-syntax FR-306 has one quantifier per document; allocation of fresh ids (tl-syntax first_proposition) vs FR-029 "1+max" allocation has no precedence rule. | FR-032; tl-syntax FR-306/FR-307 |
| FND-004 | low | FR-028 wording "the two future profiles above" garbled; v5 excludes past, unbounded and fairness while tl-mltl FR-055 and tl-solve ADR-200 handle origin-complete past, so past numeric requirements have no text form. | FR-028:43 |
| FND-005 | low | Hash/digest use: dialect digest record, catalog digest in report (checklist antipattern; follows v4 precedent). | FR-028; FR-029 |
