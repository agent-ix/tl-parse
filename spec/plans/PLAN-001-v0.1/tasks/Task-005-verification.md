---
id: Task-005
title: "Corpus, fuzz, CLI, and verification"
type: Task
status: done
track: Verification
priority: P0
relationships:
  - target: ix://agent-ix/tl-parse/PLAN-001
    type: part_of
---
# Task-005: Corpus, fuzz, CLI, and verification

## Scope

Retain hostile/resource fixtures and fuzz seeds, expose thin validate/format
CLI paths, run all local MSRV/lint/test/docs/supply-chain/specification gates,
and resolve agent and pull-request review findings.

## Completion Evidence

Corpus hashes, seed consumption, successful round trips, CLI file/stdin/profile
and exit-class tests, evidence contracts, code review, and gap analysis pass.
Hosted CI remains manual-only and belongs to the finalized Task-006 revision.
