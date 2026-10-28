use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::feed::Episode;
use crate::report::Block;
use crate::run::{read_json, write_json};
use crate::section::spanish_duration;
use crate::summarize::{Assistant, Provider, Summary};
use crate::tally::{self, Airtime};

const WEEKDAYS: [&str; 7] = ["lunes", "martes", "miércoles", "jueves", "viernes", "sábado", "domingo"];
const MONTHS: [&str; 12] = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeDigest {
    pub id: String,
    pub title: String,
    pub published: Option<DateTime<FixedOffset>>,
    pub duration_s: u64,
    pub summary: String,
    pub topics: Vec<Airtime>,
    pub provider: Provider,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeekReply {
    pub summary: String,
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedWeek {
    pub provider: Provider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub episodes: Vec<String>,
    pub reply: WeekReply,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShowBlock {
    pub show: String,
    pub publisher: String,
    pub episodes: Vec<Option<DateTime<FixedOffset>>>,
    pub duration_s: u64,
    pub summary: String,
    pub shares: Vec<(String, u32)>,
    pub assistants: Vec<(Provider, Option<String>)>,
}

pub fn digest(folder: &Path) -> Result<Option<EpisodeDigest>> {
    let Some(summary) = read_json::<Summary>(&folder.join("summary.json"))? else { return Ok(None) };
    if summary.chunks.len() < summary.chunk_count {
        return Ok(None);
    }
    let episode = read_json::<Episode>(&folder.join("episode.json"))?.ok_or_else(|| anyhow!("{} has no episode.json", folder.display()))?;
    let merged = summary.merged();
    Ok(Some(EpisodeDigest {
        id: episode.id(),
        title: episode.title,
        published: episode.published,
        duration_s: episode.duration_s,
        summary: merged.summary,
        topics: tally::airtime(&merged.topics),
        provider: summary.provider,
        model: summary.model,
    }))
}

pub fn topic_label(episode: usize, name: &str) -> String {
    format!("E{}: {}", episode + 1, name)
}

pub fn day(date: &DateTime<FixedOffset>) -> String {
    let local = date.with_timezone(&Local).date_naive();
    format!("{} {}", WEEKDAYS[local.weekday().num_days_from_monday() as usize], local.day())
}

pub fn schema() -> Value {
    let text = json!({ "type": "string" });
    json!({
        "type": "object",
        "properties": {
            "summary": text,
            "groups": { "type": "array", "items": {
                "type": "object",
                "properties": { "name": text, "topics": { "type": "array", "items": text } },
                "required": ["name", "topics"],
                "additionalProperties": false,
            } },
        },
        "required": ["summary", "groups"],
        "additionalProperties": false,
    })
}

pub fn prompt(show: &str, episodes: &[EpisodeDigest]) -> String {
    let mut text = format!(
        "These are the summaries and topics of the episodes the Spanish-language podcast «{show}» published this week. Every topic has a label and its airtime as mm:ss.\n\
         \n\
         Return:\n\
         - summary: three or four sentences in Spanish, at most 80 words, on what the show talked about this week as a whole. Describe what was said and who said it, without judging it.\n\
         - groups: between 5 and 8 broad themes in Spanish, each with a short name of at most six words and the labels of the topics it gathers, copied exactly. Every label goes in exactly one group. With fewer than 5 topics, one group per topic.\n"
    );
    for (index, episode) in episodes.iter().enumerate() {
        let when = episode.published.as_ref().map(day).unwrap_or_default();
        text.push_str(&format!("\nEpisode {} ({when}, {}): {}\nSummary: {}\nTopics:\n", index + 1, spanish_duration(episode.duration_s), episode.title, episode.summary.trim()));
        for topic in &episode.topics {
            text.push_str(&format!("- {} ({})\n", topic_label(index, &topic.name), crate::chunk::clock(topic.seconds)));
        }
    }
    text
}

pub fn shares(episodes: &[EpisodeDigest], groups: &[Group]) -> Vec<(String, u32)> {
    let total: u64 = episodes.iter().map(|episode| episode.duration_s).sum();
    let seconds = |label: &str| -> f64 {
        episodes.iter().enumerate().flat_map(|(index, episode)| episode.topics.iter().map(move |topic| (topic_label(index, &topic.name), topic.seconds))).filter(|(name, _)| name == label).map(|(_, seconds)| seconds).sum()
    };
    let mut placed: Vec<String> = Vec::new();
    let mut shares: Vec<(String, f64)> = Vec::new();
    for group in groups {
        let mut sum = 0.0;
        for label in &group.topics {
            if !placed.contains(label) {
                placed.push(label.clone());
                sum += seconds(label);
            }
        }
        shares.push((group.name.trim().to_string(), sum));
    }
    for (index, episode) in episodes.iter().enumerate() {
        for topic in &episode.topics {
            if !placed.contains(&topic_label(index, &topic.name)) {
                log::warn!("topic left out of every group: {}", topic_label(index, &topic.name));
            }
        }
    }
    shares.sort_by(|a, b| b.1.total_cmp(&a.1));
    shares.into_iter().filter(|(_, sum)| *sum > 0.0).map(|(name, sum)| (name, if total == 0 { 0 } else { (100.0 * sum / total as f64).round() as u32 })).collect()
}

pub fn week_path(show_dir: &Path, monday: NaiveDate) -> PathBuf {
    show_dir.join(format!("week-{monday}.json"))
}

pub fn summarize_week(show_dir: &Path, monday: NaiveDate, show: &str, episodes: &[EpisodeDigest], provider: Provider, assistant: &dyn Assistant) -> Result<SavedWeek> {
    let path = week_path(show_dir, monday);
    let ids: Vec<String> = episodes.iter().map(|episode| episode.id.clone()).collect();
    if let Some(saved) = read_json::<SavedWeek>(&path)?.filter(|saved| saved.episodes == ids) {
        log::info!("  week summary already done for {show}");
        return Ok(saved);
    }
    log::info!("  summarizing the week of {show}, {} episode(s)", episodes.len());
    let reply = assistant.ask(&prompt(show, episodes), &schema())?;
    let reply: WeekReply = serde_json::from_value(reply).context("reading the assistant's week reply")?;
    let saved = SavedWeek { provider, model: assistant.model().map(String::from), episodes: ids, reply };
    write_json(&path, &saved)?;
    Ok(saved)
}

pub fn block(show: &str, publisher: &str, episodes: &[EpisodeDigest], week: &SavedWeek) -> ShowBlock {
    let mut assistants: Vec<(Provider, Option<String>)> = episodes.iter().map(|episode| (episode.provider, episode.model.clone())).collect();
    assistants.push((week.provider, week.model.clone()));
    assistants.dedup();
    ShowBlock {
        show: show.into(),
        publisher: publisher.into(),
        episodes: episodes.iter().map(|episode| episode.published).collect(),
        duration_s: episodes.iter().map(|episode| episode.duration_s).sum(),
        summary: week.reply.summary.trim().into(),
        shares: shares(episodes, &week.reply.groups),
        assistants,
    }
}

fn spoken_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [one] => one.clone(),
        [rest @ .., last] => format!("{} y {last}", rest.join(", ")),
    }
}

impl ShowBlock {
    fn details(&self) -> String {
        let mut dated: Vec<DateTime<Local>> = self.episodes.iter().flatten().map(|date| date.with_timezone(&Local)).collect();
        dated.sort();
        let days: Vec<String> = dated.iter().map(|date| format!("{} {}", WEEKDAYS[date.weekday().num_days_from_monday() as usize], date.day())).collect();
        let month = dated.last().map(|date| format!(" de {}", MONTHS[date.month0() as usize])).unwrap_or_default();
        let count = match self.episodes.len() {
            1 => "1 episodio".to_string(),
            many => format!("{many} episodios"),
        };
        let days = if days.is_empty() { String::new() } else { format!(" ({}{month})", spoken_list(&days)) };
        let publisher = if self.publisher.is_empty() { String::new() } else { format!("{}. ", self.publisher) };
        format!("{publisher}{count}{days}, {}.", spanish_duration(self.duration_s))
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut blocks = vec![Block::Heading { level: 2, text: self.show.clone() }, Block::Paragraph(self.details())];
        if !self.summary.is_empty() {
            blocks.push(Block::Paragraph(self.summary.clone()));
        }
        if !self.shares.is_empty() {
            let mut rows = vec![vec!["Tema".to_string(), "% del tiempo".to_string()]];
            rows.extend(self.shares.iter().map(|(name, percent)| vec![name.clone(), if *percent == 0 { "<1 %".into() } else { format!("{percent} %") }]));
            blocks.push(Block::Table(rows));
        }
        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(id: &str, published: &str, duration_s: u64, topics: &[(&str, f64)]) -> EpisodeDigest {
        EpisodeDigest {
            id: id.into(),
            title: format!("Episodio {id}"),
            published: DateTime::parse_from_rfc3339(published).ok(),
            duration_s,
            summary: "Resumen.".into(),
            topics: topics.iter().map(|(name, seconds)| Airtime { name: (*name).into(), seconds: *seconds }).collect(),
            provider: Provider::Codex,
            model: None,
        }
    }

    fn week() -> Vec<EpisodeDigest> {
        vec![
            digest("a", "2026-10-26T12:00:00-03:00", 1000, &[("Lula", 300.0), ("Cierre", 50.0)]),
            digest("b", "2026-10-27T12:00:00-03:00", 1000, &[("Brasil", 200.0), ("Decretos", 900.0)]),
        ]
    }

    fn group(name: &str, topics: &[&str]) -> Group {
        Group { name: name.into(), topics: topics.iter().map(|topic| topic.to_string()).collect() }
    }

    #[test]
    fn shares_are_percent_of_the_week_largest_first_each_topic_counted_once() {
        let groups = vec![group("Brasil", &["E1: Lula", "E2: Brasil"]), group("Decretos", &["E2: Decretos", "E1: Lula"]), group("Cierre", &["E1: Cierre"]), group("Vacío", &["E9: nada"])];
        assert_eq!(shares(&week(), &groups), [("Decretos".to_string(), 45), ("Brasil".to_string(), 25), ("Cierre".to_string(), 3)]);
    }

    #[test]
    fn prompt_labels_topics_by_episode() {
        let text = prompt("Programa", &week());
        assert!(text.contains("Episode 2 (martes 27, 17 min): Episodio b"));
        assert!(text.contains("- E2: Decretos (15:00)"));
    }

    #[test]
    fn block_reads_in_spanish_with_days_and_percent() {
        let saved = SavedWeek { provider: Provider::Codex, model: None, episodes: vec!["a".into(), "b".into()], reply: WeekReply { summary: " La semana. ".into(), groups: vec![group("Decretos", &["E2: Decretos"]), group("Cierre", &["E1: Cierre"])] } };
        let block = block("Programa", "Radio", &week(), &saved);
        assert_eq!(block.assistants, [(Provider::Codex, None)]);
        assert_eq!(block.blocks(), vec![
            Block::Heading { level: 2, text: "Programa".into() },
            Block::Paragraph("Radio. 2 episodios (lunes 26 y martes 27 de octubre), 33 min.".into()),
            Block::Paragraph("La semana.".into()),
            Block::Table(vec![vec!["Tema".into(), "% del tiempo".into()], vec!["Decretos".into(), "45 %".into()], vec!["Cierre".into(), "3 %".into()]]),
        ]);
    }

    #[test]
    fn saved_week_is_reused_only_for_the_same_episodes() {
        struct Counting(std::cell::Cell<usize>);
        impl Assistant for Counting {
            fn ask(&self, _prompt: &str, _schema: &Value) -> Result<Value> {
                self.0.set(self.0.get() + 1);
                Ok(json!({ "summary": "S", "groups": [] }))
            }
        }
        let dir = std::env::temp_dir().join(format!("podcast-monitor-weekly-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let monday = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
        let assistant = Counting(std::cell::Cell::new(0));
        summarize_week(&dir, monday, "Programa", &week(), Provider::Codex, &assistant).unwrap();
        summarize_week(&dir, monday, "Programa", &week(), Provider::Codex, &assistant).unwrap();
        assert_eq!(assistant.0.get(), 1);
        summarize_week(&dir, monday, "Programa", &week()[..1], Provider::Codex, &assistant).unwrap();
        assert_eq!(assistant.0.get(), 2);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
