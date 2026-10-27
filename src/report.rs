use anyhow::{Result, anyhow};
use typst::foundations::{Array, Dict, IntoValue, Value};
use typst_as_lib::TypstEngine;
use typst_layout::PagedDocument;

const TEMPLATE: &str = include_str!("report.typ");
const FONTS: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/lmroman10-regular.ttf"),
    include_bytes!("../assets/fonts/lmroman10-bold.ttf"),
    include_bytes!("../assets/fonts/lmroman10-italic.ttf"),
    include_bytes!("../assets/fonts/lmroman10-bolditalic.ttf"),
];
pub const HEADER: &str = "Desde el escritorio de Felipe Carvajal Brown";

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Paragraph(String),
}

impl Block {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        match self {
            Block::Paragraph(text) => {
                dict.insert("kind".into(), "paragraph".into_value());
                dict.insert("text".into(), text.into_value());
            }
        }
        dict.into_value()
    }
}

fn layout(title: &str, blocks: Vec<Block>) -> Result<PagedDocument> {
    let mut inputs = Dict::new();
    inputs.insert("title".into(), title.into_value());
    inputs.insert("header".into(), HEADER.into_value());
    inputs.insert("blocks".into(), blocks.into_iter().map(Block::into_value).collect::<Array>().into_value());
    let engine = TypstEngine::builder().main_file(TEMPLATE).fonts(FONTS).build();
    engine.compile_with_input(inputs).output.map_err(|error| anyhow!("laying out the report: {error:?}"))
}

pub fn pdf(title: &str, blocks: Vec<Block>) -> Result<Vec<u8>> {
    let document = layout(title, blocks)?;
    typst_pdf::pdf(&document, &Default::default()).map_err(|error| anyhow!("writing the report PDF: {error:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filler(count: usize) -> Vec<Block> {
        let text = "Párrafo de prueba con acentos, eñes y signos de apertura: ¿qué pasa? ¡Nada! ".repeat(6);
        (0..count).map(|_| Block::Paragraph(text.clone())).collect()
    }

    #[test]
    fn frame_flows_onto_later_pages() {
        let document = layout("Prueba", filler(14)).unwrap();
        assert!(document.pages().len() >= 2);
        let bytes = pdf("Prueba", filler(1)).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    #[ignore = "writes a sample PDF for tools/pdf_check/measure.py"]
    fn writes_sample() {
        let path = std::env::temp_dir().join("podcast-monitor-sample.pdf");
        std::fs::write(&path, pdf("Prueba", filler(14)).unwrap()).unwrap();
        println!("{}", path.display());
    }
}
