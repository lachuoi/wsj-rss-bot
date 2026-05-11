// Copyright 2026 Seungjin Kim
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::wasi_http::http_request;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::collections::HashMap;
use wasi as bindings;

const LOCAL_KV_FILE: &str = "local_kv.json";

#[derive(Serialize)]
struct JsonRpcRequest<P> {
    jsonrpc: String,
    method: String,
    params: P,
    id: i64,
}

#[derive(Serialize)]
struct KvSetParams {
    token: String,
    task_id: i64,
    key: String,
    value: String,
}

#[derive(Serialize)]
struct KvGetParams {
    token: String,
    task_id: i64,
    key: String,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

fn load_local_db() -> Result<HashMap<String, Vec<String>>> {
    match fs::read_to_string(LOCAL_KV_FILE) {
        Ok(contents) => {
            let db: HashMap<String, Vec<String>> = serde_json::from_str(&contents)?;
            Ok(db)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(anyhow::anyhow!("Failed to read local DB: {}", e)),
    }
}

fn save_local_db(db: &HashMap<String, Vec<String>>) -> Result<()> {
    let contents = serde_json::to_string_pretty(db)?;
    fs::write(LOCAL_KV_FILE, contents)?;
    Ok(())
}

fn get_kv_local(key: &str) -> Result<Vec<String>> {
    let db = load_local_db()?;
    Ok(db.get(key).cloned().unwrap_or_default())
}

fn set_kv_local(key: &str, value: &str) -> Result<()> {
    let mut db = load_local_db()?;
    db.entry(key.to_string()).or_insert_with(Vec::new).push(value.to_string());
    save_local_db(&db)?;
    Ok(())
}

fn get_kv_rpc(key: &str, endpoint: &str) -> Result<Vec<String>> {
    let token = env::var("LACHUOI_TOKEN").map_err(|_| anyhow::anyhow!("LACHUOI_TOKEN not set"))?;
    let task_id = env::var("APP_ID")
        .map_err(|_| anyhow::anyhow!("APP_ID not set"))?
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("APP_ID must be a number"))?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "get_key".to_string(),
        params: KvGetParams {
            token,
            task_id,
            key: key.to_string(),
        },
        id: 1,
    };

    let body = serde_json::to_vec(&request)?;
    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string().into_bytes()),
    ];

    let resp_body = http_request(
        bindings::http::types::Method::Post,
        endpoint,
        headers,
        Some(body),
    )?;

    let resp: JsonRpcResponse = serde_json::from_slice(&resp_body)?;
    
    if let Some(error) = resp.error {
        return Err(anyhow::anyhow!("JSON-RPC error: {}", error));
    }

    match resp.result {
        Some(serde_json::Value::Array(arr)) => {
            let mut values = Vec::new();
            for val in arr {
                if let Some(s) = val.as_str() {
                    values.push(s.to_string());
                } else {
                    values.push(val.to_string().trim_matches('"').to_string());
                }
            }
            Ok(values)
        }
        Some(serde_json::Value::String(s)) => Ok(vec![s]),
        Some(serde_json::Value::Null) => Ok(vec![]),
        Some(v) if v.is_null() => Ok(vec![]),
        Some(v) => Ok(vec![v.to_string().trim_matches('"').to_string()]),
        None => Ok(vec![]),
    }
}

fn set_kv_rpc(key: &str, value: &str, endpoint: &str) -> Result<()> {
    let token = env::var("LACHUOI_TOKEN").map_err(|_| anyhow::anyhow!("LACHUOI_TOKEN not set"))?;
    let task_id = env::var("APP_ID")
        .map_err(|_| anyhow::anyhow!("APP_ID not set"))?
        .parse::<i64>()
        .map_err(|_| anyhow::anyhow!("APP_ID must be a number"))?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: "set_key".to_string(),
        params: KvSetParams {
            token,
            task_id,
            key: key.to_string(),
            value: value.to_string(),
        },
        id: 1,
    };

    let body = serde_json::to_vec(&request)?;
    let headers = vec![
        ("Content-Type".to_string(), "application/json".to_string().into_bytes()),
    ];

    let resp_body = http_request(
        bindings::http::types::Method::Post,
        endpoint,
        headers,
        Some(body),
    )?;

    let resp: JsonRpcResponse = serde_json::from_slice(&resp_body)?;
    
    if let Some(error) = resp.error {
        return Err(anyhow::anyhow!("JSON-RPC error: {}", error));
    }

    Ok(())
}

pub fn get_kv(key: &str) -> Result<Vec<String>> {
    if let Ok(endpoint) = env::var("RPC_ENDPOINT") {
        if !endpoint.is_empty() {
            return get_kv_rpc(key, &endpoint);
        }
    }
    
    // Standalone fallback
    get_kv_local(key)
}

pub fn set_kv(key: &str, value: &str) -> Result<()> {
    if let Ok(endpoint) = env::var("RPC_ENDPOINT") {
        if !endpoint.is_empty() {
            return set_kv_rpc(key, value, &endpoint);
        }
    }
    
    // Standalone fallback
    set_kv_local(key, value)
}
