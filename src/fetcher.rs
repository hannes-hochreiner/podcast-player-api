use anyhow::{anyhow, Context, Result};
use hyper::{client::HttpConnector, Body, Client, Response, StatusCode};
use hyper_tls::HttpsConnector;
use tokio::time::{self, Duration};

pub async fn request(
    url: &str,
    timeout: &Duration,
) -> Result<(Option<Response<Body>>, Vec<(String, i16)>)> {
    let mut urls: Vec<(String, i16)> = Vec::new();
    let mut res = time::timeout(*timeout, internal_request(url)).await??;
    urls.push((url.into(), res.status().as_u16() as i16));

    while (res.status() == StatusCode::TEMPORARY_REDIRECT)
        || (res.status() == StatusCode::FOUND)
        || (res.status() == StatusCode::PERMANENT_REDIRECT)
        || (res.status() == StatusCode::MOVED_PERMANENTLY)
    {
        let next_url = res.headers()["location"].to_str()?.to_string();

        res = time::timeout(*timeout, internal_request(&*next_url)).await??;
        urls.push((next_url.into(), res.status().as_u16() as i16));
    }

    if res.status() == StatusCode::OK {
        Ok((Some(res), urls))
    } else {
        Ok((None, urls))
    }
}

async fn internal_request(url: &str) -> Result<Response<Body>> {
    let uri: hyper::Uri = url.parse()?;

    match uri.scheme_str() {
        Some(s) => match s {
            "http" => Client::builder()
                .build::<_, Body>(HttpConnector::new())
                .get(uri)
                .await
                .context("request failed"),
            "https" => Client::builder()
                .build::<_, Body>(HttpsConnector::new())
                .get(uri)
                .await
                .context("request failed"),
            _ => Err(anyhow!("no connector available for scheme \"{}\"", s)),
        },
        None => Err(anyhow!("scheme not recognized")),
    }
}
