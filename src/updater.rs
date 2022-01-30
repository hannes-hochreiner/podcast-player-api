use crate::{fetcher::request, repo::Repo, rss_feed::RssFeed};
use anyhow::Result;
use chrono::Utc;
use hyper::{Body, Response};
use log::{error, info, warn};
use podcast_player_common::{FeedUrl, FeedVal};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

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
                warn!("error creating repo; waiting 10s before retry: {}", e);
                sleep(Duration::from_secs(10)).await;
                Repo::new(&self.connection).await
            }
        }?;

        let feeds = repo.get_objects::<FeedVal>(None).await?;

        for feed in feeds {
            let title = feed.title.clone();
            match process_feed(&feed, &repo).await {
                Ok(_) => info!("successfully parsed \"{}\"", title),
                Err(e) => error!("error parsing \"{}\": {}", title, e),
            }
        }

        Ok(())
    }
}

async fn process_feed(db_feed: &FeedVal, repo: &Repo) -> Result<()> {
    let res = get_feed_response(db_feed, repo).await?;

    // Concatenate the body stream into a single buffer...
    let buf = hyper::body::to_bytes(res).await?;
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
                        rss_item.enclosure.length,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

async fn get_feed_response(db_feed: &FeedVal, repo: &Repo) -> Result<Response<Body>> {
    let mut feed_urls = repo.get_urls_by_feed_id(&db_feed.id).await?;

    feed_urls.sort();

    let mut feed_url_ids = feed_urls.iter().map(|fu| fu.id).collect::<Vec<Uuid>>();

    while feed_url_ids.len() > 0 {
        // find first url, which has not been tried
        match feed_urls.iter().find(|&fu| feed_url_ids.contains(&fu.id)) {
            Some(feed_url) => {
                let res = request(&feed_url.url, &TIMEOUT).await?;

                for (res_url, res_status) in res.1 {
                    // check whether the url is in the repo
                    match feed_urls.iter().find(|&f| f.url == res_url) {
                        Some(fu) => {
                            // remove url from map
                            feed_url_ids.retain(|&id| id != fu.id);
                            // update if url is in repo
                            let mut new_fu = fu.clone();

                            new_fu.status = Some(res_status);
                            new_fu.update_ts = Utc::now().into();
                            repo.update_feed_url(&new_fu).await?;
                        }
                        None => {
                            // add if not
                            repo.create_feed_url(&FeedUrl {
                                feed_id: db_feed.id,
                                id: Uuid::new_v4(),
                                manual: false,
                                status: Some(res_status),
                                synced: false,
                                update_ts: Utc::now().into(),
                                url: res_url,
                            })
                            .await?;
                        }
                    }
                }

                if let Some(resp) = res.0 {
                    return Ok(resp);
                }
            }
            None => {
                // should not happen, as long as there is an id in the id vector, it should also be in the url vector
                return Err(anyhow::anyhow!(
                    "unexpected result in processing the urls of feed \"{}\"",
                    db_feed.title
                ));
            }
        }
    }

    Err(anyhow::anyhow!(
        "none of the urls for the feed was retrievable"
    ))
}
