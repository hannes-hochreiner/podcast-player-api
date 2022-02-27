use chrono::{DateTime, TimeZone, Utc};

use crate::rss_feed::{RssEnclosure, RssItem};

use super::{RssChannel, RssFeed};
use std::{convert::TryFrom, fs};

#[test]
fn parse_bots() {
    let feed = RssFeed::try_from(&*fs::read_to_string("testFiles/bots.xml").unwrap()).unwrap();

    assert_eq!(feed.channels.len(), 1);
    assert_eq!(
        feed.channels[0],
        RssChannel {
            title: String::from("O'Reilly Bots Podcast - O'Reilly Media Podcast"),
            description: String::from("The O'Reilly Bots Podcast covers advances in conversational user interfaces, artificial intelligence, and messaging that are revolutionizing the way we interact with software."),
            image: Some(String::from("http://cdn.oreilly.com/radar/bot-podcast/avatar_Bots_1400x1400.png")),
            items: vec![RssItem {
                title: String::from("Jason Laska and Michael Akilian on using AI to schedule meetings"),
                date: Utc.ymd(2017, 05, 25).and_hms(10, 30, 00).into(),
                enclosure: RssEnclosure {
                    url: String::from("http://dts.podtrac.com/redirect.mp3/cdn.oreillystatic.com/radar/bot-podcast/Jason_Laska_and_Michael_Akilian_on_scheduling_bots.mp3"),
                    mime_type: String::from("audio/mpeg"),
                    length: 44983910,
                }
            },
            RssItem {
                title: String::from("Chris Messina on Facebook as a utility"),
                date: Utc.ymd(2017, 05, 11).and_hms(10, 45, 00).into(),
                enclosure: RssEnclosure {
                    url: String::from("http://dts.podtrac.com/redirect.mp3/cdn.oreillystatic.com/radar/bot-podcast/Chris_Messina_on_Facebook_as_a_utility.mp3"),
                    mime_type: String::from("audio/mpeg"),
                    length: 68157440,
                }
            }],
        }
    );
}

#[test]
fn parse_c_radar() {
    let feed = RssFeed::try_from(&*fs::read_to_string("testFiles/c-radar.xml").unwrap()).unwrap();

    assert_eq!(feed.channels.len(), 1);
    assert_eq!(
        feed.channels[0],
        RssChannel {
            title: String::from("C-RadaR"),
            description: String::from("Monatliche Radiosendung des Chaos Computer Clubs auf Radio Darmstadt. Jeden 2ten Donnerstag im Monat, 21-23 Uhr. 103,4 MHz / 99,85 MHz im Kabel / Stream. Tune In!"),
            image: Some(String::from("https://www.c-radar.de/wp-content/uploads/2017/07/Cradar_Logo1.jpg")),
            items: vec![
                RssItem {
                    title: String::from("c-radar September 2017 – Martin Tschirsich zu PC-Wahl, Piratenspitzenkandidat Sebulino und ein Überraschungsgast"),
                    date: DateTime::parse_from_rfc3339("2017-09-15T11:00:36.000Z").unwrap(),
                    enclosure: RssEnclosure {
                        url: String::from("http://ftp.ccc.de/broadcast/c-radar/c-radar/2017/c-radar-09-2017.mp3"),
                        mime_type: String::from("audio/mpeg"),
                        length: 285600026,
                    }
                },
                RssItem {
                    title: "c-radar Juli 2017 – zwei Berichte vom G20 in HH; NODE Forum, CCC Regiotreff,  Grundrechte Demo in Berlin".into(),
                    date: DateTime::parse_from_rfc3339("2017-07-27T20:39:43.000Z").unwrap(),
                    enclosure: RssEnclosure {
                        url: "http://ftp.ccc.de/broadcast/c-radar/c-radar/2017/c-radar-07-2017.mp3".into(),
                        mime_type: "audio/mpeg".into(),
                        length: 287815167,
                    }
                },
                RssItem {
                    title: "c-radar Juni 2017 – BalCCon, FF Hessentag, Night of Science, SHA2017, Debian Release Party".into(),
                    date: DateTime::parse_from_rfc3339("2017-06-12T08:33:35.000Z").unwrap(),
                    enclosure: RssEnclosure {
                        url: "http://ftp.ccc.de/broadcast/c-radar/c-radar/2017/c-radar-06-2017.mp3".into(),
                        mime_type: "audio/mpeg".into(),
                        length: 288079553,
                    }
                }
            ],
        }
    );
}

#[test]
fn parse_empty() {
    let feed = RssFeed::try_from(&*fs::read_to_string("testFiles/empty.xml").unwrap()).unwrap();

    assert_eq!(feed.channels.len(), 1);
    assert_eq!(
        feed.channels[0],
        RssChannel {
            title: String::from("C-RadaR"),
            description: String::from("Monatliche Radiosendung des Chaos Computer Clubs auf Radio Darmstadt. Jeden 2ten Donnerstag im Monat, 21-23 Uhr. 103,4 MHz / 99,85 MHz im Kabel / Stream. Tune In!"),
            image: None,
            items: vec![],
        }
    );
}

#[test]
fn parse_lin_digres() {
    let feed =
        RssFeed::try_from(&*fs::read_to_string("testFiles/lin_digres.xml").unwrap()).unwrap();

    assert_eq!(feed.channels.len(), 1);
    assert_eq!(
        feed.channels[0],
        RssChannel {
            title: String::from("Linear Digressions"),
            description: String::from(
                "search for me"
            ),
            image: None,
            items: vec![
                RssItem {
                    title: "Autoencoders".into(),
                    date: DateTime::parse_from_rfc3339("2018-03-12T01:46:48Z").unwrap(),
                    enclosure: RssEnclosure {
                        url: "http://static1.squarespace.com/static/56c89b14c2ea51c475ee830b/t/5aa5dbc6c8302542e9d31410/1520819158384/autoencoders+produced.mp3".into(),
                        mime_type: "audio/mpeg".into(),
                        length: 6090534,
                    }
                },
                RssItem {
                    title: "When is open data too open?".into(),
                    date: DateTime::parse_from_rfc3339("2018-03-05T03:34:55Z").unwrap(),
                    enclosure: RssEnclosure {
                        url: "http://static1.squarespace.com/static/56c89b14c2ea51c475ee830b/t/5a9cba3dc830255b24811f2b/1520220754818/data+privacy+produced.mp3".into(),
                        mime_type: "audio/mpeg".into(),
                        length: 12644552,
                    }
                }
            ],
        }
    );
}

#[test]
fn parse_minkorrekt() {
    let feed =
        RssFeed::try_from(&*fs::read_to_string("testFiles/minkorrekt.xml").unwrap()).unwrap();

    assert_eq!(feed.channels.len(), 1);
    assert_eq!(
        feed.channels[0],
        RssChannel {
            title: String::from("Methodisch inkorrekt!"),
            description: String::from("der methodisch inkorrekte Wissenschaftspodcast"),
            image: Some(String::from("https://images.podigee-cdn.net/0x,srhipgyug44BeyTko7VrGtswPXmBVkFavcRQPA4gts-E=/https://cdn.podigee.com/uploads/u2487/51751e79-4c75-43cc-b213-8e0cd040f439.jpg")),
            items: vec![RssItem {
                title: String::from("Mi206 - \"Zwischen den Jahren\""),
                date: Utc.ymd(2021, 12, 29).and_hms(09, 55, 58).into(),
                enclosure: RssEnclosure {
                    url: String::from("https://cdn.podigee.com/media/podcast_3451_methodisch_inkorrekt_episode_627156_mi206_zwischen_den_jahren.mp3?v=1640772750&source=feed"),
                    mime_type: String::from("audio/mpeg"),
                    length: 253548623,
                }
            },
            RssItem {
                title: String::from("Mi205 - \"Mettbett\""),
                date: Utc.ymd(2021, 12, 14).and_hms(18, 55, 06).into(),
                enclosure: RssEnclosure {
                    url: String::from("https://adn.podigee.com/adswizz/media/podcast_3451_methodisch_inkorrekt_episode_615122_mi205_mettbett.mp3?awCollectionId=svo_cea0cf&awEpisodeId=615122&source=feed&v=1639560073"),
                    mime_type: String::from("audio/mpeg"),
                    length: 302222176,
                }
            }],
        }
    );
}

#[test]
fn parse_quanta() {
    let feed = RssFeed::try_from(&*fs::read_to_string("testFiles/quanta.xml").unwrap()).unwrap();

    assert_eq!(feed.channels.len(), 1);
    assert_eq!(feed.channels[0].items.len(), 5);
}

#[test]
fn parse_minkorrekt_missing() {
    let _ = RssFeed::try_from(&*fs::read_to_string("testFiles/minkorrekt_missing.xml").unwrap())
        .unwrap();
}

#[test]
fn itunes_summary() {
    let feed =
        RssFeed::try_from(&*fs::read_to_string("testFiles/itunes_summary.xml").unwrap()).unwrap();
    assert_eq!(
        feed.channels[0],
        RssChannel {
            title: String::from("title"),
            description: String::from("itunes summary"),
            image: None,
            items: vec![]
        }
    );
}

#[test]
fn parse_date_1() {
    assert_eq!(
        RssFeed::parse_date("Tue, 13 Mar 2018 19:08:36 +0000").unwrap(),
        DateTime::parse_from_rfc3339("2018-03-13T19:08:36Z").unwrap()
    );
}

#[test]
fn parse_date_2() {
    assert_eq!(
        RssFeed::parse_date("25 Nov 2016 9:30:00 +0100").unwrap(),
        DateTime::parse_from_rfc3339("2016-11-25T08:30:00Z").unwrap()
    );
}
