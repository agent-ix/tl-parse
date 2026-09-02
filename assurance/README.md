# Shared assurance

This directory holds what tl-parse *declares*. It holds no evidence, no
manifest, no verdict, and no store.

| File | What it is |
|---|---|
| `pins.json` | The Engineering Assurance release this repository adopts, and the digests of the artifacts it reads from that release. Component versions are deliberately not restated: the packaged compatibility matrix is their authority. |
| `change-assurance.json` | The author's statement about the change under issue #13, in the shape Quoin's FR-063 record requires. |

## How the pieces relate

```
make assurance-inputs        the ONLY target that runs a producer
   |
   +-> target/assurance/*    structured results, written by domain tools
          |
          +-> scripts/assurance_chain.py   reads those bytes, seals through quoin
          +-> scripts/check_shared_pins.py classifies the toolchain through the matrix
```

Two rules make this different from what it replaced.

**The driver never produces.** If an input is absent, the chain says so and
names `make assurance-inputs`. It does not run the producer itself. A driver
that can produce its own inputs can produce a green run out of nothing.

**Every attested result is read from producer bytes.** `derive_result()` reads a
field the producer wrote — row outcomes, `matched`, or cargo's own
`build-finished` message. Nothing is inferred from an exit code alone, and
nothing is scraped from a transcript.

## What is not here

No evidence envelope, manifest, anchor file, retention store, audit store, tool
lock, or aggregate verdict — and, since 2026-09-02, no retained evidence and no
compatibility view over it.

The twelve records this repository kept under `evidence/` were early-development
output. The repository owner released the evidence-preservation constraint for
the pre-stable phase, recorded in `agent-ix/engineering-assurance#7`, and the
records were deleted rather than carried forward. Nothing here claims they still
verify anything; `agent-ix/engineering-assurance#21`, which tracked the mapping's
refusal to read them, is moot rather than fixed. The constraint re-applies
unchanged when this repository moves toward stable releases.

Git history plus pull-request review remain the integrity boundary for what this
repository does keep, which is what `CONTRIBUTING.md` has always said.
