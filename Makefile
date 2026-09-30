# =============================================================================
# TL Parse Makefile
# =============================================================================
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the corpus conformance runner and the round-trip sweep
# for the parser, quire for static export.
#
# agent-ix/tl-parse#11: the local evidence collector this guard used to
# protect is gone, but the class it caught is real — a `-` prefix, `.IGNORE`,
# or a `SHELL := /usr/bin/true` assignment can make `make ci` exit 0 having
# executed nothing. scripts/check_make_execution_control.sh reinstates a
# minimal, collector-free version of that check: a parse-time scan (below) of
# this file's own text for the constructs that neuter Make's execution
# controls. Because it runs at parse time via $(shell ...) + $(error ...), it
# fires before any target — including a neutered one — gets a chance to run,
# and it re-reads this file from disk so an append anywhere below still gets
# caught.
MAKE_EXECUTION_CONTROL_GUARD := $(shell bash scripts/check_make_execution_control.sh $(MAKEFILE_LIST) 2>&1)
ifneq ($(strip $(MAKE_EXECUTION_CONTROL_GUARD)),)
$(error $(MAKE_EXECUTION_CONTROL_GUARD))
endif

# Ambient environment overrides the text scan above cannot see: MAKEFLAGS
# already carries any -i/--ignore-errors or -s/--silent the invocation passed,
# and PYTHONOPTIMIZE/ASAN_OPTIONS/LSAN_OPTIONS can silently strip Python
# assertions or mute a sanitizer abort in the fuzz lane. All four are refused
# rather than let `make ci` merely appear to have run these gates.
# examples/fuzz_campaign.rs sets its own narrowly-scoped LSAN_OPTIONS
# (fuzz/lsan_suppressions.txt, TL-195) after this guard passes; it is refused
# here only when present ambiently, the same as ASAN_OPTIONS.
#
# GNU Make bundles single-letter flags into one no-dash word (`-ik` and
# `-i -k` both become the MAKEFLAGS word "ki"), except when a `-j` jobserver
# is active, which forces -i back out to its own standalone "-i" word
# (`-j -i` -> "j -i"). A bare $(filter i,$(MAKEFLAGS)) only matches the exact
# lone word "i" and misses both of those (FND-001). Check every MAKEFLAGS
# word instead: skip anything carrying "=" (jobserver-fds and other
# long-option arguments), then look for "i" once leading dashes are gone.
MAKEFLAGS_IGNORE_ERRORS := $(strip $(foreach w,$(MAKEFLAGS),$(if $(findstring =,$(w)),,$(if $(findstring i,$(subst -,,$(w))),$(w)))))
ifneq ($(MAKEFLAGS_IGNORE_ERRORS),)
$(error execution-control guard: -i/--ignore-errors was passed to make (MAKEFLAGS="$(MAKEFLAGS)"); every recipe would report success without checking its exit code)
endif
ifdef PYTHONOPTIMIZE
$(error execution-control guard: PYTHONOPTIMIZE is set in the environment; Python assert statements in scripts/ would be stripped)
endif
ifdef ASAN_OPTIONS
$(error execution-control guard: ASAN_OPTIONS is set in the environment; it can silence a sanitizer abort in the fuzz lane)
endif
ifdef LSAN_OPTIONS
$(error execution-control guard: LSAN_OPTIONS is set in the environment; it can silence a sanitizer abort in the fuzz lane)
endif

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make conformance      - Replay the hostile-input corpus through the crate"
	@echo "  make roundtrip        - Sweep the parse-format-parse fixed point"
	@echo "  make test-census      - Bind requirement-tagged tests to compiled tests"
	@echo "  make fuzz-build       - Compile the checked-in cargo-fuzz target"
	@echo "  make fuzz-smoke       - Execute the checked-in fuzz corpus"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make test-execution-control-guard - Self-test the Make execution-control guard"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.98.1"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
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

.PHONY: test
test:
	$(CARGO) test --all-targets --all-features

# =============================================================================
# Parser domain
# =============================================================================

.PHONY: conformance
conformance:
	$(CARGO) run --quiet --example corpus_conformance -- --manifest corpus/v1/manifest.json

.PHONY: roundtrip
roundtrip:
	$(CARGO) run --quiet --release --example roundtrip_sweep

.PHONY: test-census
test-census:
	$(PYTHON) scripts/rust_test_census.py

.PHONY: fuzz-build
fuzz-build:
	rustup run nightly cargo fuzz build parser --target-dir "$${CARGO_TARGET_DIR:-target}/fuzz"
	rustup run nightly cargo fuzz build clean_ascii_v2 --target-dir "$${CARGO_TARGET_DIR:-target}/fuzz"

.PHONY: fuzz-smoke
fuzz-smoke:
	$(CARGO) run --quiet --example fuzz_campaign -- parser
	$(CARGO) run --quiet --example fuzz_campaign -- clean_ascii_v2

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
	$(CARGO) deny check advisories
	$(CARGO) deny check bans
	$(CARGO) deny check licenses
	$(CARGO) deny check sources

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

# Regression coverage for scripts/check_make_execution_control.sh and the
# MAKEFLAGS check above (agent-ix/tl-parse#11), reproducing each bypass an
# independent review found plus a control per fix. This is the guard's own
# self-test, not the guard itself — the guard runs unconditionally at parse
# time for every invocation of this Makefile, this target runs it once as
# part of the gate set.
.PHONY: test-execution-control-guard
test-execution-control-guard:
	bash scripts/check_make_execution_control.sh --self-test

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md' 'docs/*.md' --strict --summary
	$(QUIRE) coverage --scope . --strict

.PHONY: msrv
msrv:
	rustup run 1.98.1 $(CARGO) check --locked --all-targets --all-features

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test conformance roundtrip test-census \
	fuzz-build fuzz-smoke deny audit-unsafe test-execution-control-guard spec \
	msrv rustdoc
