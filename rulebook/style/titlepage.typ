// The book's title page (build.sh fills in the games and the date): the title
// over the chapters' ornament, a board from Ur as frontispiece, the games
// the book holds, and the date of this build.
// It goes in before pandoc's document set-up (build.sh passes it with
// -H), so it is page 1 with no number, and sets the book's font itself.
#page(paper: "a4", numbering: none, margin: (x: 2.4cm, top: 2.4cm, bottom: 2.6cm))[
  #set text(font: "Libertinus Serif", size: 11pt, lang: "en", region: "GB")
  #set align(center)
  #set par(justify: false)
  #text(size: 9.5pt, tracking: 0.4em, fill: rgb("#b5432a"))[ASCIIVERSE]
  #v(1.5cm)
  #text(size: 42pt)[Ancient Games]
  #v(0.15cm)
  // The chapters' ornament (metadata.yaml), which is defined after this.
  #align(center, box(width: 55%, grid(
    columns: (1fr, auto, 1fr), align: horizon, column-gutter: 7pt,
    line(length: 100%, stroke: 0.7pt + rgb("#b5432a")),
    rotate(45deg, square(size: 5.5pt, fill: rgb("#2b4c8c"))),
    line(length: 100%, stroke: 0.7pt + rgb("#b5432a")),
  )))
  #v(0.35cm)
  #text(size: 15pt, style: "italic")[The rules, and where they come from]
  #v(1.1cm)
  #block(stroke: 0.6pt + rgb("#b5432a"), inset: 5pt, radius: 2pt,
    image("images/ur-board-british-museum.jpg", width: 72%))
  #v(0.15cm)
  #block(width: 72%, text(size: 8pt, fill: luma(90))[
    One of Leonard Woolley's boards from the Royal Cemetery at Ur, from the
    middle of the third millennium BC, now in the British Museum
    (1928,1009.378). Photograph: BabelStone, CC0.
  ])
  #v(1fr)
  #block(width: 88%, text(size: 11pt, fill: rgb("#2b4c8c"))[@GAMES@])
  #v(0.9cm)
  #text(size: 9.5pt, fill: luma(90))[@DATE@]
]
