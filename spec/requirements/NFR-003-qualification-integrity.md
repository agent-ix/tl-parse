---
id: NFR-003
title: Make qualification controls explicit and fail closed
type: NFR
quality_attribute: reliability
---

# NFR-003: Make qualification controls explicit and fail closed

## Statement

Candidate qualification shall keep the producer boundary observable, derive every
attested result from the bytes a producer wrote, keep the twelve verification
outcomes distinguishable, preserve unique tracked specification-review
identities, bind the hosted ix-flow executable to its scoped package identity,
keep hosted CI manual-only, and grant no release authority.

## Scope

This requirement owns the shared-assurance intake path: the pinned toolchain
declaration in `assurance/pins.json`, the change-assurance declaration in
`assurance/change-assurance.json`, the driver `scripts/assurance_chain.py`, the
pin classifier `scripts/check_shared_pins.py`, and the tests that exercise them.
It also owns `.github/workflows/ci.yml` for the hosted workflow's exact ix-flow
package identity and trigger surface. Workflow comments are explanatory text,
not executable package installs or triggers.

It no longer owns `tools.lock`, a local-CI runner, Make execution-control
probes, a collector, a finalizer, a manifest verifier, an anchor file, or a
retraction registry. Those were removed with the local evidence framework they
belonged to. It no longer owns a retained-evidence compatibility view either:
the records that view read were deleted under the pre-stable release of the
preservation constraint recorded in `agent-ix/engineering-assurance#7`.

That is a real reduction in local detection, and the extent of it is stated here
rather than minimised. Adding `.IGNORE:` to the `Makefile` makes every recipe
report success without running, and nothing in this repository inspects Make's
own execution controls to notice.

A structural backstop exists but covers only part of the gate set. Quoin binds
each retained input by digest and every attested result is derived from the
producer's own bytes, so a *producer* that did not run yields an absent or empty
input the chain names. That protects the five targets whose work is re-run
inside `make assurance-inputs`. It does **not** protect a gate whose recipe
writes nothing the chain reads: `fmt-check`, `lint`, `test`, `check-corpus`, `fuzz-build`, `fuzz-smoke`, `deny`, `audit-unsafe`, `rustdoc`, and the `quire validate` half of `spec`
can each be neutered and every remaining check stays green. This was found by an
adversarial review of this change, not predicted by it.

The residue is recorded as an open unknown in the change-assurance declaration
and tracked as `agent-ix/tl-parse#11`, which carries the reproduction.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Components classified by the packaged matrix | 4/4 | 4/4 | Test |
| Verification outcomes demonstrated and matched | 12/12 | 12/12 | Test |
| Negatives without an accepted positive control | 0 | 0 | Test |
| Attested results not derived from producer bytes | 0 | 0 | Test |
| Duplicate normalized identities among tracked SpecReview artifacts | 0 | 0 | Test |
| Executable ix-flow package identities in the hosted workflow | exactly `@agent-ix/ix-flow@0.0.4` once | exactly one scoped identity and zero aliases | Test |
| Automatic hosted-workflow triggers | 0 | 0 | Test |
| Automatic release decisions | 0 | 0 | Inspection |

## Verification

Behaviour tests invoke the gates rather than reimplementing them. The producer
boundary is asserted with two runs — producers replaced by logging stubs with the
log required to be empty, and a control that stubs the tool the chain does use
and requires the chain to fail — because an empty log and an unconsulted `PATH`
are otherwise the same observation. Mutation probes remove one load-bearing
check at a time and require the corresponding gate to go red.
The review-identity census reads every version-control-tracked SpecReview
frontmatter, normalizes matching YAML quotes, refuses an empty population, and
reports every identity with more than one owning path.
The hosted-workflow census first isolates scalar YAML `run` values, including
quoted `run` keys and literal blocks, and then tokenizes their shell commands.
YAML metadata never enters the executable population. Inside a run script, `#`
starts a shell comment only at a word boundary; a word-internal hash remains
part of its argument. The census inspects package arguments consumed by npm
`install`, `add`, `i`, `in`, `ins`, `inst`, `insta`, `instal`, `isnt`, `isnta`,
`isntal`, or `isntall`, requires exactly one scoped ix-flow package at the
pinned version, rejects every alternate or duplicate identity, verifies the
installed executable's version, and requires `workflow_dispatch` to be the sole
trigger. Independent review of the workflow file is the second control against
coordinated scanner-and-expected-side edits.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every attested proof result is derived from the producer's own structured output; a producer whose output is absent, empty, or unreadable is an error naming the target that writes it, and never a pass. | Test (TC-023) |
| NFR-003-AC-2 | Neither Quire nor Quoin executes a producer, demonstrated by stubbing every producer and requiring no invocation, together with a control that stubs Quoin and requires the chain to fail. | Test (TC-023) |
| NFR-003-AC-3 | The twelve verification outcomes stay distinguishable, each demonstrated by a case that produced it and matched, with every negative paired with a positive control and a control naming a non-existent scenario refused. The dangling-control fixture owns its Quoin store, shares only produced inputs, resolves the repository store without requiring that leaf to exist, and proves the unmutated chain succeeds in the same scratch. | Test (TC-026) |
| NFR-003-AC-4 | Every version-control-tracked SpecReview artifact has one unique normalized frontmatter identity; matching plain and quoted YAML spellings collide, and an empty tracked review population is refused rather than reported as unique. | Test (TC-029) |
| NFR-003-AC-5 | Across semantic `jobs.*.steps[*].run` string scalars, independent of YAML key spelling, spacing, block style, or flow style, the package arguments consumed by a bare or path-qualified npm executable at command position after leading assignments, directly or through shell groups and a statically literal `sh`/`bash -c` script, by every documented npm-install alias (`install`, `add`, `i`, `in`, `ins`, `inst`, `insta`, `instal`, `isnt`, `isnta`, `isntal`, `isntall`) contain exactly one ix-flow identity, `@agent-ix/ix-flow@0.0.4`, and no unscoped, npm-alias, git/GitHub, URL, file, workspace/link, or duplicate alternate; any run script containing unquoted shell redirection is rejected as unsupported rather than partially scanned. Npm- or shell-shaped data arguments and `sh`/`bash` invocations without `-c` are not nested commands and do not suppress later commands in the same scalar, YAML metadata and YAML/shell comments do not enter that population, a shell `#` after any started word (including an empty quoted word) remains executable argument content, the semantic trigger set is exactly [`workflow_dispatch`], and `ix-flow --version` resolves to exactly `0.0.4` under the released local toolchain. | Test (TC-032) |

## Qualification Boundary

These controls make a presented candidate and its produced results reproducible
and reviewable. They confer no qualification, certification, or
accreditation. Branch protection and the remote review history, not the local
repository, establish resistance to history replacement.
