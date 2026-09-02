---
id: Task-002
title: "Producers, adapter, and the shared intake path"
type: Task
status: done
track: Intake
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-parse/FR-006
    type: references
---
# Task-002: Producers, adapter, and the shared intake path

## Scope

Emit every domain result in a declared structured format, transcribe it through
a native adapter, seal and retain it through Quoin, obtain static facts from
Quire, and read retained evidence through the shared compatibility mapping —
with neither tool executing a producer.

## Completion Evidence

`examples/corpus_conformance.rs` and `examples/roundtrip_sweep.rs` are the two
new domain producers; `scripts/rust_test_census.py` and
`scripts/check_attribution.py` gained structured output. `make assurance-inputs`
is the only target that runs any of them. `scripts/assurance_chain.py` derives
each attested result from the producer's own bytes, and
`scripts/legacy_evidence_view.py` reads all twelve retained envelopes through
`map_pgm01_bytes` without a byte moving. `tests/shared_assurance.rs` asserts the
producer boundary with logging shims plus a control that stubs Quoin and
requires the chain to fail.
