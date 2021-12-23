CREATE TABLE feeds (
  id uuid PRIMARY KEY,
  url varchar(1024) UNIQUE NOT NULL,
  update_ts timestamp with time zone NOT NULL
);

CREATE TABLE channels (
  id uuid PRIMARY KEY,
  title varchar(512) UNIQUE NOT NULL,
  description varchar(2048) NOT NULL,
  image varchar(1024),
  feed_id uuid REFERENCES feeds (id),
  update_ts timestamp with time zone NOT NULL
);

CREATE TABLE items (
  id uuid PRIMARY KEY,
  title varchar(512) NOT NULL,
  date timestamp with time zone NOT NULL,
  enclosure_type varchar(128) NOT NULL,
  enclosure_url varchar(1024) NOT NULL,
  channel_id uuid REFERENCES channels (id),
  update_ts timestamp with time zone NOT NULL
);

CREATE FUNCTION set_update_timestamp() RETURNS trigger AS $$
BEGIN
  new.update_ts := current_timestamp;
  RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_timestamp_items
BEFORE INSERT, UPDATE ON items
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER update_timestamp_channels
BEFORE INSERT, UPDATE ON channels
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER update_timestamp_feeds
BEFORE INSERT, UPDATE ON feeds
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE ROLE updater LOGIN PASSWORD '{{updater_password}}';

GRANT SELECT, INSERT, UPDATE ON feeds TO updater;
GRANT SELECT, INSERT, UPDATE ON channels TO updater;
GRANT SELECT, INSERT, UPDATE ON items TO updater;

CREATE ROLE rss_json_service LOGIN PASSWORD '{{service_password}}';

GRANT SELECT, INSERT ON feeds TO rss_json_service;
GRANT SELECT ON channels TO rss_json_service;
GRANT SELECT ON items TO rss_json_service;
