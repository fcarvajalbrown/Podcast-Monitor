use std::path::Path;

use anyhow::{Result, anyhow};
use serde::Deserialize;

use crate::run::read_json;
use crate::transcribe::Segment;

pub const DEFAULT_MAX_CHARS: usize = 120_000;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub start_s: f64,
    pub end_s: f64,
    pub text: String,
}

#[derive(Deserialize)]
struct Segments {
    segments: Vec<Segment>,
}

pub fn load_segments(path: &Path) -> Result<Vec<Segment>> {
    read_json::<Segments>(path)?.map(|found| found.segments).ok_or_else(|| anyhow!("{} not found", path.display()))
}

pub fn clock(seconds: f64) -> String {
    let whole = seconds.max(0.0) as u64;
    format!("{:02}:{:02}", whole / 60, whole % 60)
}

pub fn split(segments: &[Segment], max_chars: usize) -> Vec<Chunk> {
    let lines: Vec<(&Segment, String)> = segments
        .iter()
        .filter(|segment| !segment.text.trim().is_empty())
        .map(|segment| (segment, format!("[{}] {}\n", clock(segment.start_s), segment.text.trim())))
        .collect();
    let total: usize = lines.iter().map(|(_, line)| line.chars().count()).sum();
    let parts = total.div_ceil(max_chars.max(1)).max(1);
    let mut chunks: Vec<(usize, Chunk)> = Vec::new();
    let mut offset = 0;
    for (segment, line) in lines {
        let part = offset * parts / total;
        offset += line.chars().count();
        match chunks.last_mut() {
            Some((last, chunk)) if *last == part => {
                chunk.end_s = segment.end_s;
                chunk.text.push_str(&line);
            }
            _ => chunks.push((part, Chunk { start_s: segment.start_s, end_s: segment.end_s, text: line })),
        }
    }
    chunks.into_iter().map(|(_, chunk)| chunk).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(start_s: f64, text: &str) -> Segment {
        Segment { start_s, end_s: start_s + 5.0, text: text.into() }
    }

    #[test]
    fn short_transcript_is_one_chunk_with_minutes() {
        let chunks = split(&[segment(0.0, " Hola a todos."), segment(65.4, "  "), segment(3725.0, "Chao.")], 1_000);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "[00:00] Hola a todos.\n[62:05] Chao.\n");
        assert_eq!((chunks[0].start_s, chunks[0].end_s), (0.0, 3730.0));
    }

    #[test]
    fn long_transcript_splits_into_even_parts_under_the_cap() {
        let segments: Vec<Segment> = (0..10).map(|i| segment(i as f64 * 10.0, "palabra palabra")).collect();
        let line = "[00:00] palabra palabra\n".chars().count();
        let chunks = split(&segments, line * 4);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|chunk| chunk.text.chars().count() <= line * 4));
        assert_eq!(chunks.iter().map(|chunk| chunk.text.lines().count()).sum::<usize>(), 10);
        assert_eq!(chunks[1].start_s, chunks[0].end_s + 5.0);
    }

    #[test]
    fn empty_transcript_has_no_chunks() {
        assert!(split(&[], 100).is_empty());
    }
}
