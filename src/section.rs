use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate};

use crate::chunk::clock;
use crate::report::Block;
use crate::tally::{Airtime, Mentions};

const MONTHS: [&str; 12] = ["enero", "febrero", "marzo", "abril", "mayo", "junio", "julio", "agosto", "septiembre", "octubre", "noviembre", "diciembre"];

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub start_s: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpisodeSection {
    pub show: String,
    pub title: String,
    pub published: Option<DateTime<FixedOffset>>,
    pub duration_s: u64,
    pub summary: String,
    pub topics: Vec<Airtime>,
    pub people: Vec<Mentions>,
    pub quotes: Vec<Quote>,
}

pub fn spanish_date(date: NaiveDate) -> String {
    format!("{} de {} de {}", date.day(), MONTHS[date.month0() as usize], date.year())
}

pub fn spanish_duration(seconds: u64) -> String {
    let minutes = (seconds + 30) / 60;
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{minutes} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes} min"),
    }
}

fn row(cells: [String; 2]) -> Vec<String> {
    cells.to_vec()
}

impl EpisodeSection {
    fn details(&self) -> Option<String> {
        let parts: Vec<String> = [
            self.published.as_ref().map(|date| format!("Publicado el {}.", spanish_date(date.with_timezone(&Local).date_naive()))),
            (self.duration_s > 0).then(|| format!("Duración: {}.", spanish_duration(self.duration_s))),
        ]
        .into_iter()
        .flatten()
        .collect();
        (!parts.is_empty()).then(|| parts.join(" "))
    }

    pub fn blocks(&self) -> Vec<Block> {
        let heading = |level: usize, text: &str| Block::Heading { level, text: text.into() };
        let mut blocks = vec![heading(1, &self.show), heading(2, &self.title)];
        blocks.extend(self.details().map(Block::Paragraph));
        let summary: Vec<Block> = self.summary.split("\n\n").map(str::trim).filter(|text| !text.is_empty()).map(|text| Block::Paragraph(text.into())).collect();
        if !summary.is_empty() {
            blocks.push(heading(3, "Resumen"));
            blocks.extend(summary);
        }
        if !self.topics.is_empty() {
            blocks.push(heading(3, "Temas por tiempo al aire"));
            let mut rows = vec![row(["Tema".into(), "Tiempo al aire".into()])];
            rows.extend(self.topics.iter().map(|topic| row([topic.name.clone(), clock(topic.seconds)])));
            blocks.push(Block::Table(rows));
        }
        if !self.people.is_empty() {
            blocks.push(heading(3, "Personas mencionadas"));
            let mut rows = vec![row(["Persona".into(), "Menciones".into()])];
            rows.extend(self.people.iter().map(|person| row([person.name.clone(), person.count.to_string()])));
            blocks.push(Block::Table(rows));
        }
        if !self.quotes.is_empty() {
            blocks.push(heading(3, "Citas"));
            blocks.extend(self.quotes.iter().map(|quote| Block::Paragraph(format!("[{}] «{}»", clock(quote.start_s), quote.text.trim()))));
        }
        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section() -> EpisodeSection {
        EpisodeSection {
            show: "Programa de prueba".into(),
            title: "Ep. 1 - Un episodio".into(),
            published: DateTime::parse_from_rfc3339("2026-10-26T15:00:00Z").ok(),
            duration_s: 4685,
            summary: "Primer párrafo.\n\n Segundo párrafo. ".into(),
            topics: vec![Airtime { name: "Tema largo".into(), seconds: 1880.0 }, Airtime { name: "Cierre".into(), seconds: 65.0 }],
            people: vec![Mentions { name: "Ana Pérez".into(), count: 16 }, Mentions { name: "Eva Ríos".into(), count: 0 }],
            quotes: vec![Quote { start_s: 1424.4, text: " una cita del episodio ".into() }],
        }
    }

    fn table(cells: &[[&str; 2]]) -> Block {
        Block::Table(cells.iter().map(|cells| cells.map(String::from).to_vec()).collect())
    }

    #[test]
    fn section_reads_in_spanish_in_the_agreed_order() {
        let heading = |level: usize, text: &str| Block::Heading { level, text: text.into() };
        let paragraph = |text: &str| Block::Paragraph(text.into());
        assert_eq!(section().blocks(), vec![
            heading(1, "Programa de prueba"),
            heading(2, "Ep. 1 - Un episodio"),
            paragraph("Publicado el 26 de octubre de 2026. Duración: 1 h 18 min."),
            heading(3, "Resumen"),
            paragraph("Primer párrafo."),
            paragraph("Segundo párrafo."),
            heading(3, "Temas por tiempo al aire"),
            table(&[["Tema", "Tiempo al aire"], ["Tema largo", "31:20"], ["Cierre", "01:05"]]),
            heading(3, "Personas mencionadas"),
            table(&[["Persona", "Menciones"], ["Ana Pérez", "16"], ["Eva Ríos", "0"]]),
            heading(3, "Citas"),
            paragraph("[23:44] «una cita del episodio»"),
        ]);
    }

    #[test]
    fn empty_parts_and_missing_details_are_left_out() {
        let section = EpisodeSection { published: None, duration_s: 0, summary: " ".into(), topics: vec![], people: vec![], quotes: vec![], ..section() };
        assert_eq!(section.blocks().len(), 2);
    }

    #[test]
    fn durations_round_to_the_minute() {
        assert_eq!(spanish_duration(4685), "1 h 18 min");
        assert_eq!(spanish_duration(2699), "45 min");
        assert_eq!(spanish_duration(3590), "1 h");
    }
}
