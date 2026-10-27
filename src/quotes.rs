use crate::names::{is_word_char, strip_accent};
use crate::transcribe::Segment;

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub text: String,
    pub start_s: f64,
}

struct Word {
    segment: usize,
    start: usize,
    end: usize,
    folded: String,
}

pub struct Index<'a> {
    segments: &'a [Segment],
    words: Vec<Word>,
}

fn fold(word: &str) -> String {
    word.chars().flat_map(char::to_lowercase).map(strip_accent).collect()
}

fn spans(text: &str) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut start = None;
    for (at, c) in text.char_indices() {
        match (is_word_char(c), start) {
            (true, None) => start = Some(at),
            (false, Some(from)) => {
                found.push((from, at));
                start = None;
            }
            _ => {}
        }
    }
    found.extend(start.map(|from| (from, text.len())));
    found
}

fn seconds(minute: &str) -> Option<f64> {
    let (m, s) = minute.trim().trim_matches(['[', ']']).split_once(':')?;
    Some(m.trim().parse::<f64>().ok()? * 60.0 + s.trim().parse::<f64>().ok()?)
}

impl<'a> Index<'a> {
    pub fn new(segments: &'a [Segment]) -> Self {
        let words = segments
            .iter()
            .enumerate()
            .flat_map(|(segment, s)| spans(&s.text).into_iter().map(move |(start, end)| Word { segment, start, end, folded: fold(&s.text[start..end]) }))
            .collect();
        Self { segments, words }
    }

    pub fn find(&self, quote: &str, minute: &str) -> Option<Quote> {
        let wanted: Vec<String> = spans(quote).into_iter().map(|(start, end)| fold(&quote[start..end])).collect();
        if wanted.is_empty() || wanted.len() > self.words.len() {
            return None;
        }
        let hits = (0..=self.words.len() - wanted.len()).filter(|&at| self.words[at..at + wanted.len()].iter().zip(&wanted).all(|(word, want)| &word.folded == want));
        let near = seconds(minute).unwrap_or(0.0);
        let first = hits.min_by(|&a, &b| self.distance(a, near).total_cmp(&self.distance(b, near)))?;
        Some(Quote { text: self.text(first, first + wanted.len() - 1), start_s: self.segments[self.words[first].segment].start_s })
    }

    fn distance(&self, at: usize, near: f64) -> f64 {
        (self.segments[self.words[at].segment].start_s - near).abs()
    }

    fn text(&self, first: usize, last: usize) -> String {
        let (from, to) = (&self.words[first], &self.words[last]);
        (from.segment..=to.segment)
            .map(|segment| {
                let text = &self.segments[segment].text;
                let start = if segment == from.segment { from.start } else { 0 };
                let end = if segment == to.segment { to.end } else { text.len() };
                text[start..end].trim()
            })
            .filter(|piece| !piece.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start_s: f64, text: &str) -> Segment {
        Segment { start_s, end_s: start_s + 5.0, text: text.into() }
    }

    #[test]
    fn finds_across_segments_and_prints_transcript_words() {
        let segments = [segment(10.0, " Bueno, la reunión fue"), segment(15.0, " muy larga. Después"), segment(20.0, " nos fuimos.")];
        let found = Index::new(&segments).find("la reunion fue muy larga", "00:12").unwrap();
        assert_eq!(found, Quote { text: "la reunión fue muy larga".into(), start_s: 10.0 });
    }

    #[test]
    fn picks_the_hit_nearest_the_given_minute() {
        let segments = [segment(0.0, "hola a todos"), segment(600.0, "Hola a todos.")];
        assert_eq!(Index::new(&segments).find("hola a todos", "09:58").unwrap().start_s, 600.0);
        assert_eq!(Index::new(&segments).find("hola a todos", "sin minuto").unwrap().start_s, 0.0);
    }

    #[test]
    fn drops_quotes_not_in_the_transcript() {
        let segments = [segment(0.0, "hola a todos")];
        let index = Index::new(&segments);
        assert!(index.find("hola a todas", "00:00").is_none());
        assert!(index.find("...", "00:00").is_none());
        assert!(index.find("hola a todos y todas", "00:00").is_none());
    }
}
