pub mod channel;
pub mod feed;
pub mod item;

use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use std::{convert::TryFrom, str};
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

use channel::Channel;
use feed::Feed;
use item::Item;

pub struct Repo {
    client: Client,
}

impl Repo {
    pub async fn new(config: &str) -> Result<Self> {
        let (client, connection) = tokio_postgres::connect(config, NoTls).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Repo { client })
    }

    pub async fn get_feeds(&self, update_ts: Option<DateTime<FixedOffset>>) -> Result<Vec<Feed>> {
        match update_ts.as_ref() {
            Some(update) => {
                self.client
                    .query("SELECT * FROM feeds WHERE update_ts >= $1", &[update])
                    .await?
            }
            None => self.client.query("SELECT * FROM feeds", &[]).await?,
        }
        .iter()
        .map(Feed::try_from)
        .collect()
    }

    pub async fn update_feed(&self, feed: &Feed) -> Result<Feed> {
        let rows = self
            .client
            .query(
                "UPDATE feeds SET url=$1 WHERE id=$2 RETURNING *",
                &[&feed.url, &feed.id],
            )
            .await?;

        match rows.len() {
            1 => Ok(Feed::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating feed")),
        }
    }

    pub async fn create_feed(&self, feed_url: &str) -> Result<Feed> {
        let rows = self
            .client
            .query(
                "INSERT INTO feeds (id, url) VALUES ($1, $2) RETURNING *",
                &[&Uuid::new_v4(), &String::from(feed_url)],
            )
            .await?;

        match rows.len() {
            1 => Ok(Feed::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating feed")),
        }
    }

    pub async fn get_channel_by_title_feed_id(
        &self,
        title: &str,
        feed_id: &Uuid,
    ) -> Result<Option<Channel>> {
        let rows = self
            .client
            .query(
                "SELECT * FROM channels WHERE title=$1 AND feed_id=$2",
                &[&title, feed_id],
            )
            .await?;

        match rows.len() {
            0 => Ok(None),
            1 => Ok(Some(Channel::try_from(&rows[0])?)),
            _ => Err(anyhow::Error::msg("more than one row found")),
        }
    }

    pub async fn get_all_channels(
        &self,
        since: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Channel>> {
        match since {
            Some(s) => {
                self.client
                    .query("SELECT * FROM channels WHERE update_ts >= $1", &[&s])
                    .await?
            }
            None => self.client.query("SELECT * FROM channels", &[]).await?,
        }
        .iter()
        .map(Channel::try_from)
        .collect()
    }

    pub async fn create_channel(
        &self,
        title: &str,
        description: &str,
        image: &Option<String>,
        feed_id: &Uuid,
    ) -> Result<Channel> {
        let rows = self.client.query("INSERT INTO channels (id, title, description, image, feed_id) VALUES ($1, $2, $3, $4, $5) RETURNING *", &[&Uuid::new_v4(), &title, &description, &image, feed_id]).await?;

        match rows.len() {
            1 => Ok(Channel::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating channel")),
        }
    }

    pub async fn update_channel(&self, channel: &Channel) -> Result<Channel> {
        let rows = self.client.query("UPDATE channels SET title=$1, description=$2, image=$3, feed_id=$4 WHERE id=$5 RETURNING *", &[&channel.title, &channel.description, &channel.image, &channel.feed_id, &channel.id]).await?;

        match rows.len() {
            1 => Ok(Channel::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating channel")),
        }
    }

    pub async fn get_items_by_channel_id(
        &self,
        channel_id: &Uuid,
        since: Option<DateTime<FixedOffset>>,
    ) -> Result<Vec<Item>> {
        match since.as_ref() {
            Some(update) => {
                self.client
                    .query(
                        "SELECT * FROM items WHERE channel_id = $1 WHERE update_ts >= $2",
                        &[channel_id, update],
                    )
                    .await?
            }
            None => {
                self.client
                    .query("SELECT * FROM items WHERE channel_id = $1", &[channel_id])
                    .await?
            }
        }
        .iter()
        .map(Item::try_from)
        .collect()
    }

    pub async fn get_item_by_id(&self, id: &Uuid) -> Result<Item> {
        let rows = self
            .client
            .query("SELECT * FROM items WHERE id = $1", &[id])
            .await?;

        match rows.len() {
            0 => Err(anyhow::Error::msg("item not found")),
            1 => Ok(Item::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("more than one row found")),
        }
    }

    pub async fn get_item_by_title_date_channel_id(
        &self,
        title: &str,
        date: &DateTime<FixedOffset>,
        channel_id: &Uuid,
    ) -> Result<Option<Item>> {
        let rows = self
            .client
            .query(
                "SELECT * FROM items WHERE title=$1 AND date=$2 AND channel_id=$3",
                &[&title, date, channel_id],
            )
            .await?;

        match rows.len() {
            0 => Ok(None),
            1 => Ok(Some(Item::try_from(&rows[0])?)),
            _ => Err(anyhow::Error::msg("more than one row found")),
        }
    }

    pub async fn create_item(
        &self,
        title: &str,
        date: &DateTime<FixedOffset>,
        enclosure_type: &str,
        enclosure_url: &str,
        channel_id: &Uuid,
    ) -> Result<Item> {
        let rows = self.client.query("INSERT INTO items (id, title, date, enclosure_type, enclosure_url, channel_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *", &[&Uuid::new_v4(), &title, date, &enclosure_type, &enclosure_url, channel_id]).await?;

        match rows.len() {
            1 => Ok(Item::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating channel")),
        }
    }

    pub async fn update_item(&self, item: &Item) -> Result<Item> {
        let rows = self.client.query("UPDATE items SET title=$1, date=$2, enclosure_type=$3, enclosure_url=$4, channel_id=$5 WHERE id=$6 RETURNING *", &[&item.title, &item.date, &item.enclosure_type, &item.enclosure_url, &item.channel_id, &item.id]).await?;

        match rows.len() {
            1 => Ok(Item::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating channel")),
        }
    }
}
