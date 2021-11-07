#[macro_use]
extern crate rocket;
extern crate rss_json_service;
use hyper::{
    body::Bytes, body::HttpBody as _, header::ToStrError, http::uri::InvalidUri,
    Client as HttpClient, StatusCode,
};
use hyper_tls::HttpsConnector;
use log::error;
use rocket::{
    http::Status, response, response::stream::ByteStream, response::Responder, serde::json::Json,
    Request, State,
};
use rss_json_service::repo::{channel::Channel, item::Item, Repo};
use std::{env, str};
use uuid::Uuid;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/channels")]
async fn channels(repo: &State<Repo>) -> Result<Json<Vec<Channel>>, CustomError> {
    let channels = repo.get_all_channels().await?;

    Ok(Json(channels))
}

#[get("/channels/<channel_id>/items")]
async fn channel_items(
    repo: &State<Repo>,
    channel_id: &str,
) -> Result<Json<Vec<Item>>, CustomError> {
    let channel_id = Uuid::parse_str(channel_id)?;

    Ok(Json(repo.get_items_by_channel_id(&channel_id).await?))
}

#[get("/items/<item_id>/stream")]
async fn item_stream(repo: &State<Repo>, item_id: &str) -> Result<ByteStream![Bytes], CustomError> {
    let item_id = Uuid::parse_str(item_id)?;
    let item = repo.get_item_by_id(&item_id).await?;
    let https = HttpsConnector::new();
    let http_client = HttpClient::builder().build::<_, hyper::Body>(https);
    let mut res = http_client.get(item.enclosure_url.parse()?).await.unwrap();

    match res.status() {
        StatusCode::FOUND
        | StatusCode::MOVED_PERMANENTLY
        | StatusCode::TEMPORARY_REDIRECT
        | StatusCode::PERMANENT_REDIRECT => {
            res = http_client
                .get(res.headers()["location"].to_str()?.parse()?)
                .await?;
        }
        StatusCode::OK => {}
        _ => {}
    }

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
    let repo = Repo::new(&*env::var("RSS_JSON_CONNECTION").unwrap())
        .await
        .unwrap();

    rocket::build()
        .manage(repo)
        .mount("/", routes![index, channels, channel_items, item_stream])
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
