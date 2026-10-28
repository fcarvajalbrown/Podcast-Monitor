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

pub fn in_week<Tz: TimeZone>(episode: &Episode, monday: NaiveDate, now: &DateTime<Tz>) -> bool {
    episode.published.as_ref().is_some_and(|published| {
        let local = published.with_timezone(&now.timezone());
        local.date_naive() >= monday && local <= *now
    })
}

pub fn pick<Tz: TimeZone>(show: &SavedShow, episodes: Vec<Episode>, now: &DateTime<Tz>) -> Vec<Request> {
    let first = monday(now.date_naive());
    let mut picked: Vec<Episode> = episodes.into_iter().filter(|episode| in_week(episode, first, now)).collect();
    picked.sort_by(|a, b| b.published.cmp(&a.published));
    picked.into_iter().map(|episode| Request { show: show.clone(), episode }).collect()
}

pub fn collect(shows: &[SavedShow], now: DateTime<Local>) -> Week {
    let mut sorted = shows.to_vec();
    sorted.sort_by_key(|show| show.title.to_lowercase());
    let mut week = Week { monday: monday(now.date_naive()), requests: Vec::new(), failed: Vec::new() };
    for show in sorted {
        match feed::fetch(&show.feed) {
            Ok(fetched) => {
                let picked = pick(&show, fetched.episodes, &now);
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
