.PHONY: all run-dev release-web release
all: run-dev
run-dev:
	cargo run --features bevy/dynamic_linking
release-web:
	cargo build --profile wasm-release
release:
	cargo build --release
