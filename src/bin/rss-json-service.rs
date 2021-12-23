#[macro_use]
extern crate rocket;
extern crate rss_json_service;
use anyhow::Context;
use chrono::DateTime;
use hyper::{body::Bytes, body::HttpBody as _, header::ToStrError, http::uri::InvalidUri};
use log::error;
use rocket::{
    http::Status, response, response::stream::ByteStream, response::Responder, serde::json::Json,
    Request, State,
};
use rss_json_service::{
    fetcher,
    repo::{channel::Channel, feed::Feed, item::Item, Repo},
    types::service_config,
};
use std::{env, str};
use tokio::{
    fs,
    time::{sleep, Duration},
};
use uuid::Uuid;

const TIMEOUT: Duration = Duration::from_secs(3);

#[get("/feeds?<since>")]
async fn get_feeds(
    repo: &State<Repo>,
    since: Option<String>,
) -> Result<Json<Vec<Channel>>, CustomError> {
    match since {
        Some(s) => Ok(Json(
            repo.get_all_channels(Some(
                DateTime::parse_from_rfc3339(&s)
                    .context(format!("could not parse filter date \"{}\"", s))?,
            ))
            .await?,
        )),
        None => Ok(Json(repo.get_all_channels(None).await?)),
    }
}

#[post("/feeds", data = "<url>")]
async fn post_feeds(repo: &State<Repo>, url: String) -> Result<Json<Feed>, CustomError> {
    Ok(Json(repo.create_feed(&url).await?))
}

#[get("/channels?<since>")]
async fn channels(
    repo: &State<Repo>,
    since: Option<String>,
) -> Result<Json<Vec<Channel>>, CustomError> {
    match since {
        Some(s) => Ok(Json(
            repo.get_all_channels(Some(
                DateTime::parse_from_rfc3339(&s)
                    .context(format!("could not parse filter date \"{}\"", s))?,
            ))
            .await?,
        )),
        None => Ok(Json(repo.get_all_channels(None).await?)),
    }
}

#[get("/channels/<channel_id>/items?<since>")]
async fn channel_items(
    repo: &State<Repo>,
    channel_id: &str,
    since: Option<String>,
) -> Result<Json<Vec<Item>>, CustomError> {
    let channel_id = Uuid::parse_str(channel_id)?;

    match since {
        Some(s) => Ok(Json(
            repo.get_items_by_channel_id(
                &channel_id,
                Some(
                    DateTime::parse_from_rfc3339(&s)
                        .context(format!("could not parse filter date \"{}\"", s))?,
                ),
            )
            .await?,
        )),
        None => Ok(Json(repo.get_items_by_channel_id(&channel_id, None).await?)),
    }
}

#[get("/items/<item_id>/stream")]
async fn item_stream(repo: &State<Repo>, item_id: &str) -> Result<ByteStream![Bytes], CustomError> {
    let item_id = Uuid::parse_str(item_id)?;
    let item = repo.get_item_by_id(&item_id).await?;

    let mut res = fetcher::request(&item.enclosure_url, &TIMEOUT)
        .await
        .unwrap()
        .0;

    Ok(ByteStream! {
        while let Some(next) = res.data().await {
            let chunk = next.unwrap();
            yield chunk;
        }
    })
}

#[launch]
async fn rocket() -> _ {
    env_logger::init();

    let connection = match (
        env::var("RSS_JSON_SERVICE_CONNECTION"),
        env::var("RSS_JSON_SERVICE_CONFIG_FILE"),
    ) {
        (Ok(conn), _) => Ok(conn),
        (_, Ok(file)) => {
            let config: service_config::ServiceConfig =
                serde_json::from_str(&fs::read_to_string(file).await.unwrap()).unwrap();

            Ok(config.db_connection)
        }
        (_, _) => {
            error!("error reading configuration");
            Err(anyhow::anyhow!("error reading configuration"))
        }
    }
    .unwrap();

    let repo = match Repo::new(&connection).await {
        Ok(rep) => Ok(rep),
        Err(e) => {
            log::warn!("error creating repo; waiting 3s before retry: {}", e);
            sleep(Duration::from_secs(3)).await;
            Repo::new(&connection).await
        }
    }
    .unwrap();

    rocket::build().manage(repo).mount(
        "/",
        routes![channels, channel_items, item_stream, get_feeds, post_feeds],
    )
}

struct CustomError {
    msg: String,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for CustomError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        error!("{}", self.msg);
        Err(Status::InternalServerError)
    }
}

impl std::convert::From<anyhow::Error> for CustomError {
    fn from(e: anyhow::Error) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<uuid::Error> for CustomError {
    fn from(e: uuid::Error) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<ToStrError> for CustomError {
    fn from(e: ToStrError) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<InvalidUri> for CustomError {
    fn from(e: InvalidUri) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}

impl std::convert::From<hyper::Error> for CustomError {
    fn from(e: hyper::Error) -> Self {
        CustomError {
            msg: format!("{}", e),
        }
    }
}
