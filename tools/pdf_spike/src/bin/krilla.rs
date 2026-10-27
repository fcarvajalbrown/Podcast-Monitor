use std::sync::Arc;

use krilla::Document;
use krilla::color::rgb;
use krilla::geom::{PathBuilder, Point};
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, Stroke};
use krilla::text::{Font, TextDirection};
use pdf_spike::{HEADER, PARAGRAPHS, SECTIONS, TABLE, TITLE, column_weights, font, out_path};

const WIDTH: f32 = 595.276;
const HEIGHT: f32 = 841.89;
const MARGIN: f32 = 70.866;
const FOOTER_Y: f32 = HEIGHT - 39.685;

#[derive(Clone, Copy, PartialEq)]
enum Face {
    Regular,
    Bold,
    Italic,
}

enum Mark {
    Text { x: f32, y: f32, face: Face, size: f32, text: String, grey: bool },
    Rule { y: f32, width: f32, grey: bool },
}

type Words = Vec<(String, f32)>;

struct Faces {
    data: [Vec<u8>; 3],
}

impl Faces {
    fn width(&self, face: Face, size: f32, text: &str) -> f32 {
        let shaper = rustybuzz::Face::from_slice(&self.data[face as usize], 0).unwrap();
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str(text);
        let glyphs = rustybuzz::shape(&shaper, &[], buffer);
        let units: i32 = glyphs.glyph_positions().iter().map(|position| position.x_advance).sum();
        units as f32 * size / shaper.units_per_em() as f32
    }

    fn lines(&self, face: Face, size: f32, text: &str, width: f32) -> Vec<Words> {
        let space = self.width(face, size, " ");
        let mut lines: Vec<Words> = vec![Vec::new()];
        let mut used = 0.0;
        for word in text.split_whitespace() {
            let measured = self.width(face, size, word);
            let line = lines.last_mut().unwrap();
            if !line.is_empty() && used + space + measured > width {
                lines.push(vec![(word.to_string(), measured)]);
                used = measured;
            } else {
                used += if line.is_empty() { measured } else { space + measured };
                line.push((word.to_string(), measured));
            }
        }
        lines
    }
}

struct Style {
    face: Face,
    size: f32,
    leading: f32,
    after: f32,
    justify: bool,
    grey: bool,
}

const BODY: Style = Style { face: Face::Regular, size: 10.5, leading: 15.0, after: 8.0, justify: true, grey: false };
const HEADER_STYLE: Style = Style { face: Face::Italic, size: 9.5, leading: 12.0, after: 2.0, justify: false, grey: true };

struct Layout {
    faces: Faces,
    pages: Vec<Vec<Mark>>,
    y: f32,
}

impl Layout {
    fn new_page(&mut self) {
        self.pages.push(Vec::new());
        self.y = MARGIN;
    }

    fn mark(&mut self, mark: Mark) {
        self.pages.last_mut().unwrap().push(mark);
    }

    fn room(&mut self, height: f32) {
        if self.y + height > HEIGHT - MARGIN {
            self.new_page();
        }
    }

    fn text_line(&mut self, words: &Words, style: &Style, width: f32, justify: bool) {
        self.room(style.leading);
        self.y += style.leading;
        let natural: f32 = words.iter().map(|(_, measured)| measured).sum();
        let space = if justify && words.len() > 1 { (width - natural) / (words.len() - 1) as f32 } else { self.faces.width(style.face, style.size, " ") };
        let baseline = self.y - (style.leading - style.size) / 2.0 - style.size * 0.22;
        let mut x = MARGIN;
        for (word, measured) in words {
            self.mark(Mark::Text { x, y: baseline, face: style.face, size: style.size, text: word.clone(), grey: style.grey });
            x += measured + space;
        }
    }

    fn paragraph(&mut self, text: &str, style: &Style) {
        let width = WIDTH - 2.0 * MARGIN;
        let lines = self.faces.lines(style.face, style.size, text, width);
        let last = lines.len() - 1;
        for (index, words) in lines.iter().enumerate() {
            self.text_line(words, style, width, style.justify && index < last);
        }
        self.y += style.after;
    }

    fn heading(&mut self, text: &str, size: f32, leading: f32, before: f32, after: f32) {
        self.room(before + leading + after + 2.0 * BODY.leading);
        if self.y > MARGIN {
            self.y += before;
        }
        self.paragraph(text, &Style { face: Face::Bold, size, leading, after, justify: false, grey: false });
    }

    fn rule(&mut self, width: f32, grey: bool) {
        let y = self.y;
        self.mark(Mark::Rule { y, width, grey });
    }

    fn table(&mut self) {
        let weights = column_weights();
        let total: f32 = weights.iter().sum();
        let widths: Vec<f32> = weights.iter().map(|weight| (WIDTH - 2.0 * MARGIN) * weight / total).collect();
        self.rule(0.6, false);
        for (row_index, row) in TABLE.iter().enumerate() {
            let face = if row_index == 0 { Face::Bold } else { Face::Regular };
            let space = self.faces.width(face, 9.0, " ");
            let cells: Vec<Vec<Words>> = row.iter().zip(&widths).map(|(cell, width)| self.faces.lines(face, 9.0, cell, width - 8.0)).collect();
            let height = cells.iter().map(Vec::len).max().unwrap_or(1) as f32 * 11.5 + 6.0;
            self.room(height);
            let top = self.y;
            let mut x = MARGIN;
            for (lines, width) in cells.iter().zip(&widths) {
                for (line_index, words) in lines.iter().enumerate() {
                    let baseline = top + 3.0 + (line_index as f32 + 1.0) * 11.5 - 2.5;
                    let mut word_x = x + 4.0;
                    for (word, measured) in words {
                        self.mark(Mark::Text { x: word_x, y: baseline, face, size: 9.0, text: word.clone(), grey: false });
                        word_x += measured + space;
                    }
                }
                x += width;
            }
            self.y = top + height;
            let below = if row_index == 0 { 0.4 } else if row_index == TABLE.len() - 1 { 0.6 } else { 0.0 };
            self.rule(below, false);
        }
        self.y += 12.0;
    }
}

fn main() {
    let faces = Faces { data: [font("regular"), font("bold"), font("italic")] };
    let mut layout = Layout { faces, pages: Vec::new(), y: MARGIN };
    layout.new_page();
    layout.paragraph(HEADER, &HEADER_STYLE);
    layout.rule(0.5, true);
    layout.y += 14.0;
    layout.heading(TITLE, 20.0, 24.0, 10.0, 14.0);
    layout.heading(SECTIONS[0], 15.0, 19.0, 16.0, 8.0);
    for text in PARAGRAPHS {
        layout.paragraph(text, &BODY);
    }
    layout.heading(SECTIONS[1], 15.0, 19.0, 16.0, 8.0);
    layout.table();
    layout.heading(SECTIONS[2], 15.0, 19.0, 16.0, 8.0);
    for _ in 0..3 {
        for text in PARAGRAPHS {
            layout.paragraph(text, &BODY);
        }
    }

    let fonts: Vec<Font> = layout.faces.data.iter().map(|bytes| Font::new(Arc::new(bytes.clone()).into(), 0).unwrap()).collect();
    let fill = |shade: u8| Fill { paint: rgb::Color::new(shade, shade, shade).into(), opacity: NormalizedF32::ONE, rule: Default::default() };
    let mut document = Document::new();
    let page_count = layout.pages.len();
    for (index, marks) in layout.pages.iter().enumerate() {
        let mut page = document.start_page_with(PageSettings::from_wh(WIDTH, HEIGHT).unwrap());
        let mut surface = page.surface();
        for mark in marks {
            match mark {
                Mark::Text { x, y, face, size, text, grey } => {
                    surface.set_fill(Some(fill(if *grey { 0x66 } else { 0 })));
                    surface.draw_text(Point::from_xy(*x, *y), fonts[*face as usize].clone(), *size, text, false, TextDirection::Auto);
                }
                Mark::Rule { y, width, grey } if *width > 0.0 => {
                    let shade = if *grey { 0x99 } else { 0 };
                    let mut path = PathBuilder::new();
                    path.move_to(MARGIN, *y);
                    path.line_to(WIDTH - MARGIN, *y);
                    surface.set_fill(None);
                    surface.set_stroke(Some(Stroke { paint: rgb::Color::new(shade, shade, shade).into(), width: *width, ..Default::default() }));
                    surface.draw_path(&path.finish().unwrap());
                    surface.set_stroke(None);
                }
                Mark::Rule { .. } => {}
            }
        }
        let number = (index + 1).to_string();
        let number_width = layout.faces.width(Face::Regular, 9.0, &number);
        surface.set_fill(Some(fill(0x66)));
        surface.draw_text(Point::from_xy((WIDTH - number_width) / 2.0, FOOTER_Y), fonts[0].clone(), 9.0, &number, false, TextDirection::Auto);
        surface.finish();
        page.finish();
    }
    let path = out_path("krilla");
    std::fs::write(&path, document.finish().unwrap()).unwrap();
    println!("{} ({page_count} pages)", path.display());
}
