use anyhow::{anyhow, Context, Result};
use hyper::{client::HttpConnector, Body, Client, Response, StatusCode};
use hyper_tls::HttpsConnector;
use tokio::time::{self, Duration};

pub async fn request(url: &str, timeout: &Duration) -> Result<(Response<Body>, Option<String>)> {
    let mut res = time::timeout(*timeout, internal_request(url)).await??;
    let mut new_url = None;

    while (res.status() == StatusCode::TEMPORARY_REDIRECT)
        || (res.status() == StatusCode::FOUND)
        || (res.status() == StatusCode::PERMANENT_REDIRECT)
        || (res.status() == StatusCode::MOVED_PERMANENTLY)
    {
        let next_url = res.headers()["location"].to_str()?;

        if (res.status() == StatusCode::PERMANENT_REDIRECT)
            || (res.status() == StatusCode::MOVED_PERMANENTLY)
        {
            new_url = Some(String::from(next_url));
        }

        res = time::timeout(*timeout, internal_request(next_url)).await??;
    }

    if res.status() == StatusCode::OK {
        Ok((res, new_url))
    } else {
        Err(anyhow!(format!("request of \"{}\" failed", url)))
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
