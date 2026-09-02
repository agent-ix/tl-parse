# =============================================================================
# TL Parse Makefile
# =============================================================================
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the corpus conformance runner and the round-trip sweep
# for the parser, quire for static export, quoin for evidence. Nothing here
# computes a verdict, attests to its own correctness, or retains evidence of its
# own.
#
# This file is not a trust root and no longer tries to be one. The parse-time
# guards that used to police Make's own execution controls — SHELL, .SHELLFLAGS,
# MAKEFLAGS, the `-` prefix, $(eval) — went with the collector they were
# protecting. That is a deliberate reduction in local detection and it is
# recorded as an open unknown in assurance/change-assurance.json rather than
# claimed away: what replaces them is structural, not another guard. Quoin binds
# each retained input by digest and the chain derives every attested result from
# the producer's own bytes, so a Makefile that lies about what it ran produces an
# absent or empty input, and the chain reports that by name instead of passing.

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
QUOIN ?= quoin

# The shared-assurance lane runs in its own interpreter. Unlike tl-syntax there
# is no jsonschema conflict to resolve here — nothing in this repository imports
# jsonschema once the local evidence machinery is gone. The environment exists
# because engineering-assurance is pinned as a git tag, and resolving a git
# dependency into the system interpreter would make the pin depend on whatever
# else that interpreter happens to have.
ASSURANCE_VENV ?= .venv-assurance
ASSURANCE_PYTHON ?= $(ASSURANCE_VENV)/bin/python

ASSURANCE_DIR := target/assurance
CONFORMANCE_RESULT := $(ASSURANCE_DIR)/parser-conformance.jsonl
ROUNDTRIP_RESULT := $(ASSURANCE_DIR)/roundtrip-property.jsonl
CENSUS_RESULT := $(ASSURANCE_DIR)/test-census.json
ATTRIBUTION_RESULT := $(ASSURANCE_DIR)/attribution.json
QUIRE_EXPORT := $(ASSURANCE_DIR)/quire-static-export.json
COMPAT_RESULT := $(ASSURANCE_DIR)/legacy-compatibility.json
MSRV_RESULT := $(ASSURANCE_DIR)/msrv.jsonl
REVISION ?= $(shell git rev-parse HEAD)

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test plus the shared-assurance tests"
	@echo "  make check-corpus     - Verify malformed and fuzz-seed corpus bytes"
	@echo "  make conformance      - Replay the hostile-input corpus through the crate"
	@echo "  make roundtrip        - Sweep the parse-format-parse fixed point"
	@echo "  make test-census      - Bind requirement-tagged tests to compiled tests"
	@echo "  make attribution      - Re-derive the clean-room attribution digests"
	@echo "  make fuzz-build       - Compile the checked-in cargo-fuzz target"
	@echo "  make fuzz-smoke       - Execute the checked-in fuzz corpus"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.75"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean and drop the assurance environment"
	@echo "  make assurance-env    - Create the pinned shared-assurance interpreter"
	@echo "  make assurance-inputs - Run the producers and write their structured results"
	@echo "  make pins             - Classify the toolchain through the shared matrix"
	@echo "  make compat-view      - Read retained evidence through the shared mapping"
	@echo "  make assurance-chain  - Seal, retain, and verify through Quoin"
	@echo "  make assurance        - pins + compat-view + assurance-chain"
	@echo "  make ci               - All CI gates locally (hosted CI is manual-only)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# The traced tests invoke the assurance gates, so the producers must already have
# run. They are a prerequisite rather than something a test creates for itself: a
# test that can produce its own inputs can produce a green run out of nothing.
.PHONY: test
test: assurance-inputs
	$(CARGO) test --all-targets --all-features

# =============================================================================
# Parser domain
# =============================================================================

.PHONY: check-corpus
check-corpus:
	$(PYTHON) scripts/check_checksum_manifest.py corpus/v1
	$(PYTHON) scripts/check_checksum_manifest.py fuzz/corpus/parser

.PHONY: conformance
conformance:
	$(CARGO) run --quiet --example corpus_conformance -- --manifest corpus/v1/manifest.json

.PHONY: roundtrip
roundtrip:
	$(CARGO) run --quiet --release --example roundtrip_sweep

.PHONY: test-census
test-census:
	$(PYTHON) scripts/rust_test_census.py

.PHONY: attribution
attribution:
	$(PYTHON) scripts/check_attribution.py

.PHONY: fuzz-build
fuzz-build:
	rustup run nightly cargo fuzz build parser --target-dir "$${CARGO_TARGET_DIR:-target}/fuzz"

.PHONY: fuzz-smoke
fuzz-smoke:
	bash scripts/run_fuzz_smoke.sh

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean
	rm -rf $(ASSURANCE_VENV)

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check advisories
	$(CARGO) deny check bans
	$(CARGO) deny check licenses
	$(CARGO) deny check sources

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/*.md' --strict --summary
	$(QUIRE) coverage --scope . --strict

.PHONY: msrv
msrv:
	rustup run 1.75.0 $(CARGO) check --locked --all-targets --all-features

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

# =============================================================================
# Shared assurance
# =============================================================================

$(ASSURANCE_PYTHON):
	$(PYTHON) -m venv $(ASSURANCE_VENV)
	$(ASSURANCE_VENV)/bin/pip install --quiet --disable-pip-version-check \
		-r requirements-assurance.txt

.PHONY: assurance-env
assurance-env: $(ASSURANCE_PYTHON)

# The only target that runs a producer. Everything downstream consumes these
# files and refuses to create them.
.PHONY: assurance-inputs
assurance-inputs: assurance-env
	mkdir -p $(ASSURANCE_DIR)
	$(CARGO) run --quiet --example corpus_conformance -- \
		--manifest corpus/v1/manifest.json > $(CONFORMANCE_RESULT)
	$(CARGO) run --quiet --release --example roundtrip_sweep > $(ROUNDTRIP_RESULT)
	$(PYTHON) scripts/rust_test_census.py --json > $(CENSUS_RESULT)
	$(PYTHON) scripts/check_attribution.py --json > $(ATTRIBUTION_RESULT)
	$(QUIRE) coverage --scope . --json > $(QUIRE_EXPORT)
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py --json > $(COMPAT_RESULT)
	rustup run 1.75.0 $(CARGO) check --locked --all-targets --all-features \
		--message-format=json > $(MSRV_RESULT)

.PHONY: pins
pins: assurance-env
	$(ASSURANCE_PYTHON) scripts/check_shared_pins.py

.PHONY: compat-view
compat-view: assurance-env
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py --mutation-probes

.PHONY: assurance-chain
assurance-chain: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --candidate-revision $(REVISION)

.PHONY: assurance
assurance: pins compat-view assurance-chain

# An operator target, not a CI gate. It writes into this repository's own Quoin
# evidence store, which is a reviewed change to spec/evidence/ rather than
# something a gate should do on every run.
.PHONY: assurance-record
assurance-record: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --adapt $(CONFORMANCE_RESULT) \
		> $(ASSURANCE_DIR)/entries.json
	$(QUOIN) evidence record \
		--repo . \
		--suite SUITE-001 \
		--commit $(REVISION) \
		--tool "tl-parse-corpus-conformance 0.1.0" \
		--adapter entries \
		--kind Integration \
		--results $(ASSURANCE_DIR)/entries.json

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test check-corpus conformance roundtrip test-census attribution \
	fuzz-build fuzz-smoke deny audit-unsafe spec msrv rustdoc assurance
