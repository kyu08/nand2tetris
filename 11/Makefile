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
	RUST_BACKTRACE=1 cargo nextest run

test-watch: tool-test
	RUST_BACKTRACE=1 cargo watch -x "nextest run"

build:
	@cargo build

fmt:
	@cargo fmt -- --check

check:
	@cargo clippy -- -D warnings

.PHONY: ci tool-test test test-watch build fmt check
