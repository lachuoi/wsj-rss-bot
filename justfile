set dotenv-load := true

# Default task
default: help

# Show available tasks
help:
    @just --list

# Check the project for errors
check:
    cargo component check --target wasm32-wasip2

# Format the code
fmt:
    cargo fmt

# Lint the code
lint:
    cargo clippy --target wasm32-wasip2 -- -D warnings

# Build the WebAssembly component
build flags="":
    cargo component build --target wasm32-wasip2 {{flags}}

# Build the component in release mode
build-release: (build "--release")

# Clean the build artifacts
clean:
    cargo clean

# Run the project
run flags="": (build flags)
    @wasmtime run \
        -S http \
        -S inherit-network=y \
        -S allow-ip-name-lookup=y \
        -S inherit-env=y \
        --dir . \
        ./target/wasm32-wasip2/$(if [ "{{flags}}" == "--release" ]; then echo "release"; else echo "debug"; fi)/wsj-rss-bot.wasm

# Run the project in release mode
run-release: (run "--release")

# Run the project in dry-run mode (no Mastodon posts)
dry-run flags="": (build flags)
    @DRY_RUN=true wasmtime run \
        -S http \
        -S inherit-network=y \
        -S allow-ip-name-lookup=y \
        -S inherit-env=y \
        --dir . \
        ./target/wasm32-wasip2/$(if [ "{{flags}}" == "--release" ]; then echo "release"; else echo "debug"; fi)/wsj-rss-bot.wasm

