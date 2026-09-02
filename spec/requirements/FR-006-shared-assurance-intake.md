---
id: FR-006
title: Adopt the shared assurance intake path
type: FR
relationships:
  - target: ix://agent-ix/tl-parse/StR-002
    type: implements
---

# FR-006: Adopt the shared assurance intake path

## Description

The repository shall produce its parser, formatter, corpus, round-trip and
provenance results with its own tools in declared structured formats, and shall
obtain every static specification fact from Quire and every retention,
integrity, audit and receipt behaviour from Quoin, without either tool executing
a producer and without a repository-local generic evidence framework.

## Behavior

- Component versions are classified by the compatibility matrix packaged with
  the pinned Engineering Assurance release. This repository observes what is
  installed and restates no version rule of its own.
- One target, `make assurance-inputs`, runs the producers and writes their
  structured results. Everything downstream consumes those files and refuses to
  create them; an absent input is an error naming that target, never a skip.
- Each proof attestation states the verdict read out of the bytes its producer
  wrote. No verdict is inferred from a transcript, an exit code alone, or a
  caller's expectation.
- Malformed parser input is reported as malformed. It does not fail its proof
  obligation, and it is never transcribed as a pass.
- Twelve verification outcomes remain distinguishable across the intake path,
  each demonstrated by a case that produced it, and each negative paired with a
  positive control observed to be accepted.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-006-AC-1 | The adopted component versions are classified by the packaged Engineering Assurance compatibility matrix, not by a local restatement of it, and no component resolves from the internal mirror. | Test (TC-022) |
| FR-006-AC-2 | Native parser, formatter, corpus, round-trip and test-census results are produced by this repository's tools in a declared structured format and transcribed by Quoin without Quoin or Quire executing the producer. | Test (TC-023) |
| FR-006-AC-3 | Static specification, obligation, and coverage facts come from a Quire export that names every requirement in the repository, and Quire executes no producer. | Test (TC-024) |
| FR-006-AC-5 | Pass, fail, unavailable, unsupported, inconclusive, not-computed, malformed, partial, stale, suspect, vacuous, and tampered remain twelve distinguishable states, each demonstrated and each negative paired with a positive control. | Test (TC-026) |
| FR-006-AC-6 | A malformed source rejected with its declared diagnostic is reported as malformed, the count agrees with the corpus manifest's own declaration, and the state survives into the bytes Quoin retained. | Test (TC-027) |
| FR-006-AC-7 | No repository-local generic runner, evidence envelope, manifest, identity framework, retention store, audit store, anchor verifier, or aggregate verdict remains in the execution path. | Test (TC-028) |

## Dependencies

Depends on the released Engineering Assurance, quire-cli, Quoin and ix-flow
pins recorded in `assurance/pins.json`, and on FR-003, FR-004 and FR-005 for the
domain behaviour whose results it transcribes.
