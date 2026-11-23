---
id: FR-032
title: Parse set aggregation and contract declarations
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-014
    type: implements
  - target: ix://agent-ix/tl-parse/FR-029
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-306
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-307
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-309
    type: depends_on
---

# FR-032: Parse set aggregation and contract declarations

## Description
When v5 text declares member sets, quantified templates or contracts, tl-parse shall build the tl-syntax aggregation or contract-set document and return the graph expanded through tl-syntax FR-307, or refuse with typed diagnostics.

## Behavior

```
document  = { setdecl } ( formula | contract { contract } )
setdecl   = "set" identifier "=" "{" [ identifier { "," identifier } ] "}" ";"
quantified= "(" ( "each" | "some" | "atleast" integer ) identifier "in" identifier ":" formula ")"
contract  = "contract" identifier ":" "assume" formula ";" "guarantee" formula ";"
```

- Set members are catalog signal names; an empty member list is explicit and
  expands by tl-syntax FR-307's empty-set rule. Set names share the identifier
  namespace's reserved-word rule; they are not catalog names and are resolved
  only after `in`.
- The bound variable is visible only inside the quantified template, as an
  expression leaf (`x > 500`) or as a Boolean leaf (`x`). Elsewhere it is
  `slot_outside_template`. Nested quantifiers over distinct variables are
  admitted to the depth of tl-syntax FR-306 (2); a third level is refused as
  `depth_limit`. The bound is the tl-syntax FR-307 bound computed over the whole
  composition (nodes, predicates and expression nodes, including the per-member
  expression copies), never per level; a variable that shadows an enclosing one is
  refused as `unexpected_token` at the shadowing binder. Fresh proposition identities
  follow the precedence of tl-syntax FR-307 and FR-029 above.
- `aggregation_too_large` is reported before any node is built, with the exact
  counts from tl-syntax FR-307.
- Contracts: names unique (`duplicate_contract`), both clauses required
  (`missing_contract_clause`). A source with contracts has as graph root the
  left-balanced conjunction of the contracts' `assume -> guarantee` obligations
  (one contract: that obligation); the contract-set document, not the root,
  carries the structure.
- Contract text and an unbound plain formula cannot be mixed in one document.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-032-AC-1 | Each quantifier over Boolean and numeric member sets parses to the same graph as the hand-written expansion in the corpus, and the report records the exact node and predicate counts. | integration-testing (TC-1552) |
| FR-032-AC-2 | An expansion at the tl-syntax bound plus one (including a nested pair whose product exceeds a bound while each level alone does not, and a template whose per-member expression copies alone exceed it) is refused with `aggregation_too_large` and counts, with no graph; a third nesting level is refused with `depth_limit`. | unit-testing (TC-1553) |
| FR-032-AC-3 | Contract declarations parse to a contract-set document with the declared names, ids in source order, and shared roots; duplicate name and missing clause are refused. | unit-testing (TC-1554) |

## Dependencies

tl-syntax FR-306, FR-307, FR-309.
