# Shared assurance

This directory holds what tl-parse *declares*. It holds no evidence, no
manifest, no verdict, and no store.

| File | What it is |
|---|---|
| `pins.json` | The Engineering Assurance release this repository adopts, and the digests of the artifacts it reads from that release. Component versions are deliberately not restated: the packaged compatibility matrix is their authority. |
| `change-assurance.json` | The author's statement about the change under issue #10, in the shape Quoin's FR-063 record requires. |

## How the pieces relate

```
make assurance-inputs        the ONLY target that runs a producer
   |
   +-> target/assurance/*    structured results, written by domain tools
          |
          +-> scripts/assurance_chain.py   reads those bytes, seals through quoin
          +-> scripts/legacy_evidence_view.py   reads evidence/ through the pinned mapping
          +-> scripts/check_shared_pins.py      classifies the toolchain through the matrix
```

Three rules make this different from what it replaced.

**The driver never produces.** If an input is absent, the chain says so and
names `make assurance-inputs`. It does not run the producer itself. A driver
that can produce its own inputs can produce a green run out of nothing.

**Every attested result is read from producer bytes.** `derive_result()` reads a
field the producer wrote — row outcomes, `matched`, or cargo's own
`build-finished` message. Nothing is inferred from an exit code alone, and
nothing is scraped from a transcript.

**A refusal is a result.** This repository's twelve retained envelopes are
`quire.derivation-evidence/v1`, which the pinned PGM-01 mapping does not cover,
so it answers `incompatible` for every one of them. That answer is reported as it
stands. It is not a pass, it is not a defect of those records, and it is not a
reason to write a local mapper — which is precisely what this migration removed.
Filed upstream as `agent-ix/engineering-assurance#21`.

## What is not here

No evidence envelope, manifest, anchor file, retention store, audit store, tool
lock, or aggregate verdict. Retained bytes under `evidence/` are immutable, and
Git history plus pull-request review are the integrity boundary for them — which
is what `CONTRIBUTING.md` has always said. The compatibility view asks Git
rather than implying a stronger claim than it can make.
