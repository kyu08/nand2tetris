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
	RUST_BACKTRACE=full FZF_MAKE_IS_TESTING=true cargo nextest run

# TODO: ここはあとで編集する
# 1. 対象の.asmを選択
# 2. 生成された.hackファイルと正解の.hackファイルのdiffを表示する
DEBUG_EXECUTABLE = ./target/debug/assembler
TEST_DIR = ./test_data
run-in-test-data: build
	@TARGET_DIR=$$(find $(TEST_DIR) -type d -maxdepth 1 | fzf) && \
	if [ -n "$$TARGET_DIR" ]; then \
		mv $(DEBUG_EXECUTABLE) "$${TARGET_DIR}" && cd "$${TARGET_DIR}" && ./fzf-make; \
	else \
	    echo "No directory selected. Staying in the current directory."; \
	fi

build:
	@cargo build

fmt:
	@cargo fmt -- --check

check:
	@cargo clippy -- -D warnings

.PHONY: ci tool-test test run-in-test-data build fmt check
