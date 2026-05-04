# mstd-wsj-rss

A Mastodon bot that fetches Wall Street Journal (WSJ) RSS feeds and posts new articles to a Mastodon instance. Built as a WebAssembly (WASI) component for portable and secure execution.

## Features

- **Multi-feed Support**: Fetches various WSJ news feeds.
- **WASI Component**: Built using `wasm32-wasip2` for modern WASM runtimes.
- **Turso Integration**: Uses Turso (libSQL) via HTTP Pipeline API for persistent state (tracking last-seen articles).
- **Mastodon Integration**: Automatically posts new articles to a configured Mastodon account.
- **HTML to Text**: Converts RSS descriptions to clean text for Mastodon posts.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [cargo-component](https://github.com/bytecodealliance/cargo-component)
- [wasmtime](https://wasmtime.dev/)
- [just](https://github.com/casey/just) (optional, but recommended for task automation)
- A [Turso](https://turso.tech/) database
- A Mastodon account and [Access Token](https://docs.joinmastodon.org/methods/apps/)

## Setup

1.  Clone the repository.
2.  Copy `sample.env` to `.env` and fill in your credentials:

    ```bash
    cp sample.env .env
    ```

    Required variables:
    - `TURSO_DATABASE_URL`: Your Turso database URL (e.g., `libsql://db-name.turso.io`).
    - `TURSO_AUTH_TOKEN`: Your Turso authentication token.
    - `MSTD_ACCESS_TOKEN`: Your Mastodon application access token.
    - `MSTD_API_URI`: Your Mastodon instance URL (e.g., `https://mastodon.social`).
    - `TURSO_KV_TABLE`: (Optional) The table name for key-value storage (defaults to `lachuoi_kv_store`).

## Building

To build the WebAssembly component:

```bash
just build
```

Or using cargo directly:

```bash
cargo component build --target wasm32-wasip2
```

## Running

To run the bot using `wasmtime`:

```bash
just run
```

This will:
1.  Build the component.
2.  Run it with `wasmtime`, granting necessary permissions for HTTP and environment variables.

## Project Structure

- `src/main.rs`: Core logic for fetching feeds, checking for updates, and posting to Mastodon.
- `src/db.rs`: Turso-backed key-value storage implementation using WASI HTTP.
- `src/wasi_http.rs`: Helper for making HTTP requests in a WASI environment.

## License

Copyright (c) 2026 Seungjin Kim.

This project is licensed under either of:

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT license](LICENSE-MIT)

at your option.
