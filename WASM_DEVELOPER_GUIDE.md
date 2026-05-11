# La Chuoi - WASM Developer Guide

This guide explains how to build and run WebAssembly (WASM) tasks on the La Chuoi distributed runtime.

## 🚀 Getting Started

La Chuoi supports both **WASI Preview 1** modules and **WASI Preview 2** (Component Model) binaries.

### Environment Variables

Every WASM task is provided with the following system environment variables:

- `APP_ID`: The unique numeric ID of the task.
- `LACHUOI_TOKEN`: A one-time authentication token required for system RPC calls.
- `ENVIRONMENT`: The current system mode (`production` or `development`). Default is `production`.
- `RPC_ENDPOINT`: The HTTP URI of the JSON-RPC service (e.g., `https://lachuoi.example.com/api/rpc`).

## 🛠 System Interaction (JSON-RPC)

WASM tasks can interact with the La Chuoi host (Master node) by sending standard **JSON-RPC 2.0** requests via HTTP POST to the `RPC_ENDPOINT`.

### Request Format

All requests must be POSTed as JSON and include the `token` and `task_id` in the `params`.

### KV Store Access

Each task has access to a persistent Key-Value store. **Note**: Keys can be duplicated; if multiple values exist for a key, `get_key` will return them as a list.

#### 1. Set a Value (`set_key`)

```json
{
  "jsonrpc": "2.0",
  "method": "set_key",
  "params": {
    "token": "YOUR_LACHUOI_TOKEN",
    "task_id": YOUR_APP_ID,
    "key": "last_run_timestamp",
    "value": "2026-05-11T12:00:00Z"
  },
  "id": 1
}
```

#### 2. Get Values (`get_key`)

```json
{
  "jsonrpc": "2.0",
  "method": "get_key",
  "params": {
    "token": "YOUR_LACHUOI_TOKEN",
    "task_id": YOUR_APP_ID,
    "key": "last_run_timestamp"
  },
  "id": 2
}
```

The system will log the response back to your task's log stream in the format:
`[rpc] Response: {"jsonrpc":"2.0","result":["2026-05-11T12:00:00Z"],"id":2}`

## 📦 Building your WASM

### Using Rust

We recommend using the `wasm32-wasip1` or `wasm32-wasip2` targets.

```bash
# Add the target
rustup target add wasm32-wasip1

# Build your task
cargo build --target wasm32-wasip1 --release
```

### Example Task (Rust)

This example demonstrates how to use the duplicate key feature for deduplication.

```rust
use std::env;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_id = env::var("APP_ID")?.parse::<i64>()?;
    let token = env::var("LACHUOI_TOKEN")?;
    let endpoint = env::var("RPC_ENDPOINT")?;

    let current_link = "https://example.com/new-post";

    // 1. Check history
    let get_request = json!({
        "jsonrpc": "2.0",
        "method": "get_key",
        "params": { "token": &token, "task_id": app_id, "key": "posted_links" },
        "id": 1
    });

    let client = reqwest::Client::new();
    let resp = client.post(&endpoint).json(&get_request).send().await?;
    let body: Value = resp.json().await?;
    
    let history = body["result"].as_array();
    let already_posted = history.map_or(false, |links| {
        links.iter().any(|l| l.as_str() == Some(current_link))
    });

    if !already_posted {
        println!("Posting new link: {}", current_link);
        
        // 2. Add to history
        let set_request = json!({
            "jsonrpc": "2.0",
            "method": "set_key",
            "params": {
                "token": token,
                "task_id": app_id,
                "key": "posted_links",
                "value": current_link
            },
            "id": 2
        });
        client.post(&endpoint).json(&set_request).send().await?;
    }

    Ok(())
}
```

## ⚖️ Runtime Constraints

- **Networking**: Components (Preview 2) have outbound HTTP access enabled by default.
- **Filesystem**: Tasks run with a private, temporary virtual filesystem unless specific mappings are provided in `cron.toml`.
- **Resources**: CPU and Memory usage are monitored and reported to the Master dashboard.

## 🛡 Security

- **Sandboxing**: Tasks run in a Wasmtime sandbox, isolated from the host and other tasks.
- **Integrity**: Always provide a `sha256` checksum in your `cron.toml` to ensure the runtime only executes verified binaries.
