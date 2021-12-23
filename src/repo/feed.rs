use anyhow::Result;
use chrono::{DateTime, FixedOffset};
use std::convert::TryFrom;
use tokio_postgres::Row;
use uuid::Uuid;

#[derive(Clone)]
pub struct Feed {
    pub id: Uuid,
    pub url: String,
    pub update_ts: DateTime<FixedOffset>,
}

impl TryFrom<&Row> for Feed {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Feed {
            id: row.try_get("id")?,
            url: row.try_get("url")?,
            update_ts: row.try_get("update_ts")?,
        })
    }
}
