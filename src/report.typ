#import sys: inputs

#let grey = rgb("#666666")
#let red = rgb("#B70002")
#let frame = 2.5cm + 6pt

#set document(title: inputs.title)
#set page(
  paper: "a4",
  margin: frame,
  footer: none,
  foreground: context place(top + center, dy: 29.7cm - 1.4cm, text(9pt, fill: grey, top-edge: "baseline", counter(page).display())),
)
#set text(font: "LM Roman 10", size: 10.5pt, lang: "es", region: "cl", top-edge: 1em, bottom-edge: "baseline", overhang: false)
#set par(justify: true, leading: 4.5pt, spacing: 12.5pt)

#show heading: set text(weight: "bold")
#show heading: set par(justify: false)
#show heading.where(level: 1): set text(20pt)
#show heading.where(level: 1): set par(leading: 4pt)
#show heading.where(level: 1): set block(above: 14pt, below: 18pt)
#show heading.where(level: 2): set text(15pt)
#show heading.where(level: 2): set par(leading: 4pt)
#show heading.where(level: 2): set block(above: 20.5pt, below: 12pt)
#show heading.where(level: 3): set text(12.5pt)
#show heading.where(level: 3): set par(leading: 3.5pt)
#show heading.where(level: 3): set block(above: 16.5pt, below: 9.5pt)

#let booktabs(rows, weights) = block(above: 12pt, below: 22pt, breakable: rows.len() > 12, {
  set text(9pt)
  set par(justify: false, leading: 2.5pt)
  table(
    columns: weights.map(weight => weight * 1fr),
    stroke: none,
    inset: (x: 4pt, top: 3pt, bottom: 5.5pt),
    align: left + top,
    table.hline(stroke: 0.6pt),
    table.header(..rows.at(0).map(cell => text(weight: "bold", cell))),
    table.hline(stroke: 0.4pt),
    ..rows.slice(1).flatten(),
    table.hline(stroke: 0.6pt),
  )
})

#block(below: 0pt, text(9.5pt, style: "italic", fill: grey, inputs.header))
#block(above: 5pt, below: 14pt, line(length: 100%, stroke: 0.5pt + red))

#for block in inputs.blocks {
  if block.kind == "heading" { heading(level: block.level, block.text) }
  if block.kind == "paragraph" { par(block.text) }
  if block.kind == "table" { booktabs(block.rows, block.weights) }
  if block.kind == "pagebreak" { pagebreak(weak: true) }
}
