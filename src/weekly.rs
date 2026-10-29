use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::feed::Episode;
use crate::job::{Choice, Failure, Request, episode_folder};
use crate::report::{self, Block};
use crate::shows::SavedShow;
use crate::summarize::LimitReached;
use crate::week::Week;
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
    pub themes: Vec<(String, f64)>,
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

pub fn bare_label(label: &str) -> &str {
    let label = label.trim();
    let Some(open) = label.rfind(" (") else { return label };
    let inside = &label[open + 2..];
    let clock = inside.strip_suffix(')').is_some_and(|time| !time.is_empty() && time.chars().all(|c| c.is_ascii_digit() || c == ':'));
    if clock { label[..open].trim_end() } else { label }
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

fn grouped_seconds(labelled: &[(String, f64)], groups: &[Group]) -> Vec<(String, f64)> {
    let seconds = |label: &str| -> f64 { labelled.iter().filter(|(name, _)| name == label).map(|(_, seconds)| seconds).sum() };
    let mut placed: Vec<String> = Vec::new();
    let mut sums: Vec<(String, f64)> = Vec::new();
    for group in groups {
        let mut sum = 0.0;
        for label in &group.topics {
            let label = bare_label(label).to_string();
            if !placed.contains(&label) {
                sum += seconds(&label);
                placed.push(label);
            }
        }
        sums.push((group.name.trim().to_string(), sum));
    }
    for (label, _) in labelled {
        if !placed.contains(label) {
            log::warn!("topic left out of every group: {label}");
        }
    }
    sums.sort_by(|a, b| b.1.total_cmp(&a.1));
    sums.into_iter().filter(|(_, sum)| *sum > 0.0).collect()
}

fn percents(sums: Vec<(String, f64)>, total_s: u64) -> Vec<(String, u32)> {
    sums.into_iter().map(|(name, sum)| (name, if total_s == 0 { 0 } else { (100.0 * sum / total_s as f64).round() as u32 })).collect()
}

pub fn show_themes(episodes: &[EpisodeDigest], groups: &[Group]) -> Vec<(String, f64)> {
    let labelled: Vec<(String, f64)> = episodes.iter().enumerate().flat_map(|(index, episode)| episode.topics.iter().map(move |topic| (topic_label(index, &topic.name), topic.seconds))).collect();
    grouped_seconds(&labelled, groups)
}

pub fn shares(episodes: &[EpisodeDigest], groups: &[Group]) -> Vec<(String, u32)> {
    percents(show_themes(episodes, groups), episodes.iter().map(|episode| episode.duration_s).sum())
}

pub fn theme_label(show: usize, name: &str) -> String {
    format!("P{}: {}", show + 1, name)
}

fn show_labels(blocks: &[ShowBlock]) -> Vec<(String, f64)> {
    blocks.iter().enumerate().flat_map(|(index, block)| block.themes.iter().map(move |(name, seconds)| (theme_label(index, name), *seconds))).collect()
}

pub fn week_shares(blocks: &[ShowBlock], groups: &[Group]) -> Vec<(String, u32)> {
    percents(grouped_seconds(&show_labels(blocks), groups), blocks.iter().map(|block| block.duration_s).sum())
}

pub fn themes_schema() -> Value {
    let text = json!({ "type": "string" });
    json!({
        "type": "object",
        "properties": {
            "groups": { "type": "array", "items": {
                "type": "object",
                "properties": { "name": text, "topics": { "type": "array", "items": text } },
                "required": ["name", "topics"],
                "additionalProperties": false,
            } },
        },
        "required": ["groups"],
        "additionalProperties": false,
    })
}

pub fn themes_prompt(blocks: &[ShowBlock]) -> String {
    let mut text = String::from(
        "These are the themes that several Spanish-language podcasts covered this week, show by show. Every theme has a label and its airtime as mm:ss.\n\
         \n\
         Return:\n\
         - groups: between 5 and 8 broad themes of the whole week across all the shows, in Spanish, each with a short name of at most six words and the labels of the themes it gathers, copied exactly. Gather the same story told by different shows into one group. Every label goes in exactly one group.\n",
    );
    for (index, block) in blocks.iter().enumerate() {
        text.push_str(&format!("\nShow {} ({}):\n", index + 1, block.show));
        for (name, seconds) in &block.themes {
            text.push_str(&format!("- {} ({})\n", theme_label(index, name), crate::chunk::clock(*seconds)));
        }
    }
    text
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedThemes {
    pub provider: Provider,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub labels: Vec<String>,
    pub groups: Vec<Group>,
}

#[derive(Deserialize)]
struct ThemesReply {
    groups: Vec<Group>,
}

pub fn summarize_themes(folder: &Path, blocks: &[ShowBlock], provider: Provider, assistant: &dyn Assistant) -> Result<SavedThemes> {
    let path = folder.join("themes.json");
    let labels: Vec<String> = show_labels(blocks).into_iter().map(|(label, _)| label).collect();
    if let Some(saved) = read_json::<SavedThemes>(&path)?.filter(|saved| saved.labels == labels) {
        log::info!("week themes already done");
        return Ok(saved);
    }
    log::info!("grouping the week's themes across {} show(s)", blocks.len());
    let reply = assistant.ask(&themes_prompt(blocks), &themes_schema())?;
    let reply: ThemesReply = serde_json::from_value(reply).context("reading the assistant's themes reply")?;
    let saved = SavedThemes { provider, model: assistant.model().map(String::from), labels, groups: reply.groups };
    write_json(&path, &saved)?;
    Ok(saved)
}

pub fn share_table(shares: &[(String, u32)]) -> Block {
    let mut rows = vec![vec!["Tema".to_string(), "% del tiempo".to_string()]];
    rows.extend(shares.iter().map(|(name, percent)| vec![name.clone(), if *percent == 0 { "<1 %".into() } else { format!("{percent} %") }]));
    Block::Table(rows)
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
        themes: show_themes(episodes, &week.reply.groups),
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
        let mut days: Vec<String> = dated.iter().map(|date| format!("{} {}", WEEKDAYS[date.weekday().num_days_from_monday() as usize], date.day())).collect();
        days.dedup();
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
        blocks
    }
}

pub const WEEKLY_FILE: &str = "informe-semanal.pdf";

pub fn assistant_name(provider: Provider, model: Option<&str>) -> String {
    let base = match provider {
        Provider::Claude => "Claude",
        Provider::Codex => "Codex",
    };
    match model {
        Some(crate::summarize::FALLBACK_MODEL) => format!("{base} Sonnet 5"),
        Some(model) => format!("{base} ({model})"),
        None => base.into(),
    }
}

pub fn closing(blocks: &[ShowBlock], quiet: &[String]) -> Vec<Block> {
    let mut lines = Vec::new();
    if !quiet.is_empty() {
        lines.push(format!("Sin episodios esta semana: {}.", spoken_list(quiet)));
    }
    let mut counts: Vec<(String, usize)> = Vec::new();
    for block in blocks {
        for (provider, model) in &block.assistants {
            let name = assistant_name(*provider, model.as_deref());
            match counts.iter_mut().find(|(seen, _)| *seen == name) {
                Some((_, count)) => *count += 1,
                None => counts.push((name, 1)),
            }
        }
    }
    let shows = |count: usize| if count == 1 { "1 programa".to_string() } else { format!("{count} programas") };
    match counts.as_slice() {
        [] => {}
        [(name, _)] => lines.push(format!("Resúmenes hechos con {name}.")),
        many => lines.push(format!("Resúmenes hechos con {}.", spoken_list(&many.iter().map(|(name, count)| format!("{name} ({})", shows(*count))).collect::<Vec<_>>()))),
    }
    lines.into_iter().map(Block::Paragraph).collect()
}

pub fn title(monday: NaiveDate) -> String {
    format!("Monitoreo de podcasts: semana del {}", crate::section::spanish_date(monday))
}

#[derive(Debug, Default)]
pub struct WeeklyOutcome {
    pub pdf: Option<PathBuf>,
    pub shows: usize,
    pub missing: Vec<String>,
    pub stopped: Option<Failure>,
}

fn limited(error: &anyhow::Error) -> bool {
    error.downcast_ref::<LimitReached>().is_some()
}

pub fn build(data: &Path, week: &Week, saved: &[SavedShow], first: Choice, mut fallback: Option<Choice>) -> Result<WeeklyOutcome> {
    let (mut provider, mut assistant) = first;
    let mut outcome = WeeklyOutcome::default();
    let mut blocks = Vec::new();
    let mut quiet = Vec::new();
    let mut shows: Vec<&SavedShow> = saved.iter().collect();
    shows.sort_by_key(|show| show.title.to_lowercase());
    for show in shows {
        let requests: Vec<&Request> = week.requests.iter().filter(|request| request.show.feed == show.feed).collect();
        let Some(first_request) = requests.first() else {
            quiet.push(show.title.clone());
            continue;
        };
        let mut digests = Vec::new();
        for request in &requests {
            match digest(&episode_folder(data, request))? {
                Some(found) => digests.push(found),
                None => outcome.missing.push(format!("{} · {}", show.title, request.episode.title)),
            }
        }
        if digests.is_empty() {
            continue;
        }
        let show_dir = episode_folder(data, first_request).parent().map(Path::to_path_buf).unwrap_or_else(|| data.to_path_buf());
        let mut result = summarize_week(&show_dir, week.monday, &show.title, &digests, provider, assistant);
        if result.as_ref().is_err_and(limited)
            && let Some((next, next_assistant)) = fallback.take()
        {
            log::warn!("{provider:?} hit its limit; switching to {next:?} for the rest of the week summaries");
            (provider, assistant) = (next, next_assistant);
            result = summarize_week(&show_dir, week.monday, &show.title, &digests, provider, assistant);
        }
        match result {
            Ok(saved_week) => blocks.push(block(&show.title, &show.publisher, &digests, &saved_week)),
            Err(error) if limited(&error) => {
                log::error!("week summary of {}: {error:#}", show.title);
                outcome.stopped = Some(Failure::Limit);
                return Ok(outcome);
            }
            Err(error) => return Err(error.context(format!("week summary of {}", show.title))),
        }
    }
    for missing in &outcome.missing {
        log::warn!("not finished, left out: {missing}");
    }
    if blocks.is_empty() {
        return Err(anyhow!("no finished episodes this week"));
    }
    let folder = data.join("weeks").join(week.monday.to_string());
    std::fs::create_dir_all(&folder).with_context(|| format!("creating {}", folder.display()))?;
    let mut themes = summarize_themes(&folder, &blocks, provider, assistant);
    if themes.as_ref().is_err_and(limited)
        && let Some((next, next_assistant)) = fallback.take()
    {
        log::warn!("{provider:?} hit its limit; switching to {next:?} for the week themes");
        (provider, assistant) = (next, next_assistant);
        themes = summarize_themes(&folder, &blocks, provider, assistant);
    }
    let themes = match themes {
        Ok(themes) => themes,
        Err(error) if limited(&error) => {
            log::error!("week themes: {error:#}");
            outcome.stopped = Some(Failure::Limit);
            return Ok(outcome);
        }
        Err(error) => return Err(error.context("week themes")),
    };
    let mut pages = vec![Block::Heading { level: 1, text: title(week.monday) }];
    let shares = week_shares(&blocks, &themes.groups);
    if !shares.is_empty() {
        pages.push(Block::Heading { level: 2, text: "Temas de la semana".into() });
        pages.push(share_table(&shares));
    }
    pages.extend(blocks.iter().flat_map(ShowBlock::blocks));
    pages.extend(closing(&blocks, &quiet));
    let pdf = folder.join(WEEKLY_FILE);
    std::fs::write(&pdf, report::pdf(report::HEADER, pages)?).with_context(|| format!("writing {}", pdf.display()))?;
    log::info!("weekly report with {} show(s) written to {}", blocks.len(), pdf.display());
    outcome.shows = blocks.len();
    outcome.pdf = Some(pdf);
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_lists_quiet_shows_and_the_assistants() {
        let saved = SavedWeek { provider: Provider::Codex, model: None, episodes: vec![], reply: WeekReply { summary: String::new(), groups: vec![] } };
        let mut first = block("A", "", &week(), &saved);
        let second = ShowBlock { assistants: vec![(Provider::Claude, Some("claude-sonnet-5".into()))], ..first.clone() };
        assert_eq!(closing(std::slice::from_ref(&first), &[]), [Block::Paragraph("Resúmenes hechos con Codex.".into())]);
        first.assistants.push((Provider::Claude, Some("claude-sonnet-5".into())));
        assert_eq!(closing(&[first, second], &["Lo Que Importa".into(), "Sin Llorar".into()]), [
            Block::Paragraph("Sin episodios esta semana: Lo Que Importa y Sin Llorar.".into()),
            Block::Paragraph("Resúmenes hechos con Codex (1 programa) y Claude Sonnet 5 (2 programas).".into()),
        ]);
    }

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
    fn labels_copied_with_their_airtime_still_match() {
        let groups = vec![group("Decretos", &["E2: Decretos (15:00)", "E1: Lula (05:00)"]), group("Resto", &[" E2: Brasil (1:03:20) ", "E1: Cierre (sin hora)"])];
        assert_eq!(shares(&week(), &groups), [("Decretos".to_string(), 60), ("Resto".to_string(), 10)]);
        assert_eq!(bare_label("E1: Lula (05:00)"), "E1: Lula");
        assert_eq!(bare_label("E1: Cierre (sin hora)"), "E1: Cierre (sin hora)");
    }

    #[test]
    fn two_episodes_on_one_day_name_the_day_once() {
        let mut episodes = week();
        episodes.insert(0, digest("c", "2026-10-26T18:00:00-03:00", 600, &[]));
        let saved = SavedWeek { provider: Provider::Codex, model: None, episodes: vec![], reply: WeekReply { summary: String::new(), groups: vec![] } };
        let block = block("Programa", "Radio", &episodes, &saved);
        assert_eq!(block.blocks()[1], Block::Paragraph("Radio. 3 episodios (lunes 26 y martes 27 de octubre), 43 min.".into()));
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
        ]);
        assert_eq!(block.themes, [("Decretos".to_string(), 900.0), ("Cierre".to_string(), 50.0)]);
    }

    #[test]
    fn week_shares_merge_show_themes_over_all_airtime() {
        let saved = SavedWeek { provider: Provider::Codex, model: None, episodes: vec![], reply: WeekReply { summary: String::new(), groups: vec![group("Decretos", &["E2: Decretos"]), group("Brasil", &["E1: Lula", "E2: Brasil"])] } };
        let first = block("A", "", &week(), &saved);
        let second = ShowBlock { show: "B".into(), duration_s: 2000, themes: vec![("Gobierno".into(), 500.0), ("Brasil y Lula".into(), 100.0)], ..first.clone() };
        let blocks = [first, second];
        let groups = vec![group("Decretos del gobierno", &["P1: Decretos (15:00)", "P2: Gobierno"]), group("Brasil", &["P1: Brasil", "P2: Brasil y Lula"])];
        assert_eq!(week_shares(&blocks, &groups), [("Decretos del gobierno".to_string(), 35), ("Brasil".to_string(), 15)]);
        assert!(themes_prompt(&blocks).contains("Show 2 (B):\n- P2: Gobierno (08:20)"));
        assert_eq!(share_table(&[("Brasil".into(), 0)]), Block::Table(vec![vec!["Tema".into(), "% del tiempo".into()], vec!["Brasil".into(), "<1 %".into()]]));
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
