export RUST_BACKTRACE=full

ci:
	@make test-ci; \
	make check; \
	make fmt;

tool-test:
	@if ! which cargo-nextest > /dev/null; then \
		cargo install cargo-nextest; \
	fi

test: tool-test
	RUST_BACKTRACE=full cargo nextest run

build:
	@cargo build

fmt:
	@cargo fmt -- --check

check:
	@cargo clippy -- -D warnings

.PHONY: ci tool-test test build fmt check

