use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use std::convert::TryFrom;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Item {
    pub id: Uuid,
    pub title: String,
    pub date: DateTime<FixedOffset>,
    pub enclosure_type: String,
    pub enclosure_url: String,
    pub channel_id: Uuid,
}

impl Item {
    pub fn needs_update(&self, enclosure_type: &String, enclosure_url: &String) -> bool {
        if &self.enclosure_type == enclosure_type && &self.enclosure_url == enclosure_url {
            false
        } else {
            true
        }
    }
}

impl TryFrom<&Row> for Item {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Item {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            date: row.try_get("date")?,
            enclosure_type: row.try_get("enclosure_type")?,
            enclosure_url: row.try_get("enclosure_url")?,
            channel_id: row.try_get("channel_id")?,
        })
    }
}
