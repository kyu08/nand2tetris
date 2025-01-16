export RUST_BACKTRACE=full

# こんな感じで任意の対象ファイルを指定すると指定したFoo.asmファイルと同階層にgen.Foo.hackが生成される
run-example:
	cargo run -- test_data/pong/Pong.asm

ci:
	@make test-ci; \
	make check; \
	make fmt;

tool-test:
	@if ! which cargo-nextest > /dev/null; then \
		cargo install cargo-nextest; \
	fi

test: tool-test
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo nextest run

build:
	@cargo build

fmt:
	@cargo fmt -- --check

check:
	@cargo clippy -- -D warnings

.PHONY: run-example ci tool-test test build fmt check
