use pdf_spike::{PARAGRAPHS, SECTIONS, TABLE, TITLE};

fn main() {
    let body = PARAGRAPHS.join("\n\n");
    let mut table: Vec<String> = TABLE.iter().map(|row| format!("| {} |", row.join(" | "))).collect();
    table.insert(1, format!("|{}", "---|".repeat(TABLE[0].len())));
    let markdown = format!(
        "# {TITLE}\n\n## {}\n\n{body}\n\n## {}\n\n{}\n\n## {}\n\n{body}\n\n{body}\n\n{body}\n",
        SECTIONS[0],
        SECTIONS[1],
        table.join("\n"),
        SECTIONS[2],
    );
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let path = std::path::Path::new(&dir).join("test-page-reference.md");
    std::fs::write(&path, markdown).unwrap();
    println!("{}", path.display());
}
