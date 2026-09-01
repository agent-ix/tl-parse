# =============================================================================
# TL Parse Makefile
# =============================================================================

ifneq ($(filter ci ci-for-evidence,$(MAKECMDGOALS)),)
ifneq ($(strip $(MAKEFLAGS)),)
$(error local CI refuses non-empty MAKEFLAGS)
endif
ifneq ($(strip $(PYTHONOPTIMIZE)),)
$(error local CI refuses optimized Python policy execution)
endif
ifneq ($(strip $(ASAN_OPTIONS)),)
$(error local CI refuses ambient ASAN_OPTIONS)
endif
ifneq ($(origin CARGO),undefined)
$(error local CI refuses a CARGO override)
endif
ifneq ($(origin PYTHON),undefined)
$(error local CI refuses a PYTHON override)
endif
ifneq ($(origin QUIRE),undefined)
$(error local CI refuses a QUIRE override)
endif
ifneq ($(origin SHA256SUM),undefined)
$(error local CI refuses a SHA256SUM override)
endif
ifneq ($(origin BASH),undefined)
$(error local CI refuses a BASH override)
endif
tl_ci_static_status := $(shell /usr/bin/env -u PYTHONOPTIMIZE MAKEFLAGS= /usr/bin/python3 scripts/check_failure_propagation.py --makefile '$(firstword $(MAKEFILE_LIST))' --static-only >/dev/null; echo $$?)
ifneq ($(tl_ci_static_status),0)
$(error local CI refuses unsafe Make recipe controls)
endif
endif

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
	@echo "  make ci-for-evidence  - Candidate gates before the candidate can self-anchor"
	@echo "  make ci               - Complete local gate, including fuzz build and smoke"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check
	@/usr/bin/printf 'fmt-check gate passed\n'

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings
	@/usr/bin/printf 'lint gate passed\n'

.PHONY: test
test:
	cargo test --all-targets --all-features
	@/usr/bin/printf 'Rust test gate passed\n'

.PHONY: check-failure-propagation
check-failure-propagation:
	/usr/bin/python3 scripts/check_failure_propagation.py

.PHONY: check-corpus
check-corpus:
	/usr/bin/python3 scripts/check_checksum_manifest.py corpus/v1
	/usr/bin/python3 scripts/check_checksum_manifest.py fuzz/corpus/parser
	@/usr/bin/printf 'corpus-integrity gate passed\n'

.PHONY: fuzz-build
fuzz-build:
	cargo +nightly fuzz build parser
	@/usr/bin/printf 'fuzz-build gate passed\n'

.PHONY: fuzz-smoke
fuzz-smoke:
	/usr/bin/bash scripts/run_fuzz_smoke.sh
	@/usr/bin/printf 'fuzz-smoke gate passed\n'

.PHONY: verify-evidence
verify-evidence:
	/usr/bin/bash scripts/verify_evidence.sh
	@/usr/bin/printf 'verify-evidence gate passed\n'

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features
	@/usr/bin/printf 'rustdoc gate passed\n'

.PHONY: evidence-tool
evidence-tool:
	/usr/bin/python3 -m compileall -q scripts
	/usr/bin/python3 scripts/check_attribution.py
	/usr/bin/python3 scripts/run_policy_tests.py
	@/usr/bin/printf 'evidence-tool gate passed\n'

.PHONY: build
build:
	cargo build --release

.PHONY: clean
clean:
	cargo clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny deny-advisories deny-bans deny-licenses deny-sources
deny: deny-advisories deny-bans deny-licenses deny-sources
	@/usr/bin/printf 'deny gate passed\n'

deny-advisories:
	cargo deny check advisories

deny-bans:
	cargo deny check bans

deny-licenses:
	cargo deny check licenses

deny-sources:
	cargo deny check sources

.PHONY: cargo-audit
cargo-audit:
	cargo audit

.PHONY: audit-unsafe
audit-unsafe:
	/usr/bin/bash scripts/check_unsafe_comments.sh
	@/usr/bin/printf 'audit-unsafe gate passed\n'

.PHONY: spec-validate
spec-validate:
	quire validate --scope . 'spec/**/*.md' 'docs/*.md'

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md' 'docs/*.md'
	/usr/bin/python3 scripts/check_traceability_coverage.py
	@/usr/bin/printf 'spec gate passed\n'

.PHONY: msrv
msrv:
	cargo +1.75.0 check --all-targets --all-features
	@/usr/bin/printf 'msrv gate passed\n'

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci ci-for-evidence
ci-for-evidence:
	/usr/bin/python3 scripts/run_local_ci.py

ci:
	/usr/bin/bash scripts/verify_evidence.sh
	/usr/bin/python3 scripts/run_local_ci.py --include-verify
