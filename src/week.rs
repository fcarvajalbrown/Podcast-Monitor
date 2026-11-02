use chrono::{DateTime, Datelike, Days, Local, NaiveDate, TimeZone};

use crate::feed::{self, Episode};
use crate::job::Request;
use crate::shows::SavedShow;

#[derive(Debug, Clone, PartialEq)]
pub struct Week {
    pub monday: NaiveDate,
    pub requests: Vec<Request>,
    pub failed: Vec<SavedShow>,
}

pub fn monday(today: NaiveDate) -> NaiveDate {
    today - Days::new(u64::from(today.weekday().num_days_from_monday()))
}

pub fn checked_monday(start: NaiveDate) -> anyhow::Result<NaiveDate> {
    if start.weekday() != chrono::Weekday::Mon {
        anyhow::bail!("{start} is a {}, not a Monday", start.weekday());
    }
    Ok(start)
}

pub fn end_of(monday: Option<NaiveDate>, now: DateTime<Local>) -> anyhow::Result<DateTime<Local>> {
    let Some(start) = monday else { return Ok(now) };
    checked_monday(start)?;
    let sunday_night = (start + Days::new(6)).and_hms_opt(23, 59, 59).and_then(|time| time.and_local_timezone(Local).earliest());
    let end = sunday_night.ok_or_else(|| anyhow::anyhow!("no local time for the Sunday after {start}"))?;
    Ok(end.min(now))
}

pub fn in_week<Tz: TimeZone>(episode: &Episode, monday: NaiveDate, now: &DateTime<Tz>) -> bool {
    episode.published.as_ref().is_some_and(|published| {
        let local = published.with_timezone(&now.timezone());
        local.date_naive() >= monday && local <= *now
    })
}

pub fn pick<Tz: TimeZone>(show: &SavedShow, episodes: Vec<Episode>, now: &DateTime<Tz>) -> Vec<Request> {
    pick_from(show, episodes, monday(now.date_naive()), now)
}

pub fn pick_from<Tz: TimeZone>(show: &SavedShow, episodes: Vec<Episode>, first: NaiveDate, now: &DateTime<Tz>) -> Vec<Request> {
    let mut picked: Vec<Episode> = episodes.into_iter().filter(|episode| in_week(episode, first, now)).collect();
    picked.sort_by(|a, b| b.published.cmp(&a.published));
    picked.into_iter().map(|episode| Request { show: show.clone(), episode }).collect()
}

pub fn collect(shows: &[SavedShow], now: DateTime<Local>) -> Week {
    collect_from(shows, monday(now.date_naive()), now)
}

pub fn collect_from(shows: &[SavedShow], first: NaiveDate, now: DateTime<Local>) -> Week {
    let mut sorted = shows.to_vec();
    sorted.sort_by_key(|show| show.title.to_lowercase());
    let mut week = Week { monday: first, requests: Vec::new(), failed: Vec::new() };
    for show in sorted {
        match feed::fetch(&show.feed) {
            Ok(fetched) => {
                let picked = pick_from(&show, fetched.episodes, first, &now);
                log::debug!("{}: {} episode(s) since {}", show.title, picked.len(), week.monday);
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
    use chrono::FixedOffset;

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

    #[test]
    fn end_of_a_past_week_is_its_sunday_night() {
        let now = Local.with_ymd_and_hms(2026, 11, 2, 0, 43, 0).unwrap();
        let end = end_of(NaiveDate::from_ymd_opt(2026, 10, 26), now).unwrap();
        assert_eq!(end, Local.with_ymd_and_hms(2026, 11, 1, 23, 59, 59).unwrap());
        assert_eq!(monday(end.date_naive()), NaiveDate::from_ymd_opt(2026, 10, 26).unwrap());
        assert!(end_of(NaiveDate::from_ymd_opt(2026, 10, 27), now).is_err());
        assert_eq!(end_of(None, now).unwrap(), now);
    }

    #[test]
    fn a_past_monday_to_now_keeps_the_new_monday_morning() {
        let show = SavedShow { title: "Show".into(), publisher: String::new(), feed: String::new() };
        let episodes = vec![episode("sunday", Some(at(25, 22))), episode("monday", Some(at(26, 4))), episode("tuesday", Some(at(27, 4)))];
        let picked: Vec<String> = pick_from(&show, episodes, NaiveDate::from_ymd_opt(2026, 10, 19).unwrap(), &at(27, 1)).into_iter().map(|request| request.episode.guid).collect();
        assert_eq!(picked, ["monday", "sunday"]);
    }

    #[test]
    fn monday_of_the_week() {
        let day = |d| NaiveDate::from_ymd_opt(2026, 10, d).unwrap();
        assert_eq!(monday(day(28)), day(26));
        assert_eq!(monday(day(26)), day(26));
        assert_eq!(monday(day(25)), day(19));
    }

    #[test]
    fn monday_to_now_in_local_time_newest_first() {
        let show = SavedShow { title: "Programa".into(), publisher: "Radio".into(), feed: "https://example.com/feed".into() };
        let utc_sunday_night = FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2026, 10, 26, 2, 30, 0).unwrap();
        let episodes = vec![
            episode("domingo", Some(at(25, 22))),
            episode("domingo-utc", Some(utc_sunday_night)),
            episode("lunes", Some(at(26, 0))),
            episode("miercoles", Some(at(28, 9))),
            episode("futuro", Some(at(28, 23))),
            episode("sin-fecha", None),
        ];
        let picked: Vec<String> = pick(&show, episodes, &at(28, 12)).into_iter().map(|request| request.episode.guid).collect();
        assert_eq!(picked, ["miercoles", "lunes"]);
    }
}
