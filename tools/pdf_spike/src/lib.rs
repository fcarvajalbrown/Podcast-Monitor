use std::path::PathBuf;

pub const HEADER: &str = "Desde el escritorio de Felipe Carvajal Brown";
pub const TITLE: &str = "Página de prueba de tipografía";

pub const PARAGRAPHS: [&str; 4] = [
    "Esta página compara cómo se componen los informes del monitor de pódcast. El texto usa la fuente Latin Modern Roman incrustada en el archivo, con todos los signos del castellano: á, é, í, ó, ú, ü, ñ, Á, É, Í, Ó, Ú, Ñ, los signos de apertura ¿ y ¡, las comillas «angulares» y “curvas”, y cifras como 1.234,56 o 23:44.",
    "Cada párrafo va justificado, de modo que el borde derecho queda recto y el espacio entre palabras se reparte a lo largo de la línea. Las palabras largas como institucionalidad, constitucionalmente o extraordinariamente ponen a prueba el corte de línea, y la última línea de cada párrafo queda alineada a la izquierda. ¿Se nota la diferencia? ¡Debería notarse!",
    "El resumen de cada episodio ocupa uno o más párrafos. Los temas aparecen en una tabla con su tiempo al aire, las personas mencionadas con su número de menciones, y las citas con el minuto en que se encuentran en la transcripción, por ejemplo [12:05] o [1:03:40], tal como quedaron escritas en ella.",
    "Cuando un párrafo no cabe al final de una página, sigue en la siguiente, y un título nunca queda solo al pie de una página: siempre viaja con el bloque que lo sigue. El número de página va centrado al pie, en gris, a un centímetro y medio del borde inferior.",
];

pub const TABLE: [[&str; 4]; 5] = [
    ["Tema", "Inicio", "Fin", "Tiempo al aire"],
    ["Presentación del episodio", "00:00", "03:12", "3:12"],
    ["Tema principal de la semana, con un nombre largo que obliga a cortar la celda en dos líneas", "03:12", "41:50", "38:38"],
    ["Preguntas de la audiencia", "41:50", "55:07", "13:17"],
    ["Cierre", "55:07", "58:00", "2:53"],
];

pub const SECTIONS: [&str; 3] = ["Texto justificado", "Tabla de temas", "Continuación en la página siguiente"];

pub fn column_weights() -> Vec<f32> {
    (0..TABLE[0].len())
        .map(|column| TABLE.iter().map(|row| row[column].chars().count()).max().unwrap_or(0).clamp(6, 60) as f32)
        .collect()
}

pub fn fonts_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts")
}

pub fn font(name: &str) -> Vec<u8> {
    let path = fonts_dir().join(format!("lmroman10-{name}.ttf"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

pub fn out_path(candidate: &str) -> PathBuf {
    let dir = std::env::args().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!("test-page-{candidate}.pdf"))
}
