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
use rss::{Channel, Item};
use serde_json::Value;
use std::env;
use std::str::{self};
use wasi as bindings;
use wasi_http::http_request;
use convert_case::{Case, Casing};

async fn fetch_rss(url: String) -> Result<Channel> {
    let user_agent = env::var("WSJ_RSS_USER_AGENT").unwrap_or_else(|_| {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0".to_string()
    });

    let headers = vec![
        (
            "User-Agent".to_string(),
            user_agent.into_bytes(),
        ),
        (
            "Accept".to_string(),
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".to_string().into_bytes(),
        ),
        (
            "Accept-Language".to_string(),
            "en-US,en;q=0.9".to_string().into_bytes(),
        ),
    ];
    let content =
        http_request(bindings::http::types::Method::Get, &url, headers, None)
            .await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

fn parse_date(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let s = s.trim();

    // 1. Try RFC 3339
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&chrono::Utc));
    }

    // 2. Try RFC 2822
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(s) {
        return Some(dt.with_timezone(&chrono::Utc));
    }

    // 3. Try naive format
    if let Ok(ndt) =
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
    {
        use chrono::TimeZone;
        return Some(chrono::Utc.from_utc_datetime(&ndt));
    }

    None
}

async fn toot(msg: String, dry_run: bool) -> Result<()> {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
    if environment == "development" {
        println!("Development mode: Skipping Mastodon post.");
        println!("Message would have been:\n{}", msg);
        return Ok(());
    }

    if dry_run {
        println!("Dry run: Would post message:\n{}", msg);
        return Ok(());
    }

    let access_token = env::var("MSTD_ACCESS_TOKEN").expect(
        "MSTD_ACCESS_TOKEN not set",
    );
    let access_token = access_token.trim();
    let access_url = env::var("MSTD_API_URI")
        .unwrap_or_else(|_| "https://mstd.seungjin.net".to_string());
    let access_url = access_url.trim().trim_end_matches('/');

    let body =
        format!("status={}&visibility=public", urlencoding::encode(&msg));

    let headers = vec![
        (
            "Authorization".to_string(),
            format!("Bearer {}", access_token).into_bytes(),
        ),
        (
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string().into_bytes(),
        ),
        (
            "User-Agent".to_string(),
            "wsj-rss-bot/0.1.0".to_string().into_bytes(),
        ),
    ];

    let url = format!("{}/api/v1/statuses", access_url);

    http_request(
        bindings::http::types::Method::Post,
        &url,
        headers,
        Some(body.into_bytes()),
    )
    .await?;

    println!("Message posted!");
    Ok(())
}

async fn process_items(
    name: &str,
    app_id: i64,
    items: Vec<Item>,
    saved_date: Option<chrono::DateTime<chrono::Utc>>,
    dry_run: bool,
) -> Result<()> {
    let now = chrono::Utc::now();
    let limit = now - chrono::Duration::hours(2);

    let mut items = items;
    items.reverse();

    for i in items {
        let link = i.link.clone().unwrap_or_default();
        if link.is_empty() {
            continue;
        }

        let pub_date = i.pub_date.as_ref().and_then(|s| parse_date(s));

        // 1. Age check (2 hours)
        if let Some(pd) = pub_date {
            if pd < limit {
                continue;
            }

            if let Some(sd) = saved_date {
                if pd <= sd {
                    continue;
                }
            }
        } else {
            continue;
        }

        // 2. Duplicate check
        if db::check_link_published(app_id, &link).await? {
            continue;
        }

        let title = i.title.clone().unwrap_or_default();
        let description_html = i.description.clone().unwrap_or_default();
        let description = html2text::config::plain()
            .string_from_read(description_html.as_bytes(), 1000)
            .unwrap_or_default();
        let description = description.trim();

        let msg: String = format!(
            "[{}] {}\n{}\n{}\n({})",
            name,
            title,
            description,
            link,
            i.pub_date.unwrap_or_default()
        );

        println!("Posting: {}", title);
        toot(msg, dry_run).await?;

        if !dry_run {
            db::add_posted_link(app_id, &link).await?;
        }
    }
    Ok(())
}

async fn magic(name: String, url: String, dry_run: bool) -> Result<()> {
    let app_id = env::var("APP_ID")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<i64>()
        .unwrap_or(0);

    println!("Processing feed: {} ({})", name, url);
    let channel = fetch_rss(url).await?;

    let camel_name = name.to_case(Case::Camel);
    let kv_key = format!("{}.last_build_date", camel_name);
    
    let saved_date_str = db::get_kv(app_id, &kv_key).await?;
    let saved_date = saved_date_str.as_ref().and_then(|s| parse_date(s));

    process_items(&name, app_id, channel.items, saved_date, dry_run).await?;

    if !dry_run {
        let now = chrono::Utc::now().to_rfc3339();
        db::set_kv(app_id, &kv_key, &now).await?;

        if let Err(e) = db::delete_old_posted_messages(app_id).await {
            eprintln!("Warning: Failed to clean up old posted links: {:?}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
    let dry_run = args.iter().any(|arg| arg == "--dryrun") 
        || env::var("DRY_RUN").map(|v| v == "true").unwrap_or(false)
        || environment == "development";

    println!("WSJ RSS starting");
    if dry_run {
        println!("Running in DRY RUN mode");
    }

    futures::executor::block_on(async {
        let feeds_url = "https://raw.githubusercontent.com/lachuoi/lachuoi/refs/heads/legacy-gpl-version/assets/wsj-news-feeds.hjson";
        let response_body = match http_request(
            bindings::http::types::Method::Get,
            feeds_url,
            vec![("User-Agent".to_string(), "wsj-rss-bot/0.1.0".to_string().into_bytes())],
            None,
        ).await {
            Ok(body) => body,
            Err(e) => {
                eprintln!("Failed to fetch feeds: {:?}", e);
                return;
            }
        };

        let response_str = match str::from_utf8(&response_body) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to parse feeds body: {:?}", e);
                return;
            }
        };

        let wsj_rss_feeds: Value = match serde_hjson::from_str(response_str) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse HJSON: {:?}", e);
                return;
            }
        };

        if let Some(feeds) = wsj_rss_feeds.as_array() {
            for feed in feeds {
                if let (Some(name), Some(url)) = (
                    feed.get("name").and_then(Value::as_str),
                    feed.get("url").and_then(Value::as_str),
                ) {
                    if let Err(e) = magic(name.to_string(), url.to_string(), dry_run).await {
                        eprintln!("Error processing feed {}: {:?}", name, e);
                    }
                }
            }
        }
    });

    println!("WSJ RSS finished");
    Ok(())
}
