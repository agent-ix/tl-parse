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
outcomes distinguishable, and grant no release authority.

## Scope

This requirement owns the shared-assurance intake path: the pinned toolchain
declaration in `assurance/pins.json`, the change-assurance declaration in
`assurance/change-assurance.json`, the driver `scripts/assurance_chain.py`, the
pin classifier `scripts/check_shared_pins.py`, and the tests that exercise them.

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
| Automatic release decisions | 0 | 0 | Inspection |

## Verification

Behaviour tests invoke the gates rather than reimplementing them. The producer
boundary is asserted with two runs — producers replaced by logging stubs with the
log required to be empty, and a control that stubs the tool the chain does use
and requires the chain to fail — because an empty log and an unconsulted `PATH`
are otherwise the same observation. Mutation probes remove one load-bearing
check at a time and require the corresponding gate to go red.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every attested proof result is derived from the producer's own structured output; a producer whose output is absent, empty, or unreadable is an error naming the target that writes it, and never a pass. | Test (TC-023) |
| NFR-003-AC-2 | Neither Quire nor Quoin executes a producer, demonstrated by stubbing every producer and requiring no invocation, together with a control that stubs Quoin and requires the chain to fail. | Test (TC-023) |
| NFR-003-AC-3 | The twelve verification outcomes stay distinguishable, each demonstrated by a case that produced it and matched, with every negative paired with a positive control and a control naming a non-existent scenario refused. The dangling-control fixture owns its Quoin store, shares only produced inputs, and proves the unmutated chain succeeds in the same scratch. | Test (TC-026) |

## Qualification Boundary

These controls make a presented candidate and its produced results reproducible
and reviewable. They confer no qualification, certification, or
accreditation. Branch protection and the remote review history, not the local
repository, establish resistance to history replacement.
