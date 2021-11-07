use anyhow::Result;
use std::env;
use tokio_postgres::NoTls;
use uuid::{self, Uuid};

#[tokio::main] // By default, tokio_postgres uses the tokio crate as its runtime.
async fn main() -> Result<()> {
    // Connect to the database.
    let (client, connection) =
        tokio_postgres::connect(&*env::var("TEST_INSERTER_CONNECTION")?, NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let urls = vec![
        "https://rss.art19.com/the-take",
        "https://feed.podbean.com/wtyppod/feed.xml",
        "https://www.theverge.com/rss/index.xml",
    ];

    for url in urls {
        client
            .query(
                "INSERT INTO feeds (id, url) VALUES ($1, $2)",
                &[&Uuid::new_v4(), &url],
            )
            .await?;
    }

    Ok(())
}
