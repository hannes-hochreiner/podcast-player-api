[![CI](https://github.com/hannes-hochreiner/rss-json-service/actions/workflows/main.yml/badge.svg)](https://github.com/hannes-hochreiner/rss-json-service/actions/workflows/main.yml)
# rss-json-service
A simple RSS to JSON web service.

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
## Development

### Setup

A local container was used as a development database.

```bash
podman run --name rss_json -e POSTGRES_DB=rss_json -e POSTGRES_PASSWORD=<password> -p 5432:5432 -d postgres:alpine
```

The database scripts were then executed using the local psql.

```bash
psql postgresql://postgres:<password>@localhost:5432/rss_json -f pg-scripts/2021-06-13_create_db.sql
```

An Ansible script automating this process can be found in the `ansible` folder.
The scripts expects the variables listed in the table below, which must be provided in a file names `vars.yml` in the Ansible folder.

| variable name | description |
| ------------- | ----------- |
| db_password | db master password |
| updater_password | password for the updater user |
| service_password | password for the service user |

If the password is encrypted with Ansible vault, the ansible script can be run with the following command:

```bash
ansible-playbook --ask-vault-pass ansible/db_create_pb.yml
```

### Test entries

A program names `test_inserter` is provided to create some entries in the feeds table.
The program can be run with the following command:

```bash
TEST_INSERTER_CONNECTION=postgresql://<test inserter db user>:<test inserter password>@<host>:5432/rss_json cargo run --bin test_inserter
```

## Deployment

### rss-json-service

The rss-json-service expects an environment variables providing the database credentials.
Alternatively, the connection string can be provided in a configuration file.
The path of the configuration file needs to be provided in an environment variable.
In both cases, the environment variable `RUST_LOG` can be used to set the logging level.

```bash
RSS_JSON_SERVICE_CONNECTION=postgresql://<rss-json db user>:<rss-json password>@<host>:5432/rss_json cargo run --bin rss-json-service
RUST_LOG=info RSS_JSON_SERVICE_CONFIG_FILE=config.json cargo run --bin updater
```

```json
{
    "db_connection": "postgresql://<service db user>:<service password>@<host>:5432/rss_json"
}
```

### updater

The updater tries to obtain the connection string for the database from the environment variable `UPDATER_CONNECTION`.
Alternatively, the connection string can be provided in a configuration file.
The path of the configuration file needs to be provided in an environment variable.
In both cases, the environment variable `RUST_LOG` can be used to set the logging level.

```bash
UPDATER_CONNECTION=postgresql://<updater db user>:<updater password>@<host>:5432/rss_json cargo run --bin updater
RUST_LOG=info UPDATER_CONFIG_FILE=config.json cargo run --bin updater
```

```json
{
    "db_connection": "postgresql://<updater db user>:<updater password>@<host>:5432/rss_json"
}
```
