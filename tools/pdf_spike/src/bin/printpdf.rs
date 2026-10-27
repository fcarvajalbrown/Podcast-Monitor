use std::collections::BTreeMap;

use pdf_spike::{HEADER, PARAGRAPHS, SECTIONS, TABLE, TITLE, column_weights, font, out_path};
use printpdf::{
    Base64OrRaw, Color, GeneratePdfOptions, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfSaveOptions, Point, Pt, Rgb, TextItem,
};

const STYLE: &str = r#"
body { font-family: "LM Roman 10"; font-size: 10.5pt; line-height: 15pt; color: #000000; }
.header { font-style: italic; font-size: 9.5pt; color: #666666; border-bottom: 0.5pt solid #999999; padding-bottom: 2pt; margin-bottom: 14pt; }
h1 { font-weight: bold; font-size: 20pt; line-height: 24pt; margin-top: 10pt; margin-bottom: 14pt; }
h2 { font-weight: bold; font-size: 15pt; line-height: 19pt; margin-top: 16pt; margin-bottom: 8pt; page-break-after: avoid; }
p { text-align: justify; margin-top: 0pt; margin-bottom: 8pt; }
table { width: 100%; border-collapse: collapse; border-top: 0.6pt solid #000000; border-bottom: 0.6pt solid #000000; margin-bottom: 12pt; }
th { font-weight: bold; text-align: left; border-bottom: 0.4pt solid #000000; }
th, td { font-size: 9pt; line-height: 11.5pt; padding: 3pt 4pt; vertical-align: top; text-align: left; }
"#;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn html() -> String {
    let paragraphs: String = PARAGRAPHS.iter().map(|text| format!("<p>{}</p>", escape(text))).collect();
    let weights = column_weights();
    let total: f32 = weights.iter().sum();
    let mut table = String::from("<table>");
    for (index, row) in TABLE.iter().enumerate() {
        let tag = if index == 0 { "th" } else { "td" };
        table.push_str("<tr>");
        for (cell, weight) in row.iter().zip(&weights) {
            table.push_str(&format!("<{tag} style=\"width: {:.1}%\">{}</{tag}>", 100.0 * weight / total, escape(cell)));
        }
        table.push_str("</tr>");
    }
    table.push_str("</table>");
    format!(
        "<html><head><style>{STYLE}</style></head><body><div class=\"header\">{}</div><h1>{}</h1><h2>{}</h2>{paragraphs}<h2>{}</h2>{table}<h2>{}</h2>{paragraphs}{paragraphs}{paragraphs}</body></html>",
        escape(HEADER),
        escape(TITLE),
        escape(SECTIONS[0]),
        escape(SECTIONS[1]),
        escape(SECTIONS[2]),
    )
}

fn width(bytes: &[u8], size: f32, text: &str) -> f32 {
    let face = rustybuzz::Face::from_slice(bytes, 0).unwrap();
    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(text);
    let units: i32 = rustybuzz::shape(&face, &[], buffer).glyph_positions().iter().map(|position| position.x_advance).sum();
    units as f32 * size / face.units_per_em() as f32
}

fn main() {
    let mut fonts = BTreeMap::new();
    for name in ["regular", "bold", "italic", "bolditalic"] {
        fonts.insert(format!("lmroman10-{name}"), Base64OrRaw::Raw(font(name)));
    }
    let options = GeneratePdfOptions {
        page_width: Some(210.0),
        page_height: Some(297.0),
        margin_top: Some(25.0),
        margin_right: Some(25.0),
        margin_bottom: Some(25.0),
        margin_left: Some(25.0),
        ..Default::default()
    };
    let mut warnings = Vec::new();
    let mut document = PdfDocument::from_html(&html(), &BTreeMap::new(), &fonts, &options, &mut warnings).expect("printpdf html");
    for warning in &warnings {
        eprintln!("warning: {warning:?}");
    }
    let regular = font("regular");
    let footer_font = document.add_font(&ParsedFont::from_bytes(&regular, 0, &mut Vec::new()).unwrap());
    let page_count = document.pages.len();
    for (index, page) in document.pages.iter_mut().enumerate() {
        let number = (index + 1).to_string();
        let x = (595.276 - width(&regular, 9.0, &number)) / 2.0;
        page.ops.extend([
            Op::SaveGraphicsState,
            Op::StartTextSection,
            Op::SetFont { font: PdfFontHandle::External(footer_font.clone()), size: Pt(9.0) },
            Op::SetFillColor { col: Color::Rgb(Rgb { r: 0.4, g: 0.4, b: 0.4, icc_profile: None }) },
            Op::SetTextCursor { pos: Point { x: Pt(x), y: Pt(39.685) } },
            Op::ShowText { items: vec![TextItem::Text(number)] },
            Op::EndTextSection,
            Op::RestoreGraphicsState,
        ]);
    }
    let path = out_path("printpdf");
    std::fs::write(&path, document.save(&PdfSaveOptions::default(), &mut Vec::new())).unwrap();
    println!("{} ({page_count} pages)", path.display());
}
