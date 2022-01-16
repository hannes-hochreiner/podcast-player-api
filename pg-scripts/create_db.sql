CREATE TABLE feed (
  id uuid PRIMARY KEY,
  title varchar(1024) UNIQUE NOT NULL,
  synced boolean NOT NULL,
  update_ts timestamp with time zone NOT NULL
);

CREATE TABLE feed_url (
  id uuid PRIMARY KEY,
  url varchar(1024) UNIQUE NOT NULL,
  feed_id uuid REFERENCES feed (id) NOT NULL,
  manual boolean NOT NULL,
  status int,
  synced boolean NOT NULL,
  update_ts timestamp with time zone NOT NULL
);

CREATE TABLE channel (
  id uuid PRIMARY KEY,
  title varchar(512) UNIQUE NOT NULL,
  description varchar(2048) NOT NULL,
  image varchar(1024),
  feed_id uuid REFERENCES feed (id) NOT NULL,
  update_ts timestamp with time zone NOT NULL
);

CREATE TABLE channel_meta (
  user_id varchar(512),
  id uuid PRIMARY KEY,
  channel_id uuid REFERENCES channel (id) NOT NULL,
  active boolean NOT NULL,
  synced boolean NOT NULL,
  volume float NOT NULL,
  playback_rate float NOT NULL,
  update_ts timestamp with time zone NOT NULL
);

CREATE TABLE item (
  id uuid PRIMARY KEY,
  title varchar(512) NOT NULL,
  date timestamp with time zone NOT NULL,
  enclosure_type varchar(128) NOT NULL,
  enclosure_url varchar(1024) NOT NULL,
  channel_id uuid REFERENCES channel (id) NOT NULL,
  size int NOT NULL,
  update_ts timestamp with time zone NOT NULL
);

CREATE TYPE download_status AS ENUM ('NotRequested', 'Pending', 'InProgress', 'Ok', 'Error');

CREATE TABLE item_meta {
  user_id varchar(512),
  id uuid PRIMARY KEY,
  item_id uuid REFERENCES item (id) NOT NULL,
  new boolean NOT NULL,
  download boolean NOT NULL,
  download_status download_status NOT NULL,
  current_time float,
  play_count int NOT NULL,
  synced boolean NOT NULL,
  update_ts timestamp with time zone NOT NULL
}

CREATE FUNCTION set_update_timestamp() RETURNS trigger AS $$
BEGIN
  new.update_ts := current_timestamp;
  RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER insert_timestamp_item
BEFORE INSERT ON item
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER update_timestamp_item
BEFORE UPDATE ON item
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER insert_timestamp_channel
BEFORE INSERT ON channel
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER update_timestamp_channel
BEFORE UPDATE ON channel
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER insert_timestamp_feed
BEFORE INSERT ON feed
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER update_timestamp_feed
BEFORE UPDATE ON feed
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER insert_timestamp_feed_url
BEFORE INSERT ON feed_url
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE TRIGGER update_timestamp_feed_url
BEFORE UPDATE ON feed_url
FOR EACH ROW 
EXECUTE PROCEDURE set_update_timestamp();

CREATE ROLE api_updater LOGIN PASSWORD '{{updater_password}}';

GRANT SELECT, INSERT, UPDATE ON feed TO api_updater;
GRANT SELECT, INSERT, UPDATE ON feed_url TO api_updater;
GRANT SELECT, INSERT, UPDATE ON channel TO api_updater;
GRANT SELECT, INSERT, UPDATE ON item TO api_updater;

CREATE ROLE api_service LOGIN PASSWORD '{{service_password}}';

GRANT SELECT ON channel TO api_service;
GRANT SELECT ON item TO api_service;
GRANT SELECT ON feed TO api_service;
GRANT SELECT ON feed_url TO api_service;
GRANT SELECT, INSERT, UPDATE ON channel_meta TO api_service;
GRANT SELECT, INSERT, UPDATE ON item_meta TO api_service;
