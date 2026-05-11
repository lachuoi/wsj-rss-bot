# mstd-wsj-rss

A Mastodon bot that fetches Wall Street Journal (WSJ) RSS feeds and posts new articles to a Mastodon instance. Built as a synchronous **WASI P2 (WebAssembly System Interface)** command for maximum portability and secure, efficient execution.

## Features

- **Multi-feed Support**: Fetches and processes various WSJ news feeds from a remote HJSON configuration.
- **WASI P2 Command**: Built as a standard WASI component using `wasm32-wasip2`, designed for direct execution in runtimes like `wasmtime`.
- **Synchronous Architecture**: Uses blocking WASI HTTP and socket calls for a simpler, more robust execution model without the overhead of an async runtime.
- **La Chuoi Integration**: Uses the La Chuoi JSON-RPC 2.0 interface for persistent state management and task tracking.
- **Duplicate Key Strategy**: Leverages the runtime's support for duplicate keys to maintain a robust history of posted links.
- **Mastodon Integration**: Automatically formats and posts new articles to a configured Mastodon account.
- **HTML to Text**: Converts RSS descriptions to clean text suitable for Mastodon posts.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-component](https://github.com/bytecodealliance/cargo-component)
- [wasmtime](https://wasmtime.dev/) (>= 22.0 recommended)
- [just](https://github.com/casey/just) (task runner)
- A [La Chuoi](https://github.com/lachuoi/lachuoi) distributed runtime environment
- A Mastodon account and [Access Token](https://docs.joinmastodon.org/methods/apps/)

## Setup

1.  Clone the repository.
2.  Copy `sample.env` to `.env` and fill in your credentials:

    ```bash
    cp sample.env .env
    ```

    Required variables:
    - `RPC_ENDPOINT`: The HTTP URI of the La Chuoi JSON-RPC service.
    - `LACHUOI_TOKEN`: Your one-time authentication token.
    - `APP_ID`: Your unique numeric task ID.
    - `MSTD_ACCESS_TOKEN`: Your Mastodon application access token.
    - `MSTD_API_URI`: Your Mastodon instance URL (e.g., `https://mastodon.social`).
    - `ENVIRONMENT`: (Optional) `production` or `development`.

## Development Workflow

The project uses `just` to automate common tasks:

- **Build**: `just build` (compiles to `wasm32-wasip2`)
- **Run**: `just run` (builds and executes with `wasmtime`)
- **Format**: `just fmt`
- **Lint**: `just lint`
- **Containerize**: `just image-build` (builds a minimal OCI image via `podman`)

## Running with Wasmtime

The bot requires specific permissions for network and environment access. These are automatically handled by `just run`, but for manual execution:

```bash
wasmtime run \
    -S http \
    -S inherit-network=y \
    -S allow-ip-name-lookup=y \
    -S inherit-env=y \
    ./target/wasm32-wasip2/release/mstd-wsj-rss.wasm
```

## Project Structure

- `src/main.rs`: Orchestrates the fetching and posting logic.
- `src/db.rs`: Implements synchronous Turso-backed key-value storage using WASI HTTP.
- `src/wasi_http.rs`: A clean, blocking wrapper for the WASI HTTP outgoing handler.

## License

Copyright (c) 2026 Seungjin Kim.

This project is licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.
