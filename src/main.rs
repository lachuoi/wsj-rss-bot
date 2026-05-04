// Copyright 2026 Seungjin Kim
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod db;
mod wasi_http;

use anyhow::Result;
use chrono::{self, DateTime, NaiveDateTime, Utc};
use convert_case::{Case, Casing};
use rss::{Channel, Item};
use serde_json::Value;
use std::env;
use std::str::{self};
use wasi as bindings;
use wasi_http::http_request;

const DB_KEY_PREFIX: &str = "wsj-rss";

fn main() -> Result<()> {
    println!("WSJ RSS starting");

    let feeds_url = "https://raw.githubusercontent.com/lachuoi/lachuoi/refs/heads/legacy-gpl-version/assets/wsj-news-feeds.hjson";
    let response_body = match http_request(
        bindings::http::types::Method::Get,
        feeds_url,
        vec![],
        None,
    ) {
        Ok(body) => body,
        Err(e) => {
            eprintln!("Failed to fetch feeds: {:?}", e);
            return Ok(());
        }
    };
    let response_str = match str::from_utf8(&response_body) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse feeds body: {:?}", e);
            return Ok(());
        }
    };
    let wsj_rss_feeds: Value = match serde_hjson::from_str(response_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse HJSON: {:?}", e);
            return Ok(());
        }
    };

    if let Some(feeds) = wsj_rss_feeds.as_array() {
        for feed in feeds {
            if let (Some(name), Some(url)) = (
                feed.get("name").and_then(Value::as_str),
                feed.get("url").and_then(Value::as_str),
            ) {
                if let Err(e) = rss_eater(name.to_string(), url.to_string()) {
                    eprintln!("Error processing feed {}: {:?}", name, e);
                }
            }
        }
    }

    println!("WSJ RSS finished");
    Ok(())
}

fn rss_eater(name: String, url: String) -> Result<()> {
    let channel = get_rss(url)?;

    let rss_last_build_date = match channel.last_build_date() {
        Some(date_str) => {
            parse_rss_date(date_str).expect("WSJ Failed to parse date")
        }
        None => Utc::now(),
    };

    let recorded_last_build_date =
        last_build_date(&name, rss_last_build_date)?;

    if rss_last_build_date > recorded_last_build_date {
        let new_items =
            get_new_items(&channel, recorded_last_build_date)?;
        post_to_mastodon(&name, new_items)?;
        update_last_build_date(&name, rss_last_build_date)?;
    } else {
        update_last_build_date(&name, rss_last_build_date)?;
    }

    Ok(())
}

fn get_rss(rss_uri: String) -> Result<Channel> {
    let body = http_request(
        bindings::http::types::Method::Get,
        &rss_uri,
        vec![],
        None,
    )?;
    let channel = Channel::read_from(&body[..])?;
    Ok(channel)
}

fn parse_rss_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();

    // Try RFC 2822 (common in RSS)
    if let Ok(dt) = DateTime::parse_from_rfc2822(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Try RFC 3339
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    None
}

fn last_build_date(
    name: &String,
    current_rss_dt: DateTime<Utc>,
) -> Result<DateTime<Utc>> {
    let camel_name = name.to_case(Case::Camel);
    let db_key = format!("{}.{}.last_build_date", DB_KEY_PREFIX, camel_name);

    match db::get_kv(&db_key)? {
        Some(stored_val) => {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&stored_val) {
                Ok(dt.with_timezone(&Utc))
            } else {
                // Fallback to old format if necessary
                if let Ok(ndt) = NaiveDateTime::parse_from_str(
                    &stored_val,
                    "%Y-%m-%d %H:%M:%S",
                ) {
                    Ok(DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc))
                } else {
                    Ok(current_rss_dt)
                }
            }
        }
        None => {
            let now = Utc::now();
            db::set_kv(&db_key, &now.to_rfc3339())?;
            Ok(now)
        }
    }
}

fn update_last_build_date(name: &String, d: DateTime<Utc>) -> Result<()> {
    let camel_name = name.to_case(Case::Camel);
    let db_key = format!("{}.{}.last_build_date", DB_KEY_PREFIX, camel_name);
    db::set_kv(&db_key, &d.to_rfc3339())?;
    Ok(())
}

fn get_new_items(
    channel: &Channel,
    recorded_last_build_date: DateTime<Utc>,
) -> Result<Vec<Item>> {
    let mut new_items: Vec<Item> = Vec::new();
    for item in channel.items() {
        if let Some(pub_date_str) = item.pub_date() {
            if let Some(item_pub_date) = parse_rss_date(pub_date_str) {
                if recorded_last_build_date < item_pub_date {
                    new_items.push(item.clone());
                }
            }
        }
    }
    new_items.reverse();
    Ok(new_items)
}

fn post_to_mastodon(name: &String, msgs: Vec<Item>) -> Result<()> {
    let mstd_api_uri = env::var("MSTD_API_URI").expect("MSTD_API_URI not set");
    let mstd_access_token =
        env::var("MSTD_ACCESS_TOKEN").expect("MSTD_ACCESS_TOKEN not set");

    if msgs.is_empty() {
        println!("WSJ {} - Nothing to publish", name);
        return Ok(());
    }

    for item in msgs {
        let description_html = item.description.clone().unwrap_or_default();
        let description = html2text::config::plain()
            .string_from_read(description_html.as_bytes(), 1000)
            .unwrap_or_default();
        let description = description.trim();

        let msg: String = format!(
            "[{}] {}\n{}\n{} #WSJ\n({})",
            name,
            item.title.clone().unwrap_or_default(),
            description,
            item.link.clone().unwrap_or_default(),
            item.pub_date.clone().unwrap_or_default()
        )
        .trim()
        .to_string();

        let body_json = serde_json::json!({
            "status": msg,
            "visibility": "public"
        });
        let body = serde_json::to_vec(&body_json)?;

        let headers = vec![
            (
                "Authorization".to_string(),
                format!("Bearer {}", mstd_access_token).into_bytes(),
            ),
            (
                "Content-Type".to_string(),
                "application/json".to_string().into_bytes(),
            ),
        ];

        let url =
            format!("{}/api/v1/statuses", mstd_api_uri.trim_end_matches('/'));
        http_request(
            bindings::http::types::Method::Post,
            &url,
            headers,
            Some(body),
        )?;

        println!("WSJ {} published: {}", name, item.title.unwrap_or_default());
    }

    Ok(())
}
