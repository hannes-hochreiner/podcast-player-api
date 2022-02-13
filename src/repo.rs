use anyhow::Result;
use bb8_postgres::{bb8::Pool, PostgresConnectionManager};
use chrono::{DateTime, FixedOffset};
use podcast_player_common::{channel_val::ChannelVal, item_val::ItemVal, FeedUrl};
use std::{convert::TryFrom, str};
use tokio_postgres::NoTls;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Repo {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl Repo {
    pub async fn new(config: &str) -> Result<Self> {
        let manager = PostgresConnectionManager::new(config.parse()?, NoTls);
        let pool = Pool::builder().max_size(15).build(manager).await?;

        Ok(Repo { pool })
    }

    pub async fn get_objects<T>(&self, update_ts: Option<&str>) -> Result<Vec<T>>
    where
        T: for<'a> std::convert::TryFrom<&'a tokio_postgres::Row> + podcast_player_common::DbInfo,
        Result<Vec<T>, anyhow::Error>:
            for<'a> FromIterator<Result<T, <T as TryFrom<&'a tokio_postgres::Row>>::Error>>,
    {
        match update_ts
            .map(DateTime::parse_from_rfc3339)
            .transpose()?
            .as_ref()
        {
            Some(update) => {
                self.pool
                    .get()
                    .await?
                    .query(
                        &*format!("SELECT * FROM {} WHERE update_ts > $1", T::table_name()),
                        &[update],
                    )
                    .await?
            }
            None => {
                self.pool
                    .get()
                    .await?
                    .query(&*format!("SELECT * FROM {}", T::table_name()), &[])
                    .await?
            }
        }
        .iter()
        .map(T::try_from)
        .collect()
    }

    pub async fn update_feed_url(&self, feed_url: &FeedUrl) -> Result<FeedUrl> {
        let rows = self
            .pool.get().await?
            .query(
                "UPDATE feed_url SET feed_id=$1, url=$2, status=$3, manual=$4, synced=$5, update_ts=$6 WHERE id=$7 RETURNING *",
                &[&feed_url.feed_id, &feed_url.url, &feed_url.status, &feed_url.manual, &feed_url.synced, &feed_url.update_ts, &feed_url.id],
            ).await?;

        match rows.len() {
            1 => Ok(FeedUrl::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating feed url")),
        }
    }

    pub async fn create_feed_url(&self, feed_url: &FeedUrl) -> Result<FeedUrl> {
        let rows = self
            .pool.get().await?
            .query(
                "INSERT INTO feed_url (id, feed_id, url, status, manual, synced, update_ts) VALUES ($7, $1, $2, $3, $4, $5, $6) RETURNING *",
                &[&feed_url.feed_id, &feed_url.url, &feed_url.status, &feed_url.manual, &feed_url.synced, &feed_url.update_ts, &feed_url.id],
            ).await?;

        match rows.len() {
            1 => Ok(FeedUrl::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating feed url")),
        }
    }

    pub async fn get_urls_by_feed_id(&self, feed_id: &Uuid) -> Result<Vec<FeedUrl>> {
        self.pool
            .get()
            .await?
            .query("SELECT * FROM feed_url WHERE feed_id=$1", &[feed_id])
            .await?
            .iter()
            .map(FeedUrl::try_from)
            .collect()
    }

    pub async fn get_channel_by_title_feed_id(
        &self,
        title: &str,
        feed_id: &Uuid,
    ) -> Result<Option<ChannelVal>> {
        let rows = self
            .pool
            .get()
            .await?
            .query(
                "SELECT * FROM channel_val WHERE title=$1 AND feed_id=$2",
                &[&title, feed_id],
            )
            .await?;

        match rows.len() {
            0 => Ok(None),
            1 => Ok(Some(ChannelVal::try_from(&rows[0])?)),
            _ => Err(anyhow::Error::msg("more than one row found")),
        }
    }

    pub async fn create_channel(
        &self,
        title: &str,
        description: &str,
        image: &Option<String>,
        feed_id: &Uuid,
    ) -> Result<ChannelVal> {
        let rows = self.pool.get().await?.query("INSERT INTO channel_val (id, title, description, image, feed_id) VALUES ($1, $2, $3, $4, $5) RETURNING *", &[&Uuid::new_v4(), &title, &description, &image, feed_id]).await?;

        match rows.len() {
            1 => Ok(ChannelVal::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating channel")),
        }
    }

    pub async fn update_channel(&self, channel: &ChannelVal) -> Result<ChannelVal> {
        let rows = self.pool.get().await?.query("UPDATE channel_val SET title=$1, description=$2, image=$3, feed_id=$4 WHERE id=$5 RETURNING *", &[&channel.title, &channel.description, &channel.image, &channel.feed_id, &channel.id]).await?;

        match rows.len() {
            1 => Ok(ChannelVal::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating channel")),
        }
    }

    pub async fn get_item_by_id(&self, id: &Uuid) -> Result<ItemVal> {
        let rows = self
            .pool
            .get()
            .await?
            .query("SELECT * FROM item_val WHERE id = $1", &[id])
            .await?;

        match rows.len() {
            0 => Err(anyhow::Error::msg("item not found")),
            1 => Ok(ItemVal::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("more than one row found")),
        }
    }

    pub async fn get_item_by_title_date_channel_id(
        &self,
        title: &str,
        date: &DateTime<FixedOffset>,
        channel_id: &Uuid,
    ) -> Result<Option<ItemVal>> {
        let rows = self
            .pool
            .get()
            .await?
            .query(
                "SELECT * FROM item_val WHERE title=$1 AND date=$2 AND channel_id=$3",
                &[&title, date, channel_id],
            )
            .await?;

        match rows.len() {
            0 => Ok(None),
            1 => Ok(Some(ItemVal::try_from(&rows[0])?)),
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
        size: i64,
    ) -> Result<ItemVal> {
        let rows = self.pool.get().await?.query("INSERT INTO item_val (id, title, date, enclosure_type, enclosure_url, channel_id, size) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *", &[&Uuid::new_v4(), &title, date, &enclosure_type, &enclosure_url, channel_id, &size]).await?;

        match rows.len() {
            1 => Ok(ItemVal::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error creating channel")),
        }
    }

    pub async fn update_item(&self, item: &ItemVal) -> Result<ItemVal> {
        let rows = self.pool.get().await?.query("UPDATE items SET title=$1, date=$2, enclosure_type=$3, enclosure_url=$4, channel_id=$5 size=$6 WHERE id=$7 RETURNING *", &[&item.title, &item.date, &item.enclosure_type, &item.enclosure_url, &item.channel_id, &item.size, &item.id]).await?;

        match rows.len() {
            1 => Ok(ItemVal::try_from(&rows[0])?),
            _ => Err(anyhow::Error::msg("error updating channel")),
        }
    }
}
