use crate::{fetcher::request, repo::Repo, rss_feed::RssFeed};
use anyhow::Result;
use log::{error, info, warn};
use podcast_player_common::feed_val::FeedVal;
use tokio::time::{sleep, Duration};

const TIMEOUT: Duration = Duration::from_secs(3);

pub struct Updater {
    connection: String,
}

impl Updater {
    pub fn new(connection: &str) -> Self {
        Self {
            connection: connection.into(),
        }
    }

    pub async fn update_loop(&self) {
        loop {
            match &self.process_feeds().await {
                Err(e) => error!("error processing feeds: {}", e),
                _ => {}
            }

            sleep(Duration::from_secs(60 * 60)).await;
        }
    }

    async fn process_feeds(&self) -> Result<()> {
        let repo = match Repo::new(&self.connection).await {
            Ok(rep) => Ok(rep),
            Err(e) => {
                warn!("error creating repo; waiting 3s before retry: {}", e);
                sleep(Duration::from_secs(3)).await;
                Repo::new(&self.connection).await
            }
        }?;

        let feeds = repo.get_feeds(None).await?;

        for db_feed in feeds {
            let feed_url = db_feed.url.clone();

            match process_feed(&db_feed, &repo).await {
                Ok(_) => info!("successfully parsed \"{}\"", feed_url),
                Err(e) => error!("error parsing \"{}\": {}", feed_url, e),
            }
        }

        Ok(())
    }
}

async fn process_feed(db_feed: &FeedVal, repo: &Repo) -> Result<()> {
    let res = request(&db_feed.url, &TIMEOUT).await?;

    if let Some(new_url) = &res.1 {
        let mut updated_feed = db_feed.clone();

        updated_feed.url = new_url.clone();

        repo.update_feed(&updated_feed).await?;
    }

    // Concatenate the body stream into a single buffer...
    let buf = hyper::body::to_bytes(res.0).await?;
    let rss_feed = RssFeed::try_from(std::str::from_utf8(&buf)?)?;

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
