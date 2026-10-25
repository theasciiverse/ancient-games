// The book's title page (build.sh fills in the games and the date): the title
// over the chapters' ornament, a board from Ur as frontispiece, the games
// the book holds, and the date of this build.
// build.sh passes it with -B, after pandoc's set-up, so it is the
// first page; style/book.typ leaves page 1 without a number.
#set document(title: "Ancient Games: The rules, and where they come from", author: "David Sandquist")
#[
  #set align(center)
  #set par(justify: false)
  #text(size: 9.5pt, tracking: 0.4em, fill: rgb("#b5432a"))[ASCIIVERSE]
  #v(0.1cm)
  #text(size: 11pt, fill: luma(60))[David Sandquist]
  #v(1.5cm)
  #text(size: 42pt)[Ancient Games]
  #v(0.15cm)
  #ornament
  #v(0.35cm)
  #text(size: 15pt, style: "italic")[The rules, and where they come from]
  #v(1.1cm)
  #block(stroke: 0.6pt + rgb("#b5432a"), inset: 5pt, radius: 2pt,
    image("images/ur-board-british-museum.jpg", width: 66%))
  #v(0.15cm)
  #block(width: 72%, text(size: 8pt, fill: luma(90))[
    One of Leonard Woolley's boards from the Royal Cemetery at Ur, from the
    middle of the third millennium BC, now in the British Museum
    (1928,1009.378). Photograph: BabelStone, CC0.
  ])
  #v(1fr)
  // At least one blank line under the caption, even when the page is full.
  #v(1.3em)
  #block(width: 88%, text(size: 11pt, fill: rgb("#2b4c8c"))[@GAMES@])
  #v(0.9cm)
  #text(size: 9.5pt, fill: luma(90))[@DATE@]
]
#pagebreak()
