.PHONY: test coverage crap

CRAP_ANALYZER ?= $(HOME)/repos/opensource/skills/crap/scripts/analyze_crap.py
LLVM_COV ?= $(shell xcrun --find llvm-cov 2>/dev/null || which llvm-cov 2>/dev/null)
LLVM_PROFDATA ?= $(shell xcrun --find llvm-profdata 2>/dev/null || which llvm-profdata 2>/dev/null)

test:
	cargo test

coverage:
	LLVM_COV="$(LLVM_COV)" LLVM_PROFDATA="$(LLVM_PROFDATA)" cargo llvm-cov --lcov --output-path lcov.info

crap: coverage
	python3 "$(CRAP_ANALYZER)" . --languages rust --top 20
