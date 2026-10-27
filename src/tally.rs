use std::collections::HashMap;

use crate::chunk::seconds;
use crate::names::words;
use crate::summarize::TopicSpan;
use crate::transcribe::Segment;

#[derive(Debug, Clone, PartialEq)]
pub struct Airtime {
    pub name: String,
    pub seconds: f64,
}

fn span_seconds(span: &TopicSpan) -> f64 {
    match (seconds(&span.start), seconds(&span.end)) {
        (Some(start), Some(end)) => (end - start).max(0.0),
        _ => 0.0,
    }
}

pub fn airtime(topics: &[TopicSpan]) -> Vec<Airtime> {
    let mut totals: Vec<Airtime> = Vec::new();
    for span in topics {
        let name = span.name.trim();
        match totals.iter_mut().find(|total| total.name == name) {
            Some(total) => total.seconds += span_seconds(span),
            None => totals.push(Airtime { name: name.into(), seconds: span_seconds(span) }),
        }
    }
    totals.sort_by(|a, b| b.seconds.total_cmp(&a.seconds));
    totals
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mentions {
    pub name: String,
    pub count: usize,
}

pub fn mentions(people: &[String], segments: &[Segment]) -> Vec<Mentions> {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    for word in segments.iter().flat_map(|segment| words(&segment.text)) {
        *seen.entry(word).or_default() += 1;
    }
    let mut counted: Vec<Mentions> = people
        .iter()
        .map(|name| Mentions { name: name.clone(), count: words(name).last().and_then(|surname| seen.get(surname)).copied().unwrap_or(0) })
        .collect();
    counted.sort_by(|a, b| b.count.cmp(&a.count));
    counted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(name: &str, start: &str, end: &str) -> TopicSpan {
        TopicSpan { name: name.into(), start: start.into(), end: end.into() }
    }

    #[test]
    fn airtime_sums_each_topic_and_ranks_longest_first() {
        let topics = [span("Uno", "00:00", "01:00"), span("Dos", "01:00", "03:30"), span("Uno ", "[03:30]", "65:00"), span("Tres", "05:00", "06:00")];
        let ranked = airtime(&topics);
        assert_eq!(ranked, vec![
            Airtime { name: "Uno".into(), seconds: 60.0 + 3690.0 },
            Airtime { name: "Dos".into(), seconds: 150.0 },
            Airtime { name: "Tres".into(), seconds: 60.0 },
        ]);
    }

    #[test]
    fn unreadable_or_backward_spans_add_nothing() {
        let topics = [span("Uno", "02:00", "01:00"), span("Uno", "inicio", "01:00"), span("Dos", "00:00", "00:10")];
        let ranked = airtime(&topics);
        assert_eq!(ranked[0], Airtime { name: "Dos".into(), seconds: 10.0 });
        assert_eq!(ranked[1], Airtime { name: "Uno".into(), seconds: 0.0 });
    }

    #[test]
    fn people_are_counted_by_exact_surname_and_zeros_kept() {
        let segment = |text: &str| Segment { start_s: 0.0, end_s: 1.0, text: text.into() };
        let segments = [segment(" Ana Pérez y Pérez."), segment("pérez, Soto, Soto; Soto")];
        let people = ["Eva Ríos", "Ana Pérez", "Luis Soto", "Rosa Pérez"].map(String::from);
        let counted = mentions(&people, &segments);
        assert_eq!(counted.iter().map(|person| (person.name.as_str(), person.count)).collect::<Vec<_>>(), vec![
            ("Luis Soto", 3),
            ("Ana Pérez", 2),
            ("Rosa Pérez", 2),
            ("Eva Ríos", 0),
        ]);
    }
}
