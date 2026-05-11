# WSJ RSS Bot - Design Document

This document outlines the architecture, components, and design decisions of the `wsj-rss-bot`.

## 🏗 Architecture Overview

The `wsj-rss-bot` is built as a **WASI P2 (WebAssembly System Interface) Component**. It is designed to be highly portable, secure, and efficient, capable of running on the **La Chuoi** distributed runtime or as a standalone binary using `wasmtime`.

### High-Level Flow
1. **Fetch Feeds**: The bot retrieves a list of WSJ RSS feeds from a remote HJSON configuration.
2. **Process RSS**: For each feed, it fetches the XML, parses items, and filters for new content.
3. **Filter & Deduplicate**:
   - **Age Filter**: Articles older than 2 hours are ignored.
   - **History Check**: Links are checked against the KV store to ensure they haven't been posted before.
4. **Post to Mastodon**: New articles are formatted and posted to the configured Mastodon instance.
5. **Update State**: The last build date for each feed and the newly posted links are saved back to the KV store.

## 🧩 Key Components

### 1. `src/main.rs` (Orchestration)
- Uses `futures::executor::block_on` to provide an async execution environment within a synchronous WASI command.
- Manages the top-level loop over multiple RSS feeds.
- Implements the 2-hour age limit and formatting logic.

### 2. `src/db.rs` (Data Persistence)
- Provides a high-level abstraction for Key-Value (KV) storage.
- **La Chuoi Mode**: Uses JSON-RPC 2.0 over HTTP (`get_key`, `set_key`) when `RPC_ENDPOINT` is present.
- **Standalone Mode**: Falls back to a local `storage.json` file when running outside a cluster.
- Supports **duplicate keys**: Retains a full history of posted links under the same key.

### 3. `src/wasi_http.rs` (Network Layer)
- A low-level, async wrapper for the `wasi:http/outgoing-handler`.
- Handles DNS resolution, TLS, and polling for HTTP responses.
- Provides detailed `ErrorCode` reporting for debugging network issues.

## 💾 Data Schema (KV Store)

| Key | Value | Purpose |
| :--- | :--- | :--- |
| `posted link` | `<URL>` | A list of all previously published article links. |
| `<feedName>.last_build_date` | `ISO 8601` | The timestamp of the last successful run for a specific feed. |

## 🛡 Design Decisions & Safety

- **Environment Awareness**: If `ENVIRONMENT` is set to `development`, `dry_run` is forced to `true` to prevent accidental Mastodon posts during testing.
- **Browser-like Headers**: To avoid being blocked by RSS servers (which often reject generic `WASI-HTTP` or empty `User-Agent` strings), the bot sends a full suite of browser-like headers:
    - `User-Agent`: `Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0`
    - `Accept`: `text/html,application/xhtml+xml,application/xml;...`
    - `Accept-Language`: `en-US,en;q=0.9`
- **List-based Deduplication**: Leveraging the La Chuoi KV store's ability to store duplicate values for a single key, the bot maintains the entire link history under the `posted link` key. This avoids the overhead of managing thousands of individual keys.
- **2-Hour Limit**: A hard limit ensures that even if the state is reset, the bot only considers articles from the last 2 hours, preventing historical "spamming".
- **Async Synchronization**: Although WASI P2 commands are technically synchronous, the bot uses an async-over-sync architecture (via `futures::executor::block_on`) to match the pattern of other bots in the ecosystem (e.g., `newspenguin-rss-bot`), allowing for cleaner IO handling.

## 🚀 Environment Variables

- `APP_ID`: Numeric task identifier.
- `LACHUOI_TOKEN`: Authentication token for JSON-RPC.
- `RPC_ENDPOINT`: URI for the La Chuoi master node.
- `MSTD_ACCESS_TOKEN`: Mastodon API Bearer token.
- `MSTD_API_URI`: Mastodon instance endpoint.
- `ENVIRONMENT`: `production` or `development`.
- `DRY_RUN`: If `true`, skips actual Mastodon posts and DB updates.
