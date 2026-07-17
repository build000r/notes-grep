.PHONY: test install-local coverage crap

CRAP_ANALYZER ?= $(HOME)/repos/opensource/skills/crap/scripts/analyze_crap.py
LLVM_COV ?= $(shell xcrun --find llvm-cov 2>/dev/null || which llvm-cov 2>/dev/null)
LLVM_PROFDATA ?= $(shell xcrun --find llvm-profdata 2>/dev/null || which llvm-profdata 2>/dev/null)
COVERAGE_LINKER ?= $(CURDIR)/scripts/coverage-linker
CARGO_INSTALL_ROOT ?= $(HOME)/.local

test:
	cargo test

install-local:
	cargo install --path . --root "$(CARGO_INSTALL_ROOT)" --force --locked

coverage:
	LLVM_COV="$(LLVM_COV)" LLVM_PROFDATA="$(LLVM_PROFDATA)" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(COVERAGE_LINKER)" cargo llvm-cov --lcov --output-path lcov.info

crap: coverage
	python3 "$(CRAP_ANALYZER)" . --languages rust --top 20
