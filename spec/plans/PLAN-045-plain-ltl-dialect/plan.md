---
id: PLAN-045
title: Plain LTL surface dialect v5 plan
type: Plan
status: not_started
relationships:
  - target: ix://agent-ix/tl-parse/FR-058
    type: references
  - target: ix://agent-ix/tl-parse/FR-059
    type: references
  - target: ix://agent-ix/tl-parse/FR-060
    type: references
  - target: ix://agent-ix/tl-parse/FR-061
    type: references
  - target: ix://agent-ix/tl-parse/FR-062
    type: references
  - target: ix://agent-ix/tl-parse/NFR-046
    type: references
---

# PLAN-045: Plain LTL surface dialect v5 plan

## Objective

Land TL-429: the bracket-free dialect `tl-parse.clean-ascii/v5` for the LTLf and
infinite-trace profiles, selected explicitly by the caller, with round-trip,
hostile-input and fuzz coverage, and v1 to v4 unchanged. Starts after ADR-041,
tl-syntax ADR-044 and ADR-045 are accepted and tl-syntax TL-428 has landed.

## Dependency DAG

Layer numbers match `ix-board project-chunk P-TL-151`.

```text
layer 1  TL-428 tl-syntax profile and corpus (external to this repo)
layer 2  TL-429
           Task-001 v5 grammar and explicit profile (FR-058, FR-059)
             -> Task-002 formatter and round trip (FR-060)
             -> Task-003 hostile corpus, fuzz, limits (FR-061, NFR-046)
           Task-004 v1 to v4 unchanged proof (FR-062)  (parallel; gates merge)
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | FR-058, FR-059 | TC-112 to TC-117 |
| Task-002 | FR-060 | TC-118, TC-119 |
| Task-003 | FR-061, NFR-046 | TC-120 to TC-122, TC-125, TC-126 |
| Task-004 | FR-062 | TC-123, TC-124 |

## Quality gates

- Gate A after Task-001: TC-123 digests of v1 to v4 identical before any further
  work. If it fails, stop.

## Deferred verification

| Item | Why deferred | Needed to run |
|---|---|---|
| Fuzz campaign beyond seed replay (TC-121) | no fuzz or build capacity during specification | `make fuzz-build fuzz-smoke`, then a budgeted campaign |
| Miri run (NFR-046) | no build capacity | the existing Miri target |
| `make ci` | no build capacity | the repository gate |
| Dialect renumbered to `tl-parse.clean-ascii/v6` | TL program decision 2026-11-23 item 2 (numeric P-TL-150 keeps v5) | ADR-041, FR-058..FR-062, TM-043 and DIALECT document name updated from v5 to v6 |
| No v6 dialect record or digest | ADR-041 asks for one; decision 7 forbids new digest/pin registrations | Human confirmation that v6 stays digest-free like numeric v5 |
| TL-429 UNBUILT: no `cargo`/`rustc`/`make` was run (branch `TL-code-unbuilt`) | Program rule for unbuilt code | `make ci` at the TL-429 head after the tl-syntax pin below lands |
| UNBUILT-PIN tl-syntax rev | `Cargo.toml` pins tl-syntax origin/main `a80d09e`, which lacks the FR-332/FR-333 LTLf edition (TL-428) | Replace with the merged tl-syntax rev of TL-428; then fix `src/ltl/model.rs` only |
| Guessed tl-syntax LTLf API (all in `src/ltl/model.rs`) | Edition written concurrently, not merged | Confirm `LtlfFormulaDocument::{new(SemanticProfile, NodeId, Vec<LtlfNode>), root(), nodes(), semantic_profile()}` with serde; `LtlfNode { kind, span }` with `new`/`with_span`; `LtlfNodeKind` variants `False True Proposition Not And Or Implies Equivalent Future Globally Until Release StrongNext WeakNext` with interval-free fields; `SemanticProfile::LtlfFiniteTraceV1` |
| TC-112 FR-058-AC-1 | unbuilt; population read as 4 leaves {p0, p1, true, false} + unary over leaves + binary over leaf pairs (120 = 4 + 5x4 + 6x16, 116 = 4 + 4x4 + 6x16) | `the_depth_one_population_parses_under_its_profile` |
| TC-113 FR-058-AC-2 | unbuilt; all of `W M O H Y S T` share `unsupported_operator` (one code per refusal class, read of "its own code") | `intervals_and_out_of_dialect_operators_have_their_own_codes` |
| TC-114 FR-059-AC-1, StR-044-VC-1 | unbuilt | `one_text_two_profiles_two_graphs` |
| TC-115, TC-116 FR-059-AC-2 | unbuilt | two unit tests |
| TC-117 FR-059-AC-3 | unbuilt; the no-`Default`/no-`From<&str>`/no-`FromStr` check uses the inherent-vs-trait associated-const probe | `no_v6_entry_point_lacks_a_profile_and_the_profile_has_no_default` |
| TC-118 FR-060-AC-1 | unbuilt; graph equality is asserted on canonical text (spans differ between non-canonical source and its canonical text) | `every_population_and_precedence_formula_round_trips_to_a_fixed_point` |
| TC-119 FR-060-AC-2 | unbuilt | `graphs_outside_the_image_are_not_expressible` |
| TC-120, TC-122 FR-061-AC-1 | unbuilt; FR-061 says "3 parse, 26 refused under each profile", but bare `X` parses under the infinite profile and `Xs`/`Xw` under LTLf, so the manifest records 5 (LTLf) and 4 (infinite) parses; expectations derived from source, not run | `the_hostile_corpus_matches_its_manifest_under_each_profile` |
| TC-121 FR-061-AC-2 | unbuilt; `fuzz/Cargo.lock` not regenerated | `make fuzz-build fuzz-smoke` |
| TC-123 FR-062-AC-1, StR-044-VC-2 | no digest literals pinned (decision 7); byte identity argued from untouched v1..v4 sources, records and corpora | existing v1..v4 suites green at head |
| TC-124 FR-062-AC-2 | unbuilt | `dialects_v1_to_v4_still_refuse_bracket_free_temporal_text` |
| TC-125 NFR-046-AC-1 | unbuilt | `each_ceiling_accepts_at_its_value_and_refuses_one_past_it` |
| TC-126 NFR-046-AC-2 | unbuilt; Miri list extended with `--test clean_ascii_v6` but not run | `two_runs_produce_byte_equal_reports`, `make miri` |

## Ungrounded items

None. Import front-ends are not in this repository (ADR-041 item 6).
