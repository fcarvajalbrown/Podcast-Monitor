use crate::chunk::seconds;
use crate::summarize::TopicSpan;

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
}
