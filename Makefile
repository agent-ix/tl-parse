# =============================================================================
# TL Parse Makefile
# =============================================================================

CARGO ?= cargo

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make check-corpus     - Verify malformed and fuzz-seed corpus bytes"
	@echo "  make fuzz-build       - Compile the checked-in cargo-fuzz target"
	@echo "  make fuzz-smoke       - Execute the checked-in fuzz corpus"
	@echo "  make verify-evidence  - Verify retained evidence SHA-256 manifests"
	@echo "  make spec             - Validate specification and strict coverage"
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

.PHONY: check-corpus
check-corpus:
	cd corpus/v1 && sha256sum --check SHA256SUMS
	cd fuzz/corpus/parser && sha256sum --check SHA256SUMS

.PHONY: fuzz-build
fuzz-build:
	cargo +nightly fuzz build parser

.PHONY: fuzz-smoke
fuzz-smoke:
	bash scripts/run_fuzz_smoke.sh

.PHONY: verify-evidence
verify-evidence:
	bash scripts/verify_evidence.sh

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

.PHONY: evidence-tool
evidence-tool:
	python3 -m py_compile scripts/build_evidence_envelope.py scripts/finalize_collection.py scripts/test_evidence_tool.py scripts/validate_json_schema.py scripts/verify_evidence_manifest.py
	python3 scripts/test_evidence_tool.py

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses
	$(CARGO) deny check sources

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

.PHONY: spec-validate
spec-validate:
	quire validate --scope . 'spec/**/*.md' 'docs/*.md'

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md' 'docs/*.md'
	quire coverage --scope . --strict

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test check-corpus fuzz-build fuzz-smoke deny audit-unsafe evidence-tool spec rustdoc verify-evidence
