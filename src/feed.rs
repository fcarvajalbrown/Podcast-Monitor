use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset};
use quick_xml::{Reader, XmlVersion};
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};

pub const USER_AGENT: &str = "Mozilla/5.0 PodcastMonitor/0.1";
const MAX_FEED_BYTES: u64 = 200 * 1024 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub guid: String,
    pub title: String,
    pub published: Option<DateTime<FixedOffset>>,
    pub duration_s: u64,
    pub audio_url: String,
    pub description: String,
}

impl Episode {
    pub fn id(&self) -> String {
        let date = self.published.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "undated".into());
        format!("{date}-{}", slug(&self.title, 60))
    }
}

pub fn fetch(url: &str) -> Result<Vec<Episode>> {
    let body = ureq::get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .with_context(|| format!("fetching feed {url}"))?
        .body_mut()
        .with_config()
        .limit(MAX_FEED_BYTES)
        .read_to_string()
        .context("reading feed body")?;
    parse(&body)
}

pub fn newest(mut episodes: Vec<Episode>, count: usize) -> Vec<Episode> {
    episodes.sort_by(|a, b| b.published.cmp(&a.published));
    episodes.truncate(count);
    episodes
}

pub fn parse(xml: &str) -> Result<Vec<Episode>> {
    let mut reader = Reader::from_str(xml);
    let mut episodes = Vec::new();
    let mut current: Option<Episode> = None;
    let mut field = String::new();
    let mut text = String::new();
    loop {
        match reader.read_event().context("parsing feed XML")? {
            Event::Start(tag) => {
                let name = tag.name().as_ref().to_string();
                if name == "item" {
                    current = Some(Episode::default());
                } else if current.is_some() {
                    field = name;
                    text.clear();
                }
            }
            Event::Empty(tag) => {
                if let Some(episode) = current.as_mut()
                    && tag.name().as_ref() == "enclosure"
                {
                    for attribute in tag.attributes().flatten() {
                        if attribute.key.as_ref() == "url" {
                            episode.audio_url = attribute.normalized_value(XmlVersion::Implicit1_0)?.into_owned();
                        }
                    }
                }
            }
            Event::Text(content) => {
                if current.is_some() && !field.is_empty() {
                    text.push_str(&content.xml10_content());
                }
            }
            Event::CData(content) => {
                if current.is_some() && !field.is_empty() {
                    text.push_str(&content);
                }
            }
            Event::GeneralRef(reference) => {
                if current.is_some() && !field.is_empty() {
                    text.push_str(&resolve_entity(&reference));
                }
            }
            Event::End(tag) => {
                let name = tag.name().as_ref().to_string();
                if name == "item" {
                    if let Some(episode) = current.take()
                        && !episode.audio_url.is_empty()
                    {
                        episodes.push(episode);
                    }
                } else if let Some(episode) = current.as_mut()
                    && name == field
                {
                    assign(episode, &field, text.trim());
                    field.clear();
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(episodes)
}

fn assign(episode: &mut Episode, field: &str, value: &str) {
    match field {
        "guid" => episode.guid = value.to_string(),
        "title" if episode.title.is_empty() => episode.title = value.to_string(),
        "pubDate" => episode.published = DateTime::parse_from_rfc2822(value).ok(),
        "itunes:duration" => episode.duration_s = duration_seconds(value),
        "description" => episode.description = value.to_string(),
        "itunes:summary" if episode.description.is_empty() => episode.description = value.to_string(),
        _ => {}
    }
}

fn resolve_entity(name: &str) -> String {
    match name {
        "amp" => "&".into(),
        "lt" => "<".into(),
        "gt" => ">".into(),
        "quot" => "\"".into(),
        "apos" => "'".into(),
        _ => name
            .strip_prefix("#x")
            .and_then(|hex| u32::from_str_radix(hex, 16).ok())
            .or_else(|| name.strip_prefix('#').and_then(|dec| dec.parse().ok()))
            .and_then(char::from_u32)
            .map(String::from)
            .unwrap_or_else(|| format!("&{name};")),
    }
}

pub fn duration_seconds(text: &str) -> u64 {
    text.trim()
        .split(':')
        .try_fold(0u64, |total, part| part.trim().parse::<f64>().ok().map(|value| total * 60 + value as u64))
        .unwrap_or(0)
}

pub fn slug(text: &str, max: usize) -> String {
    let folded: String = text
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            c if c.is_ascii_alphanumeric() => c,
            _ => '-',
        })
        .collect();
    let joined = folded.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-");
    joined.chars().take(max).collect::<String>().trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
<channel>
  <title>Example Show</title>
  <item>
    <title>Ep. 2 - Lluvia &amp; sol</title>
    <guid>ep-2</guid>
    <pubDate>Mon, 26 Oct 2026 22:03:52 +0000</pubDate>
    <itunes:duration>01:02:03</itunes:duration>
    <description><![CDATA[<p>Conversación sobre el clima</p>]]></description>
    <enclosure url="https://example.com/ep2.mp3?a=1&amp;b=2" type="audio/mpeg" length="1"/>
  </item>
  <item>
    <title>Ep. 1 - Inicio</title>
    <guid>ep-1</guid>
    <pubDate>Mon, 19 Oct 2026 10:00:00 -0300</pubDate>
    <itunes:duration>3600</itunes:duration>
    <itunes:summary>Primer episodio</itunes:summary>
    <enclosure url="https://example.com/ep1.mp3" type="audio/mpeg" length="1"/>
  </item>
  <item>
    <title>Trailer without audio</title>
  </item>
</channel>
</rss>"#;

    #[test]
    fn parses_items_with_audio() {
        let episodes = parse(FEED).unwrap();
        assert_eq!(episodes.len(), 2);
        let first = &episodes[0];
        assert_eq!(first.title, "Ep. 2 - Lluvia & sol");
        assert_eq!(first.guid, "ep-2");
        assert_eq!(first.duration_s, 3723);
        assert_eq!(first.audio_url, "https://example.com/ep2.mp3?a=1&b=2");
        assert_eq!(first.description, "<p>Conversación sobre el clima</p>");
        assert_eq!(episodes[1].description, "Primer episodio");
        assert_eq!(episodes[1].duration_s, 3600);
    }

    #[test]
    fn newest_sorts_by_date() {
        let episodes = newest(parse(FEED).unwrap(), 1);
        assert_eq!(episodes[0].guid, "ep-2");
    }

    #[test]
    fn id_is_date_and_slug() {
        let episodes = parse(FEED).unwrap();
        assert_eq!(episodes[0].id(), "2026-10-26-ep-2-lluvia-sol");
    }
}
