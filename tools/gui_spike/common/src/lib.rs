use std::path::{Path, PathBuf};

pub const WINDOW_TITLE: &str = "Podcast-Monitor";
pub const SHOWS_HEADING: &str = "Programas guardados";
pub const ADD: &str = "Agregar programa";
pub const REMOVE: &str = "Quitar";
pub const RUN: &str = "Ejecutar episodio";
pub const OPEN_PDF: &str = "Abrir PDF";
pub const PROGRESS_HEADING: &str = "Progreso";
pub const ACCENTS: &str = "Acentos: á é í ó ú ü ñ Á É Í Ó Ú Ñ ¿Qué pasó? ¡Atención! «angulares» “curvas”";
pub const ADDED_SHOW: (&str, &str) = ("Programa añadido de prueba", "Editorial Ñandú");
pub const WINDOW_SIZE: (f32, f32) = (560.0, 420.0);

pub const SHOWS: [(&str, &str); 2] = [
    ("Animales Políticos", "Casa Editorial Lo Que Importa"),
    ("Nada Personal", "Radio Duna"),
];

pub fn shows() -> Vec<(String, String)> {
    SHOWS.iter().map(|(name, publisher)| (name.to_string(), publisher.to_string())).collect()
}

pub fn status(progress: f32) -> String {
    format!("Transcribiendo episodio 1 de 2: «Ep. 34», {} %", (progress * 100.0).round() as u32)
}

pub fn advance(progress: f32) -> f32 {
    if progress >= 1.0 { 0.0 } else { (progress + 0.1).min(1.0) }
}

pub fn pdf_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../work/reports/animales-politicos-ep34-escritorio.pdf")
}

pub fn open_pdf() {
    let path = pdf_path();
    if let Err(error) = open::that(&path) {
        eprintln!("opening {}: {error}", path.display());
    }
}

pub fn screenshot_arg() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|arg| arg == "--screenshot").and_then(|index| args.get(index + 1)).map(PathBuf::from)
}

pub fn save_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
    let file = std::fs::File::create(path).unwrap_or_else(|error| panic!("creating {}: {error}", path.display()));
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().and_then(|mut writer| writer.write_image_data(rgba)).expect("writing png");
    println!("wrote {}", path.display());
}
