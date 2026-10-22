// The book's Typst set-up, passed to pandoc with -H before the title page
// (build.sh). It lives here rather than in metadata.yaml's
// header-includes, which pandoc replaces when -H is given.
// Each chapter starts on a new page, its title large and centred over
// an ornament: a carnelian rule with a lapis diamond, as on the boards.
#let ornament = align(center, box(width: 55%, grid(
  columns: (1fr, auto, 1fr), align: horizon, column-gutter: 7pt,
  line(length: 100%, stroke: 0.7pt + rgb("#b5432a")),
  rotate(45deg, square(size: 5.5pt, fill: rgb("#2b4c8c"))),
  line(length: 100%, stroke: 0.7pt + rgb("#b5432a")),
)))
// "About this book" is the exception: it sits at the foot of the page it
// falls on, under a smaller title (chapters/00-introduction.md).
#show heading.where(level: 1): it => {
  if it.has("label") and it.label == <about-this-book> {
    align(center, text(size: 1.5em, weight: "regular", it.body))
    v(-0.2cm)
    ornament
    v(0.2cm)
  } else {
    pagebreak(weak: true)
    v(0.8cm)
    align(center, text(size: 2.3em, weight: "regular", it.body))
    v(-0.1cm)
    ornament
    v(0.5cm)
  }
}
// The title page (style/titlepage.typ) has no page number.
#set page(footer: context if here().page() > 1 {
  align(center, counter(page).display())
})
// Worked examples line up in columns: one monospaced font with every
// symbol they use, no fallback.
#show raw: set text(font: "DejaVu Sans Mono", fallback: false, size: 0.82em)
// Tables and their cells to the left, like the text.
#set table(align: left)
#show table: set align(left)
