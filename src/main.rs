mod scratchpad;

use chrono;
use rss::Channel;
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::env;
use tokio;

use std::string::ToString;
use std::thread::yield_now;

use reqwest::header::AUTHORIZATION;

use std::time::{Duration, Instant};

use std::{thread, time};
use log::{debug, error, info, trace, warn};
use log4rs;
use serde_yaml;
		
const HOWOFTEN: i64 = 10;

async fn feed(url: String) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

async fn toot(msg: String) {
    let ACCESS_TOKEN = env::var("MSTDN_ACCESS_TOKEN")
        .expect("You must set the MSTDN_ACCESS_TOKEN environment var!");

    let res = reqwest::Client::new()
        .post("https://mstd.seungjin.net/api/v1/statuses")
        .header(
            AUTHORIZATION,
            format!("Bearer {}", ACCESS_TOKEN),
        )
        .form(&[("status", msg)])
        .send()
        .await;

    match res {
        Ok(r) => info!("Msg updated with code {:?}\n", r.status()),
        Err(e) => error!("Error on posting: {}", e),
    }
}

async fn showme(c: Channel) {
    for i in c.items {
        tokio::spawn(async move {
            if ! scratchpad::new_title(i.clone().title.unwrap())
                .await
                .unwrap()
            {
                scratchpad::write_title(i.clone().title.unwrap()).await;
                let msg: String = format!(
                    "{}:\n{}\n{}\n({})",
                    i.title.unwrap(),
                    i.description.unwrap(),
                    i.link.unwrap(),
                    i.pub_date.unwrap()
                );
                info!("New article: {}", msg);
                toot(msg).await;
            }
        });
    }
}

async fn magic() {
    let start = Instant::now();
    let addresses: Vec<String> = vec![
        "https://feeds.a.dj.com/rss/RSSOpinion.xml".to_string(),
        "https://feeds.a.dj.com/rss/RSSWorldNews.xml".to_string(),
        "https://feeds.a.dj.com/rss/WSJcomUSBusiness.xml".to_string(),
        "https://feeds.a.dj.com/rss/RSSMarketsMain.xml".to_string(),
        "https://feeds.a.dj.com/rss/RSSWSJD.xml".to_string(),
        "https://feeds.a.dj.com/rss/RSSLifestyle.xml".to_string(),
    ];

    for addr in addresses {
	let a = addr.clone();
        tokio::spawn(async move {
            let a = feed(addr.to_string()).await.unwrap();
            showme(a).await;
        });
        info!("{}", a);
	let a_min = Duration::new(60, 0);
        thread::sleep(a_min);
    }
}

#[tokio::main]
async fn main() {
    let config_str = include_str!("log4rs.yaml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();

    let mut interval_timer =
        tokio::time::interval(chrono::Duration::minutes(HOWOFTEN).to_std().unwrap());
    loop {
        // Wait for the next interval tick
        interval_timer.tick().await;
        info!("Starting a job");
        tokio::spawn(async {
            magic().await;
        }); // For async task
        //tokio::task::spawn_blocking(|| do_my_task()); // For blocking task
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_magic() {
        let start = Instant::now();
        magic().await;
        let duration = start.elapsed();
        debug!("Time elapsed in expensive_function() is: {:?}", duration);
    }

}




