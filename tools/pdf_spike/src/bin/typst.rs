use pdf_spike::{HEADER, PARAGRAPHS, SECTIONS, TABLE, TITLE, column_weights, font, out_path};
use typst::foundations::{Array, Dict, IntoValue};
use typst_layout::PagedDocument;
use typst_as_lib::TypstEngine;

const TEMPLATE: &str = r##"
#import sys: inputs
#let grey = rgb("#666666")
#set page(paper: "a4", margin: 2.5cm, footer-descent: 0.6cm, footer: context align(center, text(9pt, fill: grey, counter(page).display())))
#set text(font: "LM Roman 10", size: 10.5pt, lang: "es", top-edge: 1em, bottom-edge: "baseline")
#set par(justify: true, leading: 4.5pt, spacing: 8pt)
#show heading: set text(weight: "bold")
#show heading.where(level: 1): set text(20pt)
#show heading.where(level: 1): set block(above: 10pt, below: 14pt)
#show heading.where(level: 2): set text(15pt)
#show heading.where(level: 2): set block(above: 16pt, below: 8pt)

#text(9.5pt, fill: grey, style: "italic", inputs.header)
#v(-6pt)
#line(length: 100%, stroke: 0.5pt + rgb("#999999"))
#v(8pt)

= #inputs.title

#let body = for paragraph in inputs.paragraphs { par(paragraph) }

== #inputs.sections.at(0)
#body

== #inputs.sections.at(1)
#{
  set text(9pt, top-edge: "cap-height", bottom-edge: "baseline")
  set par(justify: false, leading: 4.5pt)
  let rows = inputs.table
  table(
    columns: inputs.weights.map(weight => weight * 1fr),
    stroke: none,
    inset: (x: 4pt, y: 3pt),
    align: left + top,
    table.hline(stroke: 0.6pt),
    table.header(..rows.at(0).map(cell => strong(cell))),
    table.hline(stroke: 0.4pt),
    ..rows.slice(1).flatten(),
    table.hline(stroke: 0.6pt),
  )
}

== #inputs.sections.at(2)
#body
#body
#body
"##;

fn main() {
    let mut inputs = Dict::new();
    inputs.insert("header".into(), HEADER.into_value());
    inputs.insert("title".into(), TITLE.into_value());
    inputs.insert("paragraphs".into(), PARAGRAPHS.iter().map(|text| text.into_value()).collect::<Array>().into_value());
    inputs.insert("sections".into(), SECTIONS.iter().map(|text| text.into_value()).collect::<Array>().into_value());
    inputs.insert("table".into(), TABLE.iter().map(|row| row.iter().map(|cell| cell.into_value()).collect::<Array>().into_value()).collect::<Array>().into_value());
    inputs.insert("weights".into(), column_weights().iter().map(|weight| (*weight as f64).into_value()).collect::<Array>().into_value());
    let fonts = ["regular", "bold", "italic", "bolditalic"].map(font);
    let engine = TypstEngine::builder().main_file(TEMPLATE).fonts(fonts.iter().map(|bytes| bytes.as_slice())).build();
    let document: PagedDocument = engine.compile_with_input(inputs).output.expect("typst compile");
    let pdf = typst_pdf::pdf(&document, &Default::default()).expect("typst pdf");
    let path = out_path("typst");
    std::fs::write(&path, pdf).unwrap();
    println!("{}", path.display());
}
