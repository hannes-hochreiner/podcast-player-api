extern crate rss_json_service;
use anyhow::{Context, Result};
use hyper::Client as HttpClient;
use hyper::client::HttpConnector;
use hyper_tls::HttpsConnector;
use log::{error, info};
use rss_feed::RssFeed;
use rss_json_service::repo::feed::Feed;
use rss_json_service::repo::Repo;
use rss_json_service::rss_feed;
use std::convert::TryFrom;
use std::{env, str};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let repo = Repo::new(&*env::var("UPDATER_CONNECTION")?).await?;
    let feeds = repo.get_feeds().await?;

    for db_feed in feeds {
        let feed_url = db_feed.url.clone();

        match process_feed(db_feed, &repo).await {
            Ok(_) => info!("successfully parsed \"{}\"", feed_url),
            Err(e) => error!("error parsing \"{}\": {}", feed_url, e),
        }
    }

    Ok(())
}

async fn process_feed(db_feed: Feed, repo: &Repo) -> Result<()> {
    let uri: hyper::Uri = db_feed.url.parse()?;
    let res = match uri.scheme_str() {
        Some(s) => {
            match s {
                "http" => 
                    HttpClient::builder().build::<_, hyper::Body>(HttpConnector::new()).get(uri).await.context("request failed"),
                "https" => 
                    HttpClient::builder().build::<_, hyper::Body>(HttpsConnector::new()).get(uri).await.context("request failed"),
                _ => Err(anyhow::anyhow!("no connector available for scheme \"{}\"", s))
            }
        },
        None => Err(anyhow::anyhow!("scheme not recognized"))
    }?;

    // let http_client = HttpClient::builder().build::<_, hyper::Body>(http_connector);
    // TODO: determine whether the url is http or https and choose the client accordingly
    // let client = HttpClient::new();
    // let res = http_client.get(uri).await?;
    // TODO: If the feed moved permanently, update the feed url

    // Concatenate the body stream into a single buffer...
    let buf = hyper::body::to_bytes(res).await?;
    let rss_feed = RssFeed::try_from(str::from_utf8(&buf)?)?;

    for rss_channel in &rss_feed.channels {
        let db_channel;

        match repo
            .get_channel_by_title_feed_id(&*rss_channel.title, &db_feed.id)
            .await?
        {
            Some(c) => db_channel = repo.update_channel(&c).await?,
            None => {
                db_channel = repo
                    .create_channel(
                        &*rss_channel.title,
                        &*rss_channel.description,
                        &rss_channel.image,
                        &db_feed.id,
                    )
                    .await?
            }
        }

        for rss_item in &rss_channel.items {
            match repo
                .get_item_by_title_date_channel_id(&*rss_item.title, &rss_item.date, &db_channel.id)
                .await?
            {
                Some(i) => {
                    repo.update_item(&i).await?;
                }
                None => {
                    repo.create_item(
                        &*rss_item.title,
                        &rss_item.date,
                        &*rss_item.enclosure.mime_type,
                        &*rss_item.enclosure.url,
                        &db_channel.id,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}
