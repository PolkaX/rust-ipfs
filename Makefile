SHELL := /bin/bash
#ENABLE_FEATURES ?=

default: dev

.PHONY: all

clean:
	cargo clean

## Development builds
## ------------------

all: format build test

dev: format clippy test

build:
	cargo build #--no-default-features --features "${ENABLE_FEATURES}"

## Release builds (optimized dev builds)
## ----------------------------

release:
	cargo build --release #--no-default-features --features "${ENABLE_FEATURES}"

## Testing
## -----

test:
	export LOG_LEVEL=DEBUG && \
	export RUST_BACKTRACE=1 && \
	cargo test --all ${EXTRA_CARGO_ARGS} -- --nocapture $(TEST_THREADS)

## Benchmarking
## -----

bench:
	export LOG_LEVEL=ERROR && \
	export RUST_BACKTRACE=1 && \
	cargo bench

## Static analysis
## ---------------

unset-override:
	@# unset first in case of any previous overrides
	@if rustup override list | grep `pwd` > /dev/null; then rustup override unset; fi

pre-format: unset-override
	@rustup component add rustfmt

format: pre-format
	@cargo fmt --all -- --check >/dev/null || \
	cargo fmt --all

pre-clippy: unset-override
	@rustup component add clippy

clippy: pre-clippy
	@cargo clippy --all --all-targets -- \
		-A clippy::module_inception -A clippy::needless_pass_by_value -A clippy::cognitive_complexity \
		-A clippy::unreadable_literal -A clippy::should_implement_trait -A clippy::verbose_bit_mask \
		-A clippy::implicit_hasher -A clippy::large_enum_variant -A clippy::new_without_default \
		-A clippy::neg_cmp_op_on_partial_ord -A clippy::too_many_arguments \
		-A clippy::excessive_precision -A clippy::collapsible_if -A clippy::blacklisted_name \
		-A clippy::needless_range_loop -A clippy::redundant_closure \
		-A clippy::match_wild_err_arm -A clippy::blacklisted_name -A clippy::redundant_closure_call \
		-A clippy::identity_conversion

pre-audit:
	$(eval LATEST_AUDIT_VERSION := $(strip $(shell cargo search cargo-audit | head -n 1 | awk '{ gsub(/"/, "", $$3); print $$3 }')))
	$(eval CURRENT_AUDIT_VERSION = $(strip $(shell (cargo audit --version 2> /dev/null || echo "noop 0") | awk '{ print $$2 }')))
	@if [ "$(LATEST_AUDIT_VERSION)" != "$(CURRENT_AUDIT_VERSION)" ]; then \
		cargo install cargo-audit --force; \
	fi

# Check for security vulnerabilities
audit: pre-audit
	cargo audit
