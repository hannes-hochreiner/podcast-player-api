extern crate rss_json_service;
use anyhow::Result;
use log::{error, info};
use rss_feed::RssFeed;
use rss_json_service::{fetcher::*, repo::feed::Feed, repo::Repo, rss_feed, types};
use std::convert::TryFrom;
use std::{env, str};
use tokio::{
    fs,
    time::{sleep, Duration},
};

const TIMEOUT: Duration = Duration::from_secs(3);

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let connection = match (
        env::var("UPDATER_CONNECTION"),
        env::var("UPDATER_CONFIG_FILE"),
    ) {
        (Ok(conn), _) => Ok(conn),
        (_, Ok(file)) => {
            let config: types::updater_config::UpdaterConfig =
                serde_json::from_str(&fs::read_to_string(file).await?)?;

            Ok(config.db_connection)
        }
        (_, _) => {
            error!("error reading configuration");
            Err(anyhow::anyhow!("error reading configuration"))
        }
    }?;

    let repo = Repo::new(&connection).await?;

    loop {
        let feeds = repo.get_feeds(None).await?;

        for db_feed in feeds {
            let feed_url = db_feed.url.clone();

            match process_feed(&db_feed, &repo).await {
                Ok(_) => info!("successfully parsed \"{}\"", feed_url),
                Err(e) => error!("error parsing \"{}\": {}", feed_url, e),
            }
        }

        sleep(Duration::from_secs(60 * 60)).await;
    }
}

async fn process_feed(db_feed: &Feed, repo: &Repo) -> Result<()> {
    let res = request(&db_feed.url, &TIMEOUT).await?;

    if let Some(new_url) = &res.1 {
        let mut updated_feed = db_feed.clone();

        updated_feed.url = new_url.clone();

        repo.update_feed(&updated_feed).await?;
    }

    // Concatenate the body stream into a single buffer...
    let buf = hyper::body::to_bytes(res.0).await?;
    let rss_feed = RssFeed::try_from(str::from_utf8(&buf)?)?;

    for rss_channel in &rss_feed.channels {
        let db_channel = match repo
            .get_channel_by_title_feed_id(&*rss_channel.title, &db_feed.id)
            .await?
        {
            Some(mut c) => {
                let description = rss_channel.description.clone();
                let image = rss_channel.image.clone();

                if c.needs_update(&description, &image) {
                    c.description = description;
                    c.image = image;
                    repo.update_channel(&c).await?
                } else {
                    c
                }
            }
            None => {
                repo.create_channel(
                    &*rss_channel.title,
                    &*rss_channel.description,
                    &rss_channel.image,
                    &db_feed.id,
                )
                .await?
            }
        };

        for rss_item in &rss_channel.items {
            match repo
                .get_item_by_title_date_channel_id(&*rss_item.title, &rss_item.date, &db_channel.id)
                .await?
            {
                Some(mut i) => {
                    let enclosure_type = rss_item.enclosure.mime_type.clone();
                    let enclosure_url = rss_item.enclosure.url.clone();

                    if i.needs_update(&enclosure_type, &enclosure_url) {
                        i.enclosure_type = enclosure_type;
                        i.enclosure_url = enclosure_url;

                        repo.update_item(&i).await?;
                    }
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
