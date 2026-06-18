build:
	cargo build

run:
	cargo run

release:
	cargo build --release

check:
	cargo check

fmt:
	cargo fmt

clippy:
	cargo clippy

clean:
	cargo clean

.PHONY: build run release check fmt clippy clean
