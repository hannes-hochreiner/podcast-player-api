[![CI](https://github.com/hannes-hochreiner/podcast-player-api/actions/workflows/main.yml/badge.svg)](https://github.com/hannes-hochreiner/podcast-player-api/actions/workflows/main.yml)
# Podcast Player API
A simple RSS to JSON web service created as part of the [Podcast Player](https://github.com/hannes-hochreiner/podcast-player) project.

The service parses a list of RSS feeds periodically into objects with the following properties:

```rust
pub struct RssFeed {
    channels: Vec<RssChannel>,
}

pub struct RssChannel {
    title: String,
    description: String,
    image: Option<String>,
    items: Vec<RssItem>,
}

struct RssEnclosure {
    url: String,
    mime_type: String,
    length: i32,
}

struct RssItem {
    date: DateTime<FixedOffset>,
    title: String,
    enclosure: RssEnclosure,
}
```

## Deployment

The podcast-player-api expects an environment variables providing the path to the configuration file.
The environment variable `RUST_LOG` can be used to set the logging level.

| variable name | description |
| ------------- | ----------- |
| RUST_LOG | logging level (e.g., debug, info) |
| PODCAST_PLAYER_API_CONFIG_FILE | path to the configuration file |

The configuration file has the following format.

```json
{
    "api_connection": "postgresql://<service db user>:<service password>@<host>:5432/rss_json",
    "updater_connection": "postgresql://<updater db user>:<updater password>@<host>:5432/rss_json"
}
```

## License

This work is licensed under the MIT license.

`SPDX-License-Identifier: MIT`
