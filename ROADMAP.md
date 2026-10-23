# Roadmap: more ancient games

Games from around the world that could join the collection, recorded on
2026-10-22. David: "eventually, we might do all."

**What was checked:** a GitHub search on 2026-10-22 for each game's name,
counting repositories whose name or description says terminal, console,
CLI, TUI, curses or command line ("nothing found" means none). That
search can miss projects described in other words. The hub catalog
already has chess (`chess-tui`), Go (`termsuji`), backgammon, checkers,
reversi and gomoku, and none of the games below.

**What was not checked:** the history notes here are from memory. Before
building any of them, read the sources and check each fact, as for the
first five games (see `CLAUDE.md`, "What survives of the rules").

## Egypt and the Near East

Beside the Royal Game of Ur and Senet.

- [ ] **Hounds and Jackals, or 58 Holes**: a race game with pegs, found
  in Egypt and Mesopotamia. Nothing found.
- [x] **Seega**: `seega` (2026-10-23), Lane's 1836 account on his
  three boards (5, 7, 9); see `CLAUDE.md`. Not yet read: Bell's account,
  and anything dating the game before Lane.

## Greece and Rome

Several competing reconstructions each, which suits the rule-set menu.

- [ ] **Ludus latrunculorum**: a Roman war game. One CLI client, no full
  game.
- [ ] **Duodecim scripta, or Tabula**: Roman ancestors of backgammon.
  Nothing found.
- [ ] **Petteia**: Greek. Nothing found.
- [ ] **Pente grammai**: Greek. Nothing found.

## China and India

- [ ] **Liubo**: a Han-dynasty dice game, its rules lost and debated.
  Nothing found.
- [ ] **Chaupar**: the cross-shaped race game that Pachisi and Ludo come
  from. Ludo console games exist; Chaupar itself, nothing found.
- [ ] **Chaturanga**: an early form of chess. Nothing found.
- [ ] **Shatranj**: the Persian and Arabic form of chess. One student
  client.

## The Americas and the Pacific

- [ ] **Patolli**: the Aztec race game. R. C. Bell gives suggested rules
  in *Board and Table Games from Many Civilizations* (pp. 6-8 of the
  1969 edition, seen while reading for Ur). Only client projects found.
- [ ] **Puluc**: a Maya race game. Nothing found.
- [x] **Kōnane**: `konane` (2026-10-23), four rule sets (Ernst 8x8,
  Modern, Puʻuhonua o Hōnaunau 9x11, King's papamū 14x17); see
  `CLAUDE.md`. Not yet read: Emory 1924, Buck 1957.
- [x] **Mū tōrere**: `mu-torere` (2026-10-23), two rule sets (the
  pūtahi beside an enemy; Best 1925 as written); see `CLAUDE.md`. Not
  yet read: Ascher 1987, Harko Brown 2008, Browne 2023.

## Africa and the Mediterranean

- [x] **The mancala family**, two-rank games: `mancala` (2026-10-22),
  nine rule sets from their sources (see `CLAUDE.md`). A few terminal
  versions existed, the best a Go TUI of plain Mancala
  (Broderick-Westrope/mancala-go, GPL-3.0, 10 stars).
- [ ] **Mancala, four-rank games**, version 2 of `mancala`: Chisolo
  (Bell 1969, rules read), Omweso (Uganda; Wikipedia contradicts itself on
  where captured seeds are sown from: needs Nsimbi 1968 or de Voogt 1997),
  Bao la Kujifunza, Bao la Kiswahili (de Voogt 1995).
- [ ] **Fanorona**: Madagascar. One console project.
- [ ] **Alquerque**: the ancestor of draughts. One small console
  project.
- [x] **Nine Men's Morris** and its kin: `morris` (2026-10-23), six
  rule sets from their sources (see `CLAUDE.md`, "Morris"). A few console
  projects existed, the best with 11 stars
  (rajko-z/nine-mens-morris-python, MIT).

## Suggested order

1. Hounds and Jackals: a third Egyptian game.
2. A mancala collection, each region's honest rules.
3. The Roman games, which have the competing reconstructions this
   collection is built around.
4. Patolli and Liubo: two lost dice games from opposite sides of the
   world.
5. ~~The morris family: the simplest to build.~~ Done, 2026-10-23.

Each new game follows the first five: its own binary in this workspace,
the computer at three strengths or two players, every honest rule set in
the menu, a rulebook chapter with sources, worked examples and screens,
the chapter's short form under F1 ("brief" blocks, see
`rulebook/README.md`), and its own catalog title in the `ancient-games`
collection.
