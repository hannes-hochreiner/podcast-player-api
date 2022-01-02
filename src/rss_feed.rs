use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use roxmltree::{Document, Node};
use std::convert::TryFrom;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub struct RssFeed {
    pub channels: Vec<RssChannel>,
}

#[derive(Debug, PartialEq)]
pub struct RssChannel {
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub items: Vec<RssItem>,
}

#[derive(Debug, PartialEq)]
pub struct RssEnclosure {
    pub url: String,
    pub mime_type: String,
    pub length: i32,
}

#[derive(Debug, PartialEq)]
pub struct RssItem {
    pub date: DateTime<FixedOffset>,
    pub title: String,
    pub enclosure: RssEnclosure,
}

impl RssFeed {
    fn parse_root(root: Node) -> Result<RssFeed> {
        let mut channels = Vec::<RssChannel>::new();

        if root.tag_name().name() == "channel" {
            channels.push(Self::parse_channel(root)?);
        } else {
            for node in root.children() {
                match node.tag_name().name() {
                    "channel" => channels.push(Self::parse_channel(node)?),
                    _ => {}
                }
            }
        }

        Ok(RssFeed { channels })
    }

    fn parse_channel(channel: Node) -> Result<RssChannel> {
        let mut title: Option<String> = None;
        let mut description: Option<String> = None;
        let mut itunes_description: Option<String> = None;
        let mut image: Option<String> = None;
        let mut items: Vec<RssItem> = Vec::new();

        for node in channel.children() {
            match (node.tag_name().namespace(), node.tag_name().name()) {
                (_, "title") => {
                    title = node.text().map(|e| String::from(e));
                }
                (_, "description") => {
                    description = node.text().map(|e| String::from(e.trim()));
                }
                (Some("http://www.itunes.com/dtds/podcast-1.0.dtd"), "summary") => {
                    itunes_description = node.text().map(|e| String::from(e.trim()));
                }
                (_, "image") => {
                    image = Self::parse_image(node)?;
                }
                (_, "item") => match Self::parse_item(node) {
                    Ok(item) => items.push(item),
                    Err(e) => log::error!("error parsing item: {}", e),
                },
                _ => {}
            }
        }

        match (title, description, itunes_description) {
            (Some(title), _, Some(description)) => Ok(RssChannel {
                title,
                description,
                image,
                items,
            }),
            (Some(title), Some(description), None) => Ok(RssChannel {
                title,
                description,
                image,
                items,
            }),
            _ => Err(anyhow::Error::msg(
                "either the title or the description could not be found",
            )),
        }
    }

    fn parse_image(image: Node) -> Result<Option<String>> {
        let mut image_url: Option<&str> = image.attribute("href");

        if image_url.is_none() {
            for node in image.children() {
                match node.tag_name().name() {
                    "url" => image_url = node.text(),
                    _ => {}
                }
            }
        }

        Ok(image_url.map(|e| String::from(e)))
    }

    fn parse_item(item: Node) -> Result<RssItem> {
        let mut title: Option<&str> = None;
        let mut date: Option<&str> = None;
        let mut enclosure: Option<RssEnclosure> = None;

        for node in item.children() {
            match node.tag_name().name() {
                "title" => title = node.text(),
                "pubDate" => date = node.text(),
                "enclosure" => enclosure = Some(Self::parse_enclosure(node)?),
                _ => {}
            }
        }

        log::debug!(
            "parsing item finished: title: {:?}, date: {:?}, enclosure: {:?}",
            title,
            date,
            enclosure
        );

        match (title, date, enclosure) {
            (Some(title), Some(date), Some(enclosure)) => Ok(RssItem {
                title: String::from(title),
                date: Self::parse_date(date)?,
                enclosure,
            }),
            (None, _, _) => Err(anyhow::anyhow!("could not find title for item")),
            (_, None, _) => Err(anyhow::anyhow!("could not find date for item")),
            (title, _, None) => Err(anyhow::anyhow!(
                "could not find enclosure for item: {:?}",
                title
            )),
        }
    }

    fn parse_enclosure(enclosure: Node) -> Result<RssEnclosure> {
        match (
            enclosure.attribute("url"),
            enclosure.attribute("type"),
            enclosure.attribute("length"),
        ) {
            (Some(url), Some(mime_type), Some(length)) => Ok(RssEnclosure {
                url: String::from(url),
                mime_type: String::from(mime_type),
                length: i32::from_str_radix(length, 10)?,
            }),
            _ => Err(anyhow::Error::msg("could not parse enclosure")),
        }
    }

    fn parse_date(date: &str) -> Result<DateTime<FixedOffset>> {
        let res = DateTime::parse_from_rfc2822(date).context("error parsing date");

        match res {
            Err(_) => DateTime::parse_from_str(date, "%d %b %Y %k:%M:%S%.3f %z")
                .context("error parsing date"),
            _ => res,
        }
    }
}

impl TryFrom<&str> for RssFeed {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let doc = Document::parse(value)?;

        Self::parse_root(doc.root_element())
    }
}
