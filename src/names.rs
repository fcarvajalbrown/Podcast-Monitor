use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::transcribe::Segment;

pub const DEFAULT_THRESHOLD: f64 = 0.85;
const MIN_WORD_CHARS: usize = 4;
const MAX_LENGTH_GAP: usize = 2;
const COMMON_MIN: usize = 2;

pub struct Glossary {
    names: Vec<(String, Vec<char>)>,
    keep: HashSet<String>,
    heard: HashMap<String, String>,
}

impl Glossary {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path).with_context(|| format!("reading glossary {}", path.display()))?;
        Ok(Self::parse(&text))
    }

    pub fn parse(text: &str) -> Self {
        let mut names = Vec::new();
        let mut keep = HashSet::new();
        let mut heard = HashMap::new();
        for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if let Some(word) = line.strip_prefix('!') {
                keep.insert(word.trim().to_string());
                continue;
            }
            let (name, forms) = line.split_once('=').map_or((line, ""), |(name, forms)| (name.trim(), forms));
            for form in forms.split(',').map(str::trim).filter(|form| !form.is_empty()) {
                if words(form).count() == 1 && words(form).next() == Some(form) {
                    heard.insert(form.to_lowercase(), name.to_string());
                } else {
                    log::warn!("glossary: heard form {form:?} for {name} is not one word, skipped");
                }
            }
            names.push((name.to_string(), phonetic_key(name)));
        }
        Self { names, keep, heard }
    }

    pub fn prompt(&self) -> String {
        self.names.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", ")
    }

    fn best_match(&self, word: &str, threshold: f64) -> Option<&str> {
        let key = phonetic_key(word);
        let (score, name) = self
            .names
            .iter()
            .map(|(name, name_key)| (ratio(&key, name_key), name.as_str()))
            .max_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.cmp(b.1)))?;
        let gap = word.chars().count().abs_diff(name.chars().count());
        (score >= threshold && word != name && gap <= MAX_LENGTH_GAP).then_some(name)
    }
}

pub type Replacements = BTreeMap<(String, String), usize>;

pub fn correct(segments: &[Segment], glossary: &Glossary, threshold: f64) -> (Vec<Segment>, Replacements) {
    let common = common_lowercase_words(segments.iter().map(|segment| segment.text.as_str()));
    let mut replacements = Replacements::new();
    let mut cache: HashMap<String, Option<String>> = HashMap::new();
    let corrected = segments
        .iter()
        .map(|segment| {
            let text = map_words(&segment.text, |word| {
                if let Some(name) = glossary.heard.get(&word.to_lowercase()).filter(|name| name.as_str() != word) {
                    *replacements.entry((word.to_string(), name.clone())).or_default() += 1;
                    return Some(name.clone());
                }
                if !starts_uppercase(word) || word.chars().count() < MIN_WORD_CHARS || common.contains(&word.to_lowercase()) || glossary.keep.contains(word) {
                    return None;
                }
                let found = cache.entry(word.to_string()).or_insert_with(|| glossary.best_match(word, threshold).map(String::from)).clone()?;
                *replacements.entry((word.to_string(), found.clone())).or_default() += 1;
                Some(found)
            });
            Segment { text, ..segment.clone() }
        })
        .collect();
    (corrected, replacements)
}

fn common_lowercase_words<'a>(texts: impl Iterator<Item = &'a str>) -> HashSet<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for text in texts {
        for word in words(text).filter(|word| is_lowercase_word(word)) {
            *counts.entry(word.to_string()).or_default() += 1;
        }
    }
    counts.into_iter().filter(|(_, count)| *count >= COMMON_MIN).map(|(word, _)| word).collect()
}

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub(crate) fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !is_word_char(c)).filter(|word| !word.is_empty())
}

fn is_lowercase_word(word: &str) -> bool {
    word.chars().any(char::is_alphabetic) && !word.chars().any(char::is_uppercase)
}

fn starts_uppercase(word: &str) -> bool {
    word.chars().next().is_some_and(char::is_uppercase)
}

fn map_words(text: &str, mut replace: impl FnMut(&str) -> Option<String>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut start = None;
    for (index, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
        let inside = index < text.len() && is_word_char(c);
        match (start, inside) {
            (None, true) => start = Some(index),
            (Some(from), false) => {
                let word = &text[from..index];
                out.push_str(&replace(word).unwrap_or_else(|| word.to_string()));
                start = None;
            }
            _ => {}
        }
        if !inside && index < text.len() {
            out.push(c);
        }
    }
    out
}

pub fn phonetic_key(word: &str) -> Vec<char> {
    let mut key: Vec<char> = word.to_lowercase().chars().map(strip_accent).collect();
    key = replace_all(&key, &['l', 'l'], &['y'], |_| true);
    key = replace_all(&key, &['q', 'u'], &['k'], |_| true);
    key = replace_all(&key, &['c'], &['s'], |next| matches!(next, Some('e' | 'i')));
    key = replace_all(&key, &['c', 'h'], &['X'], |_| true);
    key = replace_all(&key, &['c'], &['k'], |_| true);
    key = replace_all(&key, &['z'], &['s'], |_| true);
    key = replace_all(&key, &['x'], &['k', 's'], |_| true);
    key = replace_all(&key, &['v'], &['b'], |_| true);
    key = replace_all(&key, &['w'], &['b'], |_| true);
    key = replace_all(&key, &['h'], &[], |_| true);
    key = replace_all(&key, &['ñ'], &['N'], |_| true);
    collapse_pairs(&key)
}

pub(crate) fn strip_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        other => other,
    }
}

fn replace_all(input: &[char], pattern: &[char], with: &[char], lookahead: impl Fn(Option<char>) -> bool) -> Vec<char> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        if input[i..].starts_with(pattern) && lookahead(input.get(i + pattern.len()).copied()) {
            out.extend_from_slice(with);
            i += pattern.len();
        } else {
            out.push(input[i]);
            i += 1;
        }
    }
    out
}

fn collapse_pairs(input: &[char]) -> Vec<char> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        out.push(input[i]);
        i += if input.get(i + 1) == Some(&input[i]) { 2 } else { 1 };
    }
    out
}

pub fn ratio(a: &[char], b: &[char]) -> f64 {
    let total = a.len() + b.len();
    if total == 0 {
        return 1.0;
    }
    2.0 * matched(a, b, 0, a.len(), 0, b.len()) as f64 / total as f64
}

fn matched(a: &[char], b: &[char], alo: usize, ahi: usize, blo: usize, bhi: usize) -> usize {
    let (i, j, size) = longest_match(a, b, alo, ahi, blo, bhi);
    if size == 0 {
        return 0;
    }
    let mut total = size;
    if alo < i && blo < j {
        total += matched(a, b, alo, i, blo, j);
    }
    if i + size < ahi && j + size < bhi {
        total += matched(a, b, i + size, ahi, j + size, bhi);
    }
    total
}

fn longest_match(a: &[char], b: &[char], alo: usize, ahi: usize, blo: usize, bhi: usize) -> (usize, usize, usize) {
    let (mut best_i, mut best_j, mut best_size) = (alo, blo, 0);
    let mut lengths: HashMap<usize, usize> = HashMap::new();
    for i in alo..ahi {
        let mut next: HashMap<usize, usize> = HashMap::new();
        for j in (blo..bhi).filter(|&j| b[j] == a[i]) {
            let size = if j > 0 { lengths.get(&(j - 1)).copied().unwrap_or(0) } else { 0 } + 1;
            next.insert(j, size);
            if size > best_size {
                best_i = i + 1 - size;
                best_j = j + 1 - size;
                best_size = size;
            }
        }
        lengths = next;
    }
    (best_i, best_j, best_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(word: &str) -> String {
        phonetic_key(word).into_iter().collect()
    }

    fn segment(text: &str) -> Segment {
        Segment { start_s: 0.0, end_s: 1.0, text: text.into() }
    }

    #[test]
    fn spanish_phonetic_rules() {
        assert_eq!(key("Llanquihue"), "yankiue");
        assert_eq!(key("Cecilia"), "sesilia");
        assert_eq!(key("Chávez"), "Xabes");
        assert_eq!(key("Peña"), "peNa");
        assert_eq!(key("Rossi"), "rosi");
    }

    #[test]
    fn ratio_matches_difflib() {
        let r = |a: &str, b: &str| ratio(&a.chars().collect::<Vec<_>>(), &b.chars().collect::<Vec<_>>());
        assert!((r("abcd", "bcde") - 0.75).abs() < 1e-9);
        assert!((r("kas", "kast") - 6.0 / 7.0).abs() < 1e-9);
        assert!((r("tide", "diet") - 0.25).abs() < 1e-9);
        assert_eq!(r("", ""), 1.0);
    }

    #[test]
    fn replaces_near_spelling_and_keeps_punctuation() {
        let glossary = Glossary::parse("Ferreira\nMontecinos\n");
        let (fixed, replacements) = correct(&[segment("Dijo Ferreyra, y luego Montesinos.")], &glossary, DEFAULT_THRESHOLD);
        assert_eq!(fixed[0].text, "Dijo Ferreira, y luego Montecinos.");
        assert_eq!(replacements.values().sum::<usize>(), 2);
    }

    #[test]
    fn heard_forms_replace_whole_words_in_any_case() {
        let glossary = Glossary::parse("Huneeus = Uneos, Luneus
Mimica = Mimisa
Kast
");
        let input = [segment("Cristóbal Uneos, UNEOS y luneus. Davor Mimisa, Uneoso.")];
        let (fixed, replacements) = correct(&input, &glossary, DEFAULT_THRESHOLD);
        assert_eq!(fixed[0].text, "Cristóbal Huneeus, Huneeus y Huneeus. Davor Mimica, Uneoso.");
        assert_eq!(replacements.values().sum::<usize>(), 4);
        assert_eq!(glossary.prompt(), "Huneeus, Mimica, Kast");
    }

    #[test]
    fn heard_forms_of_several_words_are_skipped() {
        let glossary = Glossary::parse("Kouyoumdjian = Coyun Tian, Coyuntian
");
        let (fixed, _) = correct(&[segment("Coyun Tian y Coyuntian")], &glossary, DEFAULT_THRESHOLD);
        assert_eq!(fixed[0].text, "Coyun Tian y Kouyoumdjian");
    }

    #[test]
    fn skips_lowercase_short_common_and_kept_words() {
        let glossary = Glossary::parse("Ferreira\nSalas\n!Ferreyra\n");
        let input = [segment("ferreyra Sal Ferreyra"), segment("Salas casas casas Casas")];
        let (fixed, replacements) = correct(&input, &glossary, DEFAULT_THRESHOLD);
        assert_eq!(fixed[0].text, "ferreyra Sal Ferreyra");
        assert_eq!(fixed[1].text, "Salas casas casas Casas");
        assert!(replacements.is_empty());
    }

    #[test]
    #[ignore]
    fn lists_every_change_the_fix_makes_on_saved_transcripts() {
        #[derive(serde::Deserialize)]
        struct Transcript {
            segments: Vec<Segment>,
        }
        let root = std::path::PathBuf::from(std::env::var("PM_RUNS").expect("PM_RUNS names a run folder"));
        let glossary = Glossary::load(Path::new(&std::env::var("PM_GLOSSARY").expect("PM_GLOSSARY names a glossary"))).unwrap();
        println!("folder\tstart_s\theard\tfixed\tkind\tsegment");
        for folder in crate::assemble::episode_folders(&root).unwrap() {
            let Some(transcript) = crate::run::read_json::<Transcript>(&folder.join("transcript.json")).unwrap() else { continue };
            let (fixed, _) = correct(&transcript.segments, &glossary, DEFAULT_THRESHOLD);
            for (before, after) in transcript.segments.iter().zip(&fixed) {
                for (heard, name) in words(&before.text).zip(words(&after.text)).filter(|(heard, name)| heard != name) {
                    let kind = if glossary.heard.contains_key(&heard.to_lowercase()) { "heard-as" } else { "fuzzy" };
                    println!("{}\t{:.0}\t{heard}\t{name}\t{kind}\t{}", folder.strip_prefix(&root).unwrap().display(), before.start_s, before.text.trim().replace(['\t', '\n'], " "));
                }
            }
        }
    }
}
