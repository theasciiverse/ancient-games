# The Ancient Games rulebook

The rules of every game in the collection, with where each rule comes
from: what survives, what is reconstructed and by whom, and what we chose
where every source is silent.

One source, several outputs. `./build.sh` writes to `out/`:

- `ancient-games-rules.pdf`: the whole book (pandoc, then Typst)
- `ancient-games-rules.html`: the whole book as one self-contained page
- `<game>.html`: one game's chapter, for its catalog page
- `<game>.txt`: one game's chapter as plain text, meant for `RULES.txt`
  in that game's release archive
- `<game>-help.txt`: the chapter's short form, copied to the game's crate
  as `HELP.txt`, which the game shows under F1

Needs pandoc 3.1.2 or newer, Typst, Python 3 and cargo (the build compiles
the games to draw their examples and screens). `RULEBOOK_OUT=dir` builds
elsewhere and leaves the crates' `RULES.txt` alone.

## Files

- `chapters/`: one Markdown file per game, after `00-introduction.md`
- `references.bib`: every source cited; a source only seen cited in
  another is marked there and cited "as cited in" that source
- `pictures.txt`: what the build draws from the games themselves. Each
  game takes `--rulebook-examples` (worked examples: positions set up,
  played by the real rules engine, printed as text, into `examples/`)
  and `--rulebook-screen RULES:SECONDS` (the computer playing a rule set
  on a simulated clock, drawn as SVG, into `screens/`); both come out the
  same on every build. A chapter shows an example with
  ```` ```{.example file="senet/swap"}``` ```` (`filters/examples.lua`)
  and a screen as an image; examples appear in every format and the F1
  help, screens only in the PDF and HTML
- `bib/<game>.bib`: each chapter's own sources, beside `references.bib`
- `figures/make_figures.py`: draws the boards as SVG and as ASCII, from
  the same geometry as the game's code; the build reruns it
- `images/`: photographs, each with its source, author and licence in
  `images/CREDITS.md` and in its caption; the PDF and HTML show them,
  the text and the F1 help leave them out
- `filters/`: `no-dashes.lua` (no en or em dashes in any output; the
  build also fails if one slips through), `text-figures.lua` (ASCII
  boards and capital headings in the text output)
- `style/rulebook.css`: the HTML pages
- `style/book.typ`: the PDF's Typst set-up (chapter titles over the
  ornament, each on a new page; "About this book" at the foot of its page;
  no number on page 1), passed to pandoc with `-H`; it is not in
  `metadata.yaml`, because pandoc's `-H` replaces the metadata's
  header-includes
- `style/titlepage.typ`: the PDF's title page; build.sh fills in the
  games (from the chapters' headings) and the date

## Writing a chapter

**Every game's chapter has a short form for F1** (David, 2026-10-22). The
chapter is the full story; a player who presses F1 mid-game wants the
rules in seconds. Write it into the chapter in fenced divs of class
`brief`:

    ::: brief
    - 6 pits a side, 4 seeds each. Sow anticlockwise ...
    :::

one before `## What survives` (how the game is played at all, and that
the full rules, sources and history are in RULES.txt and the rulebook),
one in each rule set's section under its facts table (3 to 8 bullets;
the heading must match the rule set's `section` in `rules.rs`, where F1
opens), one for any shared basics a rule set's brief leans on, and the
keys at the start of `## Playing it in the terminal`. The briefs say what
the program does, "Our choice" included. `build.sh` gathers them, under
their headings, into `HELP.txt` (`filters/help.lua`) and leaves them out
of the book and of `RULES.txt` (`filters/no-brief.lua`); it stops with an
error if a game's chapter has none. Mancala's chapter is the model.

Keep "what survives", "what is reconstructed" and "Our choice" apart, and
mark every choice of ours with those words. Cite with pandoc's
`[@key, page]`. The rules in the book must be the rules the program
plays: read the game's `rules.rs` before writing, and say where a
source and the program differ.
