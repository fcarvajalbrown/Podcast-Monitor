use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::run::{read_json, write_json};

const APP_FOLDER: &str = "Podcast-Monitor";
const LIST_FILE: &str = "shows.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedShow {
    pub title: String,
    pub publisher: String,
    pub feed: String,
}

pub fn data_dir() -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    } else {
        std::env::var_os("APPDATA").map(PathBuf::from)
    };
    base.unwrap_or_else(|| PathBuf::from("work")).join(APP_FOLDER)
}

pub fn list_path(data_dir: &Path) -> PathBuf {
    data_dir.join(LIST_FILE)
}

pub fn load(path: &Path) -> Result<Vec<SavedShow>> {
    Ok(read_json(path)?.unwrap_or_default())
}

pub fn save(path: &Path, shows: &[SavedShow]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    write_json(path, &shows)
}

pub fn add(shows: &mut Vec<SavedShow>, show: SavedShow) -> bool {
    if shows.iter().any(|saved| saved.feed == show.feed) {
        return false;
    }
    shows.push(show);
    shows.sort_by_key(|saved| saved.title.to_lowercase());
    true
}

pub fn remove(shows: &mut Vec<SavedShow>, feed: &str) {
    shows.retain(|saved| saved.feed != feed);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn show(title: &str, feed: &str) -> SavedShow {
        SavedShow { title: title.into(), publisher: "Editorial".into(), feed: feed.into() }
    }

    #[test]
    fn add_keeps_titles_sorted_and_skips_a_saved_feed() {
        let mut shows = Vec::new();
        assert!(add(&mut shows, show("Nada Personal", "https://a/feed")));
        assert!(add(&mut shows, show("animales", "https://b/feed")));
        assert!(!add(&mut shows, show("Otro nombre", "https://a/feed")));
        assert_eq!(shows.iter().map(|saved| saved.title.as_str()).collect::<Vec<_>>(), ["animales", "Nada Personal"]);
        remove(&mut shows, "https://b/feed");
        assert_eq!(shows, [show("Nada Personal", "https://a/feed")]);
    }

    #[test]
    fn list_survives_a_save_and_load() {
        let dir = std::env::temp_dir().join(format!("podcast-monitor-shows-{}", std::process::id()));
        let path = list_path(&dir.join("nested"));
        assert!(load(&path).unwrap().is_empty());
        let shows = vec![show("Nada Personal", "https://a/feed")];
        save(&path, &shows).unwrap();
        assert_eq!(load(&path).unwrap(), shows);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
