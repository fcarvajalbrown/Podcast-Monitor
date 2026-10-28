use anyhow::{Context, Result};
use serde::Deserialize;

use crate::feed::USER_AGENT;
use crate::shows::SavedShow;

const SEARCH_URL: &str = "https://itunes.apple.com/search";
const COUNTRY: &str = "cl";
const LIMIT: &str = "5";

#[derive(Deserialize)]
struct Results {
    results: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "collectionName", default)]
    name: String,
    #[serde(rename = "artistName", default)]
    publisher: String,
    #[serde(rename = "feedUrl")]
    feed: Option<String>,
}

pub fn search(term: &str) -> Result<Vec<SavedShow>> {
    let body = ureq::get(SEARCH_URL)
        .header("User-Agent", USER_AGENT)
        .query("term", term)
        .query("media", "podcast")
        .query("country", COUNTRY)
        .query("limit", LIMIT)
        .call()
        .with_context(|| format!("searching Apple's directory for {term}"))?
        .body_mut()
        .read_to_string()
        .context("reading the search reply")?;
    parse(&body)
}

pub fn parse(json: &str) -> Result<Vec<SavedShow>> {
    let results: Results = serde_json::from_str(json).context("parsing the search reply")?;
    let mut shows: Vec<SavedShow> = Vec::new();
    for entry in results.results {
        let Some(feed) = entry.feed.filter(|feed| !feed.is_empty()) else { continue };
        if shows.iter().all(|show| show.feed != feed) {
            shows.push(SavedShow { title: entry.name, publisher: entry.publisher, feed });
        }
    }
    Ok(shows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_entries_with_a_feed_once_each() {
        let json = r#"{"resultCount": 4, "results": [
            {"collectionName": "Nada Personal", "artistName": "Radio Duna", "feedUrl": "https://a/feed"},
            {"collectionName": "Sin feed", "artistName": "Editorial"},
            {"collectionName": "Nada Personal", "artistName": "Radio Duna", "feedUrl": "https://a/feed"},
            {"collectionName": "Nada Personal - Podcast", "artistName": "Nada Personal", "feedUrl": "https://b/feed"}
        ]}"#;
        let shows = parse(json).unwrap();
        assert_eq!(shows.iter().map(|show| (show.title.as_str(), show.publisher.as_str(), show.feed.as_str())).collect::<Vec<_>>(), [
            ("Nada Personal", "Radio Duna", "https://a/feed"),
            ("Nada Personal - Podcast", "Nada Personal", "https://b/feed"),
        ]);
    }

    #[test]
    #[ignore = "searches Apple's directory over the network"]
    fn finds_nada_personal_by_radio_duna() {
        let shows = search("nada personal").unwrap();
        assert!(shows.iter().any(|show| show.title == "Nada Personal" && show.publisher == "Radio Duna"));
    }
}
