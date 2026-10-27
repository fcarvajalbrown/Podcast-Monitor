use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

use crate::feed::Episode;
use crate::quotes::Index;
use crate::report::{self, Block};
use crate::run::{ShowInfo, read_json};
use crate::section::EpisodeSection;
use crate::summarize::Summary;
use crate::tally;
use crate::transcribe::Segment;

#[derive(Deserialize)]
struct Segments {
    segments: Vec<Segment>,
}

pub fn episode_folders(root: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        if dir.join("summary.json").is_file() {
            found.push(dir.clone());
        }
        for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

fn show_title(folder: &Path) -> Result<String> {
    let parent = folder.parent().ok_or_else(|| anyhow!("{} has no show folder", folder.display()))?;
    match read_json::<ShowInfo>(&parent.join("show.json"))? {
        Some(show) => Ok(show.title),
        None => {
            let name = parent.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
            eprintln!("  no show.json in {}; using the folder name, run `run` on the feed again to save the title", parent.display());
            Ok(name)
        }
    }
}

pub fn load(folder: &Path) -> Result<Option<EpisodeSection>> {
    let Some(summary) = read_json::<Summary>(&folder.join("summary.json"))? else { return Ok(None) };
    if summary.chunks.len() < summary.chunk_count {
        eprintln!("  {}: summary has {} of {} parts, left out", folder.display(), summary.chunks.len(), summary.chunk_count);
        return Ok(None);
    }
    let episode = read_json::<Episode>(&folder.join("episode.json"))?.ok_or_else(|| anyhow!("{} has no episode.json", folder.display()))?;
    let segments = match read_json::<Segments>(&folder.join("transcript.fixed.json"))? {
        Some(fixed) => fixed.segments,
        None => read_json::<Segments>(&folder.join("transcript.json"))?.ok_or_else(|| anyhow!("{} has no transcript", folder.display()))?.segments,
    };
    let merged = summary.merged();
    let index = Index::new(&segments);
    let mut quotes = Vec::new();
    for quote in &merged.quotes {
        match index.find(&quote.text, &quote.minute) {
            Some(found) => quotes.push(found),
            None => eprintln!("  dropped quote, not in the transcript: {}  {}", quote.minute, quote.text),
        }
    }
    Ok(Some(EpisodeSection {
        show: show_title(folder)?,
        title: episode.title,
        published: episode.published,
        duration_s: episode.duration_s,
        summary: merged.summary,
        topics: tally::airtime(&merged.topics),
        people: tally::mentions(&merged.people, &segments),
        quotes,
    }))
}

pub fn ordered(mut sections: Vec<EpisodeSection>) -> Vec<EpisodeSection> {
    sections.sort_by(|a, b| a.show.to_lowercase().cmp(&b.show.to_lowercase()).then_with(|| b.published.cmp(&a.published)));
    sections
}

pub fn blocks(sections: &[EpisodeSection]) -> Vec<Block> {
    sections.iter().flat_map(|section| std::iter::once(Block::PageBreak).chain(section.blocks())).collect()
}

pub fn build(root: &Path, out: &Path) -> Result<usize> {
    let mut sections = Vec::new();
    for folder in episode_folders(root)? {
        eprintln!("{}", folder.display());
        sections.extend(load(&folder).with_context(|| format!("episode {}", folder.display()))?);
    }
    if sections.is_empty() {
        return Err(anyhow!("no finished summaries under {}", root.display()));
    }
    let sections = ordered(sections);
    fs::write(out, report::pdf(report::HEADER, blocks(&sections))?).with_context(|| format!("writing {}", out.display()))?;
    Ok(sections.len())
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;

    fn section(show: &str, published: Option<&str>) -> EpisodeSection {
        EpisodeSection {
            show: show.into(),
            title: format!("{show} {published:?}"),
            published: published.and_then(|date| DateTime::parse_from_rfc3339(date).ok()),
            duration_s: 0,
            summary: String::new(),
            topics: vec![],
            people: vec![],
            quotes: vec![],
        }
    }

    #[test]
    fn shows_alphabetical_then_newest_first_with_undated_last() {
        let sorted = ordered(vec![
            section("beta", Some("2026-01-01T00:00:00Z")),
            section("Alfa", None),
            section("Alfa", Some("2026-01-01T00:00:00Z")),
            section("Alfa", Some("2026-03-01T00:00:00Z")),
        ]);
        let titles: Vec<&str> = sorted.iter().map(|section| section.title.as_str()).collect();
        assert_eq!(titles, vec![
            "Alfa Some(\"2026-03-01T00:00:00Z\")",
            "Alfa Some(\"2026-01-01T00:00:00Z\")",
            "Alfa None",
            "beta Some(\"2026-01-01T00:00:00Z\")",
        ]);
    }

    #[test]
    fn every_episode_opens_with_a_page_break() {
        let blocks = blocks(&[section("Alfa", None), section("Beta", None)]);
        assert_eq!(blocks.iter().filter(|block| **block == Block::PageBreak).count(), 2);
        assert_eq!(blocks[0], Block::PageBreak);
    }

    #[test]
    fn unfinished_summaries_are_left_out_and_finished_ones_loaded() {
        let root = std::env::temp_dir().join(format!("podcast-monitor-assemble-{}", std::process::id()));
        let folder = root.join("programa").join("episodio");
        fs::create_dir_all(&folder).unwrap();
        fs::write(root.join("programa").join("show.json"), r#"{"title":"Programa"}"#).unwrap();
        fs::write(folder.join("episode.json"), r#"{"guid":"g","title":"Episodio","published":"2026-10-26T15:00:00Z","duration_s":120,"audio_url":"","description":""}"#).unwrap();
        fs::write(folder.join("transcript.json"), r#"{"segments":[{"start_s":5.0,"end_s":9.0,"text":" Hola a todos, habla Ana Pérez."}]}"#).unwrap();
        let reply = r#"{"summary":"Resumen.","topics":[{"name":"Saludo","start":"00:00","end":"00:09"}],"people":["Ana Pérez"],"quotes":[{"text":"hola a todos","minute":"00:05"},{"text":"no existe","minute":"00:01"}]}"#;
        fs::write(folder.join("summary.json"), format!(r#"{{"provider":"claude","chunk_count":2,"chunks":[{reply}]}}"#)).unwrap();
        assert_eq!(episode_folders(&root).unwrap(), vec![folder.clone()]);
        assert!(load(&folder).unwrap().is_none());
        fs::write(folder.join("summary.json"), format!(r#"{{"provider":"claude","chunk_count":1,"chunks":[{reply}]}}"#)).unwrap();
        let section = load(&folder).unwrap().unwrap();
        assert_eq!(section.show, "Programa");
        assert_eq!(section.people[0].count, 1);
        assert_eq!(section.quotes.len(), 1);
        assert_eq!(section.quotes[0].start_s, 5.0);
        let out = root.join("informe.pdf");
        assert_eq!(build(&root, &out).unwrap(), 1);
        assert!(fs::read(&out).unwrap().starts_with(b"%PDF"));
        fs::remove_dir_all(&root).unwrap();
    }
}
