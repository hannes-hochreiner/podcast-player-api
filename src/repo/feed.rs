use anyhow::Result;
use std::convert::TryFrom;
use tokio_postgres::Row;
use uuid::Uuid;

pub struct Feed {
    pub id: Uuid,
    pub url: String,
}

impl TryFrom<&Row> for Feed {
    type Error = anyhow::Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Feed {
            id: row.try_get("id")?,
            url: row.try_get("url")?,
        })
    }
}
