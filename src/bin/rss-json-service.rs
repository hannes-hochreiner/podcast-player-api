extern crate podcast_player_api;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use podcast_player_api::{fetcher, repo::Repo, updater::Updater};
use podcast_player_common::{channel_val::ChannelVal, item_val::ItemVal, FeedVal};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{env, str};
use tokio::{fs, spawn, time::Duration};

#[derive(Debug, Deserialize)]
pub struct PodcastPlayerApiConfig {
    pub api_connection: String,
    pub updater_connection: String,
}

const TIMEOUT: Duration = Duration::from_secs(3);

async fn router(req: Request<Body>, repo: Repo) -> Result<Response<Body>, anyhow::Error> {
    let path = req.uri().path().split("/").collect::<Vec<&str>>();
    let query = req
        .uri()
        .query()
        .and_then(|s| {
            Some(
                s.split("&")
                    .map(|sub| {
                        let mut split = sub.split("=");
                        (
                            split
                                .nth(0)
                                .ok_or(anyhow::anyhow!("could not parse query string")),
                            split
                                .nth(0)
                                .ok_or(anyhow::anyhow!("could not parse query string")),
                        )
                    })
                    .map(|t| match (t.0, t.1) {
                        (Ok(key), Ok(val)) => Ok((key, val)),
                        (_, Err(e)) => Err(e),
                        (Err(e), _) => Err(e),
                    })
                    .collect::<Result<HashMap<&str, &str>, anyhow::Error>>(),
            )
        })
        .transpose()?;

    match (req.method(), &path[1..]) {
        (&Method::GET, &["feeds"]) => Ok(Response::new(Body::from(serde_json::to_string(
            &repo
                .get_objects::<FeedVal>(query.and_then(|q| q.get(&"since").cloned()))
                .await?,
        )?))),
        (&Method::GET, &["channels"]) => Ok(Response::new(Body::from(serde_json::to_string(
            &repo
                .get_objects::<ChannelVal>(query.and_then(|q| q.get(&"since").cloned()))
                .await?,
        )?))),
        (&Method::GET, &["items"]) => Ok(Response::new(Body::from(serde_json::to_string(
            &repo
                .get_objects::<ItemVal>(query.and_then(|q| q.get(&"since").cloned()))
                .await?,
        )?))),
        (&Method::GET | &Method::HEAD, &["items", id, "stream"]) => {
            let item = repo.get_item_by_id(&id.parse()?).await?;

            fetcher::request(&*item.enclosure_url, &TIMEOUT, &req.method())
                .await?
                .0
                .ok_or(anyhow::anyhow!("error requesting enclosure"))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let config: PodcastPlayerApiConfig = serde_json::from_str(
        &fs::read_to_string(env::var("PODCAST_PLAYER_API_CONFIG_FILE").unwrap())
            .await
            .unwrap(),
    )
    .unwrap();

    let repo = Repo::new(&config.api_connection).await?;

    let updater = Updater::new(&config.updater_connection);

    spawn(async move { updater.update_loop().await });

    let addr: SocketAddr = match env::var("HYPER_BIND_ADDRESS") {
        Ok(s) => s,
        Err(_) => String::from("127.0.0.1:8000"),
    }
    .parse()
    .unwrap();
    let service = make_service_fn(|_| {
        let repo = repo.clone();
        async { Ok::<_, anyhow::Error>(service_fn(move |req| router(req, repo.to_owned()))) }
    });
    let server = Server::bind(&addr).serve(service);

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
