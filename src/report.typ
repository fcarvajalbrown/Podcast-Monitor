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

#block(below: 0pt, text(9.5pt, style: "italic", fill: grey, inputs.header))
#block(above: 5pt, below: 0pt, line(length: 100%, stroke: 0.5pt + red))
#v(14pt)

#for block in inputs.blocks {
  if block.kind == "paragraph" { par(block.text) }
}
