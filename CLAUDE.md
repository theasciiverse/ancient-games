# Ancient games

Terminal versions of the classic board games of the world, starting with
the oldest: the Royal Game of Ur, Senet, Mehen, Hnefatafl and Fox &
Geese (David, 2026-10-20), then the mancala games (2026-10-22). More to
follow.

**Detail lives in `NOTES.md`** (not loaded automatically): release
history, the sources read and choices made for each game, measured
strengths, per-game code notes, the rule sets as first researched. Read
the game's section there before changing its rules or computer player,
and put new detail of that kind there, not here.

**Status (2026-10-25):** fifteen games playable, each its own binary, all
with the computer at three strengths or two players at one keyboard, keys
and mouse, a menu of rule sets with sources, and help: Royal Game of Ur
(`ur/`), Senet (`senet/`), Fox & Geese (`fox-and-geese/`), Hnefatafl
(`hnefatafl/`), Mehen (`mehen/`), Mancala (`mancala/`, nine two-rank
games as one title), Morris (`morris/`, five forms as one title),
Mū tōrere (`mu-torere/`), Kōnane (`konane/`), Seega (`seega/`), and,
to make fifteen (2026-10-25, David: "Lets only take it up to 15 in total
for now"), Alquerque and Fanorona (`alquerque/`), Tabula (`tabula/`),
Pachisi and chaupar (`pachisi/`), Shatranj (`shatranj/`), Patolli
(`patolli/`).

Version 0.6.0 (2026-10-26: the ornament and boards that grow with the
terminal), tag v0.6.0-av.1. On the website mockup as fifteen titles
in the collection `ancient-games`, previews from each game's `--demo` by
`asciiverse/record-preview.sh` (seed 3, Mehen 4; one take each, David:
do not overdo the recordings), local Linux builds in the website's
dev-releases/ancient-games. Release workflow `.github/workflows/release.yml`
(one archive per game, static musl Linux, macOS, Windows, x86-64 and
ARM64); Actions billing blocks builds, so the GitHub releases hold only
the rulebook PDF until a re-run of v0.6.0-av.1 adds the archives.
Licence: GPL-3.0-or-later for the whole workspace (David, 2026-10-27). Own git repo, branch
`main`, private at github.com/theasciiverse/ancient-games.

## Ornament

`ORNAMENT.md` (2026-10-26): braille ornament after each game's real
object, sourced like the rules. Started: `core/src/ornament.rs` (blank
cells only; O key and `--plain` in every game), tried on Ur (rim
lozenges; rosettes in the margins beyond 100x30), checked against the
British Museum record and a CC0 photograph; then the other ten the same
day, each after a museum object or document (table in ORNAMENT.md, notes
in `~/references/ancient-games/ornament/`). Plain: Seega, Mancala, Kōnane,
Mū tōrere, Morabaraba, Fanorona (living traditions: David's call).
Boards grow with the terminal (2026-10-26, after David's look at 190x58):
every game, the target 100x30 unchanged (each rulebook picture compared
byte for byte).

## Roadmap

`ROADMAP.md` (2026-10-22): ancient games from around the world that could
join the collection (Hounds and Jackals, mancala, the Roman games, Patolli,
Liubo, the morris family and more), with what a GitHub search found for
each and a suggested order. David: eventually perhaps all of them. Its
history notes are unchecked; verify before building.

## Rulebook

`rulebook/` (started 2026-10-22): the rules of each game with sources,
one Markdown chapter per game plus a BibTeX file, built by
`rulebook/build.sh` into a PDF (pandoc and Typst), HTML (book and one page
per game) and plain text per game (meant for `RULES.txt` in the release
archives). Board figures come from
`rulebook/figures/make_figures.py` as SVG and ASCII. Done: introduction,
all five games (2026-10-22; Mehen, Hnefatafl and Fox & Geese researched
and written by subagents from the sources, then checked against the code),
Mancala, Morris (2026-10-23), Mū tōrere, Kōnane, Seega, and the five of
2026-10-25 (Alquerque and Fanorona, Tabula, Pachisi, Shatranj, Patolli,
chapters 11 to 15).
Illustrations (David, 2026-10-22): photographs of the real objects in the
PDF and HTML, each with source, author and licence in
`rulebook/images/CREDITS.md` and its caption (CC0 museum images first:
the Met, the Brooklyn Museum, CC0 Commons photos); the text and F1 help
leave them out. Pictures from the games themselves (David, 2026-10-22):
`rulebook/pictures.txt` lists worked examples (`--rulebook-examples`,
each game's `examples.rs`, drawn by the real rules engine) and screens
(`--rulebook-screen RULES:SECONDS`, `core/src/capture.rs`: a demo on a
simulated clock drawn to SVG; Hnefatafl's and Fox & Geese's computer is
waited for and searches to a fixed depth there, so pictures are the same
on any machine).

Where the program departs from its sources (found while writing the
book, each marked "Our choice" in its chapter; candidates for changing
the code, David's call): Hnefatafl: the empty throne is hostile in Bell's
Tablut too; a side that cannot move loses in every set; no draw for a
game that cannot be ended (Copenhagen, Fetlar); Brandubh counts any third
repetition, the WTF text says "for the third time in a row", and no
two-game match. Fox & Geese: the fox always starts on D3 (Bell: any
vacant point); a third repetition is a draw (Bell silent).

The F1 help is the short form of the chapter (David, 2026-10-22: "we
need an abbreviated form in the F1"): each chapter carries "brief"
blocks (fenced divs, `::: brief`), one under each rule set's heading plus
an opening and the keys; build.sh gathers them into `<crate>/HELP.txt`,
which the game `include_str!`s (`ui::HELP`) and shows scrollable
(`shell::rules_help`, `draw_help_at`, `help_key`), opened at the heading
named by the rule set's `section`; tests check every section is there and
no line is wider than the help. The full chapter goes to
`<crate>/RULES.txt` for the release archive, without the briefs. **Every
future game's chapter must have briefs**; build.sh fails without them
(see `rulebook/README.md`, "Writing a chapter").

## Code

Cargo workspace. `core/` (`ancient-core`) holds what the games share:
the seedable RNG, players and strengths, drawing helpers and colours,
`ornament`, and `shell` (command line, the menu, the help overlay, the
terminal loop behind the `Program` trait). Each game has `rules.rs`
(tested, rule sets as data in `Rules`), `ai.rs`, `app.rs` (phases, the
computer thinking), `ui.rs`, `examples.rs` (rulebook examples) and
`main.rs`. Each game's `strengths` test (`--ignored`) measures the
computer's levels against each other; results in `NOTES.md`.

Options in every game: `--seed N`, `--demo` (computer against computer,
restarts), `--quit-after SECONDS`, `--plain`;
`RUST_LOG=debug <binary> 2>game.log`

## Why build them

Checked 2026-10-20 (GitHub search, plus the hub catalog and
`external/`): no good terminal version of any of them exists. Royal Game
of Ur: one OCaml student remake (2021). Senet, Mehen, Fox & Geese: none.
Hnefatafl: four small projects (farl-opa/hnefatafl-terminal in Rust,
network play only; RybazPuszczy/fetlar-hnefatafl-terminal in C++;
nyabla/hnefatafl in C, a university assignment; Nsgshsk/TaflSole in
C++). None of them has a licence, so none can be packaged, and none was
seen to have a computer opponent. Nothing in the catalog either.

## Settled

- **The player chooses the rules** (David, 2026-10-20). Wherever there
  is more than one honest version of a game, every one of them is
  offered at the start, each with a line on where it comes from:
  - Hnefatafl: Copenhagen (11x11 tournament rules, shield wall, exit
    forts), Fetlar (11x11), Brandubh (7x7, Irish), Linnaeus' Tablut (9x9,
    the 1732 diary as now retranslated); perhaps the 1811
    mistranslation ("except the king" for "likewise the king") as a
    historical curiosity
  - Senet: Bell's and Kendall's reconstructions, as they are (David:
    "if that is what we have, that is what we have"); no rules of our
    own invented to fill gaps
  - Fox & Geese: 13 or 17 geese, geese moving backwards or not
  - Mehen: the competing reconstructions (Hyena-game based, and James
    Masters' 2024 proposals), clearly marked as guesswork
- Each game says honestly what is known and what is reconstructed: a
  history page in the game is part of the design.

- **Decided by David, 2026-10-20:**
  - Rust + Ratatui, one collection repo with one catalog title per game
    (games/CLAUDE.md curation rule) and a shared core: board drawing,
    throwing sticks and dice, computer opponents
  - Collection name **Ancient Games** (checked 2026-10-20: no crate on
    crates.io, nothing on Steam, only a few tiny unrelated GitHub repos)
  - Royal Game of Ur first
  - Network play (iroh) later; first a computer opponent and two players
    on one keyboard, with the core shaped so network play fits in later

## Next for Ur

- [ ] Murray's and Masters' paths: need a square-by-square diagram first
      (the Masters page describes them only in words)
- [ ] Move animation, sound, preview, release workflow, hub entry

## Open questions (David's to decide)

- [x] Stack, name, first game, network play: see "Settled"
