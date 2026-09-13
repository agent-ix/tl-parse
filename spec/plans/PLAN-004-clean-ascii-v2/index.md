---
type: index
title: "PLAN-004 - Clean-ascii v2 derived future operators"
description: "Contents of the tl-parse clean-ascii v2 derived future-operator plan bundle."
---

# PLAN-004 - Clean-ascii v2 derived future operators

## Contents

| Item | Status | Evidence |
|---|---|---|
| tl-syntax lowering pin | complete | `Cargo.toml`, `TL_SYNTAX_REVISION`, `docs/ATTRIBUTION.md` |
| Dialect and requirement | complete | `docs/DIALECT-002-clean-ascii-v2.md`, FR-008 |
| Parser lowering and report | complete | `src/lexer.rs`, `src/parser.rs`, `src/derived.rs` |
| Acceptance matrix | complete | `tests/clean_ascii_v2.rs`, TC-039 through TC-045 |
| Fuzz target and local gates | complete | `fuzz/fuzz_targets/clean_ascii_v2.rs`, `fuzz/corpus/clean_ascii_v2`, plan log |
