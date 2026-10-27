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
    Heading { level: usize, text: String },
    Paragraph(String),
    Table(Vec<Vec<String>>),
    PageBreak,
}

fn strings(items: &[String]) -> Value {
    items.iter().map(|item| item.as_str().into_value()).collect::<Array>().into_value()
}

pub fn column_weights(rows: &[Vec<String>]) -> Vec<f64> {
    let columns = rows.iter().map(Vec::len).max().unwrap_or(0);
    (0..columns)
        .map(|column| rows.iter().map(|row| row.get(column).map_or(0, |cell| cell.chars().count())).max().unwrap_or(0).clamp(6, 60) as f64)
        .collect()
}

impl Block {
    fn into_value(self) -> Value {
        let mut dict = Dict::new();
        match self {
            Block::Heading { level, text } => {
                dict.insert("kind".into(), "heading".into_value());
                dict.insert("level".into(), (level as i64).into_value());
                dict.insert("text".into(), text.into_value());
            }
            Block::Paragraph(text) => {
                dict.insert("kind".into(), "paragraph".into_value());
                dict.insert("text".into(), text.into_value());
            }
            Block::PageBreak => {
                dict.insert("kind".into(), "pagebreak".into_value());
            }
            Block::Table(rows) => {
                let width = rows.iter().map(Vec::len).max().unwrap_or(0);
                let rows: Vec<Vec<String>> = rows.into_iter().map(|mut row| { row.resize(width, String::new()); row }).collect();
                dict.insert("kind".into(), "table".into_value());
                dict.insert("weights".into(), column_weights(&rows).into_iter().map(IntoValue::into_value).collect::<Array>().into_value());
                dict.insert("rows".into(), rows.iter().map(|row| strings(row)).collect::<Array>().into_value());
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
    use typst::layout::{Frame, FrameItem};

    use super::*;

    fn text_of(frame: &Frame, out: &mut String) {
        for (_, item) in frame.items() {
            match item {
                FrameItem::Group(group) => text_of(&group.frame, out),
                FrameItem::Text(text) => {
                    out.push_str(&text.text);
                    out.push(' ');
                }
                _ => {}
            }
        }
    }

    fn pages(blocks: Vec<Block>) -> Vec<String> {
        layout("Prueba", blocks).unwrap().pages().iter().map(|page| {
            let mut text = String::new();
            text_of(&page.frame, &mut text);
            text
        }).collect()
    }

    fn paragraph(text: &str) -> Block {
        Block::Paragraph(text.into())
    }

    fn heading(level: usize, text: &str) -> Block {
        Block::Heading { level, text: text.into() }
    }

    fn sample() -> Vec<Block> {
        let body = [
            "Esta página compara cómo se componen los informes del monitor de pódcast. El texto usa la fuente Latin Modern Roman incrustada en el archivo, con todos los signos del castellano: á, é, í, ó, ú, ü, ñ, Á, É, Í, Ó, Ú, Ñ, los signos de apertura ¿ y ¡, las comillas «angulares» y “curvas”, y cifras como 1.234,56 o 23:44.",
            "Cada párrafo va justificado, de modo que el borde derecho queda recto y el espacio entre palabras se reparte a lo largo de la línea. Las palabras largas como institucionalidad, constitucionalmente o extraordinariamente ponen a prueba el corte de línea, y la última línea de cada párrafo queda alineada a la izquierda. ¿Se nota la diferencia? ¡Debería notarse!",
            "El resumen de cada episodio ocupa uno o más párrafos. Los temas aparecen en una tabla con su tiempo al aire, las personas mencionadas con su número de menciones, y las citas con el minuto en que se encuentran en la transcripción, por ejemplo [12:05] o [1:03:40], tal como quedaron escritas en ella.",
            "Cuando un párrafo no cabe al final de una página, sigue en la siguiente, y un título nunca queda solo al pie de una página: siempre viaja con el bloque que lo sigue. El número de página va centrado al pie, en gris, a un centímetro y medio del borde inferior.",
        ];
        let rows = [
            ["Tema", "Inicio", "Fin", "Tiempo al aire"],
            ["Presentación del episodio", "00:00", "03:12", "3:12"],
            ["Tema principal de la semana, con un nombre largo que obliga a cortar la celda en dos líneas", "03:12", "41:50", "38:38"],
            ["Preguntas de la audiencia", "41:50", "55:07", "13:17"],
            ["Cierre", "55:07", "58:00", "2:53"],
        ];
        let mut blocks = vec![heading(1, "Página de prueba de tipografía"), heading(2, "Texto justificado")];
        blocks.extend(body.map(paragraph));
        blocks.push(heading(2, "Tabla de temas"));
        blocks.push(Block::Table(rows.iter().map(|row| row.map(String::from).to_vec()).collect()));
        blocks.push(heading(2, "Continuación en la página siguiente"));
        for _ in 0..3 {
            blocks.extend(body.map(paragraph));
        }
        blocks
    }

    #[test]
    fn sample_fills_two_pages() {
        assert_eq!(pages(sample()).len(), 2);
        assert!(pdf("Prueba", sample()).unwrap().starts_with(b"%PDF"));
    }

    #[test]
    fn heading_never_ends_a_page() {
        for count in 20..50 {
            let mut blocks: Vec<Block> = (0..count).map(|index| paragraph(&format!("Línea corta número {index}."))).collect();
            blocks.push(heading(2, "Encabezado de control"));
            blocks.push(paragraph("Texto que sigue al encabezado."));
            let text = pages(blocks);
            let page = text.iter().find(|page| page.contains("Encabezado")).unwrap();
            assert!(page.contains("sigue"), "heading left alone at the foot of a page after {count} lines");
        }
    }

    #[test]
    fn long_paragraph_flows_across_pages() {
        let mut blocks: Vec<Block> = (0..20).map(|index| paragraph(&format!("Línea corta número {index}."))).collect();
        blocks.push(paragraph(&format!("{} fin.", "palabra ".repeat(1500))));
        let text = pages(blocks);
        assert!(text.len() >= 2);
        assert!(text[0].contains("palabra") && text[1].contains("palabra"));
    }

    #[test]
    fn table_header_repeats_and_short_rows_are_padded() {
        let mut rows = vec![vec!["Persona".to_string(), "Menciones".to_string()]];
        rows.extend((0..90).map(|index| vec![format!("Nombre {index}")]));
        let text = pages(vec![Block::Table(rows)]);
        assert!(text.len() >= 2);
        assert!(text.iter().all(|page| page.contains("Persona")));
    }

    #[test]
    fn column_weights_follow_the_longest_cell_within_bounds() {
        let rows = vec![vec!["Tema".to_string(), "Fin".into()], vec!["x".repeat(80), "00:10".into()]];
        assert_eq!(column_weights(&rows), vec![60.0, 6.0]);
    }

    #[test]
    #[ignore = "writes a sample PDF for tools/pdf_check/measure.py"]
    fn writes_sample() {
        let path = std::env::temp_dir().join("podcast-monitor-sample.pdf");
        std::fs::write(&path, pdf("Prueba", sample()).unwrap()).unwrap();
        println!("{}", path.display());
    }
}
