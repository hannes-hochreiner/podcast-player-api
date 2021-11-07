use anyhow::Result;
use serde::Serialize;
use std::convert::TryFrom;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Channel {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub image: String,
    pub feed_id: Uuid,
}

impl TryFrom<&Row> for Channel {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Channel {
            id: row.try_get("id")?,
            description: row.try_get("description")?,
            title: row.try_get("title")?,
            image: row.try_get("image")?,
            feed_id: row.try_get("feed_id")?,
        })
    }
}
