#[macro_use]
extern crate rocket;
extern crate podcast_player_api;
use hyper::{body::Bytes, body::HttpBody as _};
use podcast_player_api::{fetcher, repo::Repo, updater::Updater, CustomError};
use podcast_player_common::{channel_val::ChannelVal, item_val::ItemVal, FeedVal};
use rocket::{response::stream::ByteStream, serde::json::Json, State};
use serde::Deserialize;
use std::{env, str};
use tokio::{
    fs, spawn,
    time::{sleep, Duration},
};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PodcastPlayerApiConfig {
    pub api_connection: String,
    pub updater_connection: String,
}

const TIMEOUT: Duration = Duration::from_secs(3);

#[get("/feeds?<since>")]
async fn feeds(repo: &State<Repo>, since: Option<&str>) -> Result<Json<Vec<FeedVal>>, CustomError> {
    Ok(Json(repo.get_objects::<FeedVal>(since).await?))
}

// #[post("/feeds", data = "<url>")]
// async fn post_feeds(repo: &State<Repo>, url: String) -> Result<Json<Feed>, CustomError> {
//     Ok(Json(repo.create_feed(&url).await?))
// }

#[get("/channels?<since>")]
async fn channels(
    repo: &State<Repo>,
    since: Option<&str>,
) -> Result<Json<Vec<ChannelVal>>, CustomError> {
    Ok(Json(repo.get_objects::<ChannelVal>(since).await?))
}

#[get("/items?<since>")]
async fn items(repo: &State<Repo>, since: Option<&str>) -> Result<Json<Vec<ItemVal>>, CustomError> {
    Ok(Json(repo.get_objects::<ItemVal>(since).await?))
}

#[get("/items/<item_id>/stream")]
async fn item_stream(repo: &State<Repo>, item_id: &str) -> Result<ByteStream![Bytes], CustomError> {
    let item_id = Uuid::parse_str(item_id)?;
    let item = repo.get_item_by_id(&item_id).await?;

    let mut res = fetcher::request(&item.enclosure_url, &TIMEOUT)
        .await
        .unwrap()
        .0
        .ok_or(anyhow::anyhow!("request failed"))?;

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

    let config: PodcastPlayerApiConfig = serde_json::from_str(
        &fs::read_to_string(env::var("PODCAST_PLAYER_API_CONFIG_FILE").unwrap())
            .await
            .unwrap(),
    )
    .unwrap();

    let repo = match Repo::new(&config.api_connection).await {
        Ok(rep) => Ok(rep),
        Err(e) => {
            log::warn!("error creating repo; waiting 5s before retry: {}", e);
            sleep(Duration::from_secs(5)).await;
            Repo::new(&config.api_connection).await
        }
    }
    .unwrap();

    let updater = Updater::new(&config.updater_connection);

    spawn(async move { updater.update_loop().await });

    rocket::build().manage(repo).mount(
        "/",
        routes![channels, items, item_stream, feeds], //, post_feeds],
    )
}
