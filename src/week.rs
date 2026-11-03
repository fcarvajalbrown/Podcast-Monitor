use chrono::{DateTime, Datelike, Days, FixedOffset, Local, NaiveDate, TimeDelta, TimeZone};
use serde::{Deserialize, Serialize};

use crate::feed::{self, Episode};
use crate::job::Request;
use crate::shows::SavedShow;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Span {
    Calendar { monday: NaiveDate, start: DateTime<FixedOffset>, end: DateTime<FixedOffset> },
    LastSevenDays { start: DateTime<FixedOffset>, end: DateTime<FixedOffset> },
}

impl Span {
    pub fn last_seven_days<Tz: TimeZone>(now: &DateTime<Tz>) -> Span {
        let end = now.fixed_offset();
        Span::LastSevenDays { start: end - TimeDelta::days(7), end }
    }

    pub fn calendar<Tz: TimeZone>(monday: NaiveDate, end: &DateTime<Tz>) -> anyhow::Result<Span> {
        checked_monday(monday)?;
        let midnight = monday.and_hms_opt(0, 0, 0).and_then(|time| end.timezone().from_local_datetime(&time).earliest());
        let start = midnight.ok_or_else(|| anyhow::anyhow!("no local midnight on {monday}"))?;
        Ok(Span::Calendar { monday, start: start.fixed_offset(), end: end.fixed_offset() })
    }

    pub fn start(&self) -> DateTime<FixedOffset> {
        match self {
            Span::Calendar { start, .. } | Span::LastSevenDays { start, .. } => *start,
        }
    }

    pub fn end(&self) -> DateTime<FixedOffset> {
        match self {
            Span::Calendar { end, .. } | Span::LastSevenDays { end, .. } => *end,
        }
    }

    pub fn key(&self) -> String {
        match self {
            Span::Calendar { monday, .. } => monday.to_string(),
            Span::LastSevenDays { end, .. } => format!("7d-to-{}", end.date_naive()),
        }
    }

    pub fn contains(&self, episode: &Episode) -> bool {
        episode.published.is_some_and(|published| published >= self.start() && published <= self.end())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Week {
    pub span: Span,
    pub requests: Vec<Request>,
    pub failed: Vec<SavedShow>,
}

pub fn checked_monday(start: NaiveDate) -> anyhow::Result<NaiveDate> {
    if start.weekday() != chrono::Weekday::Mon {
        anyhow::bail!("{start} is a {}, not a Monday", start.weekday());
    }
    Ok(start)
}

pub fn end_of(start: NaiveDate, now: DateTime<Local>) -> anyhow::Result<DateTime<Local>> {
    checked_monday(start)?;
    let sunday_night = (start + Days::new(6)).and_hms_opt(23, 59, 59).and_then(|time| time.and_local_timezone(Local).earliest());
    let end = sunday_night.ok_or_else(|| anyhow::anyhow!("no local time for the Sunday after {start}"))?;
    Ok(end.min(now))
}

pub fn pick(show: &SavedShow, episodes: Vec<Episode>, span: &Span) -> Vec<Request> {
    let mut picked: Vec<Episode> = episodes.into_iter().filter(|episode| span.contains(episode)).collect();
    picked.sort_by(|a, b| b.published.cmp(&a.published));
    picked.into_iter().map(|episode| Request { show: show.clone(), episode }).collect()
}

pub fn collect(shows: &[SavedShow], span: Span) -> Week {
    let mut sorted = shows.to_vec();
    sorted.sort_by_key(|show| show.title.to_lowercase());
    let mut week = Week { span, requests: Vec::new(), failed: Vec::new() };
    for show in sorted {
        match feed::fetch(&show.feed) {
            Ok(fetched) => {
                let picked = pick(&show, fetched.episodes, &span);
                log::debug!("{}: {} episode(s) since {}", show.title, picked.len(), span.start());
                week.requests.extend(picked);
            }
            Err(error) => {
                log::warn!("listing episodes of {}: {error:#}", show.title);
                week.failed.push(show);
            }
        }
    }
    week
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chile() -> FixedOffset {
        FixedOffset::west_opt(3 * 3600).unwrap()
    }

    fn at(day: u32, hour: u32) -> DateTime<FixedOffset> {
        chile().with_ymd_and_hms(2026, 10, day, hour, 0, 0).unwrap()
    }

    fn episode(guid: &str, published: Option<DateTime<FixedOffset>>) -> Episode {
        Episode { guid: guid.into(), title: guid.into(), published, duration_s: 60, audio_url: String::new(), description: String::new() }
    }

    fn show() -> SavedShow {
        SavedShow { title: "Programa".into(), publisher: "Radio".into(), feed: "https://example.com/feed".into() }
    }

    fn guids(requests: Vec<Request>) -> Vec<String> {
        requests.into_iter().map(|request| request.episode.guid).collect()
    }

    #[test]
    fn end_of_a_past_week_is_its_sunday_night() {
        let now = Local.with_ymd_and_hms(2026, 11, 2, 0, 43, 0).unwrap();
        let end = end_of(NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(), now).unwrap();
        assert_eq!(end, Local.with_ymd_and_hms(2026, 11, 1, 23, 59, 59).unwrap());
        assert!(end_of(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap(), now).is_err());
    }

    #[test]
    fn a_past_monday_to_now_keeps_the_new_monday_morning() {
        let episodes = vec![episode("sunday", Some(at(25, 22))), episode("monday", Some(at(26, 4))), episode("tuesday", Some(at(27, 4)))];
        let span = Span::calendar(NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(), &at(27, 1)).unwrap();
        assert_eq!(guids(pick(&show(), episodes, &span)), ["monday", "sunday"]);
    }

    #[test]
    fn a_calendar_week_is_monday_midnight_to_its_end_in_local_time_newest_first() {
        let utc_sunday_night = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2026, 10, 26, 2, 30, 0).unwrap();
        let episodes = vec![
            episode("domingo", Some(at(25, 22))),
            episode("domingo-utc", Some(utc_sunday_night)),
            episode("lunes", Some(at(26, 0))),
            episode("miercoles", Some(at(28, 9))),
            episode("futuro", Some(at(28, 23))),
            episode("sin-fecha", None),
        ];
        let span = Span::calendar(NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(), &at(28, 12)).unwrap();
        assert_eq!(guids(pick(&show(), episodes, &span)), ["miercoles", "lunes"]);
        assert_eq!(span.key(), "2026-10-26");
        assert!(Span::calendar(NaiveDate::from_ymd_opt(2026, 10, 27).unwrap(), &at(28, 12)).is_err());
    }

    #[test]
    fn last_seven_days_on_a_monday_morning_covers_the_whole_week_before() {
        let now = chile().with_ymd_and_hms(2026, 11, 2, 7, 0, 0).unwrap();
        let episodes = vec![
            episode("lunes-pasado-temprano", Some(at(26, 6))),
            episode("lunes-pasado", Some(at(26, 7))),
            episode("jueves", Some(at(29, 9))),
            episode("hoy", Some(chile().with_ymd_and_hms(2026, 11, 2, 5, 0, 0).unwrap())),
            episode("futuro", Some(chile().with_ymd_and_hms(2026, 11, 2, 8, 0, 0).unwrap())),
            episode("sin-fecha", None),
        ];
        let span = Span::last_seven_days(&now);
        assert_eq!(guids(pick(&show(), episodes, &span)), ["hoy", "jueves", "lunes-pasado"]);
        assert_eq!(span.key(), "7d-to-2026-11-02");
    }
}
