---
id: PLAN-002
title: "Shared assurance migration"
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-parse/FR-006
    type: references
---
# PLAN-002: Shared assurance migration

## Objective

Move tl-parse from its repository-local evidence framework onto the released
Engineering Assurance, Quire, Quoin and ix-flow contracts, preserving every
parser-domain behaviour and every retained evidence byte.

## Approach

The parser is not the subject of this change. Its lexer, precedence,
diagnostics, spans, limits, formatter, corpus, fuzz target and CLI are carried
across unchanged, and the migration only changes how their results are declared,
transcribed, retained and verified.

Three properties shape the design.

**The driver never produces.** One target, `make assurance-inputs`, runs the
producers. Everything downstream consumes those files, and an absent input is an
error naming that target rather than a step the driver quietly performs itself.

**Every attested result is read from producer bytes.** No verdict is inferred
from an exit code alone or recovered from a transcript. This is the failure that
cost Wave 1 a high finding — a chain that sealed `passed` for every proof without
reading what the producer wrote — and it is designed against here rather than
discovered later.

**Malformed stays malformed.** Six of this repository's seven corpus fixtures are
malformed by design. The migration keeps that a first-class outcome: it does not
fail its proof, and it is never transcribed as a pass.

## Scope

In scope: the pin declaration, the change-assurance declaration, the driver, the
compatibility view, the producers, the Makefile, the hosted workflow, the
specification, and the deletion of the local evidence framework.

Out of scope: parser or formatter behaviour, the corpus contents, the fuzz
target, and any release or publication decision.

## Landing constraints

- Hosted CI is manual-only and is not dispatched by this change.
- Retained evidence bytes are immutable.
- The old generic path is deleted only after both paths have been run against
  the same candidate revision and the result recorded as observed.
