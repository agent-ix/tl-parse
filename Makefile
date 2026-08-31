# =============================================================================
# TL Parse Makefile
# =============================================================================

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
SHA256SUM ?= sha256sum
BASH ?= bash

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make check-failure-propagation - prove required command failures reach CI"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make check-corpus     - Verify malformed and fuzz-seed corpus bytes"
	@echo "  make fuzz-build       - Compile the checked-in cargo-fuzz target"
	@echo "  make fuzz-smoke       - Execute the checked-in fuzz corpus"
	@echo "  make verify-evidence  - Verify retained evidence SHA-256 manifests"
	@echo "  make spec             - Validate specification and strict coverage"
	@echo "  make msrv             - Check all targets and features with Rust 1.75"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make evidence-tool    - Test evidence tooling and schemas"
	@echo "  make ci               - Complete local gate, including fuzz build and smoke"

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

.PHONY: test
test:
	$(CARGO) test --all-targets --all-features

.PHONY: check-failure-propagation
check-failure-propagation:
	$(PYTHON) scripts/check_failure_propagation.py

.PHONY: check-corpus
check-corpus:
	cd corpus/v1 && $(SHA256SUM) --check SHA256SUMS
	cd fuzz/corpus/parser && $(SHA256SUM) --check SHA256SUMS

.PHONY: fuzz-build
fuzz-build:
	$(CARGO) +nightly fuzz build parser

.PHONY: fuzz-smoke
fuzz-smoke:
	$(BASH) scripts/run_fuzz_smoke.sh

.PHONY: verify-evidence
verify-evidence:
	$(BASH) scripts/verify_evidence.sh

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

.PHONY: evidence-tool
evidence-tool:
	$(PYTHON) -m compileall -q scripts
	$(PYTHON) scripts/check_attribution.py
	$(PYTHON) scripts/test_evidence_tool.py
	$(PYTHON) scripts/test_failure_propagation.py
	$(PYTHON) scripts/test_json_schema_gate.py
	$(PYTHON) scripts/test_traceability_gate.py

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny deny-advisories deny-bans deny-licenses deny-sources
deny: deny-advisories deny-bans deny-licenses deny-sources

deny-advisories:
	$(CARGO) deny check advisories

deny-bans:
	$(CARGO) deny check bans

deny-licenses:
	$(CARGO) deny check licenses

deny-sources:
	$(CARGO) deny check sources

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	$(BASH) scripts/check_unsafe_comments.sh

.PHONY: spec-validate
spec-validate:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/*.md'

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/*.md'
	$(PYTHON) scripts/check_traceability_coverage.py

.PHONY: msrv
msrv:
	$(CARGO) +1.75.0 check --all-targets --all-features

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: check-failure-propagation fmt-check lint test check-corpus fuzz-build fuzz-smoke deny audit-unsafe evidence-tool spec msrv rustdoc verify-evidence
