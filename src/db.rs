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
use wasi as bindings;

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

pub fn get_kv(key: &str) -> Result<Vec<String>> {
    let endpoint = env::var("RPC_ENDPOINT").map_err(|_| anyhow::anyhow!("RPC_ENDPOINT not set"))?;
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
        &endpoint,
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

pub fn set_kv(key: &str, value: &str) -> Result<()> {
    let endpoint = env::var("RPC_ENDPOINT").map_err(|_| anyhow::anyhow!("RPC_ENDPOINT not set"))?;
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
        &endpoint,
        headers,
        Some(body),
    )?;

    let resp: JsonRpcResponse = serde_json::from_slice(&resp_body)?;
    
    if let Some(error) = resp.error {
        return Err(anyhow::anyhow!("JSON-RPC error: {}", error));
    }

    Ok(())
}
