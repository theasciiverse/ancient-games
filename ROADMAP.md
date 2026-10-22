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
- [ ] **Seega**: an Egyptian game of placing and capturing. One small
  student project.

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
- [ ] **Kōnane**: Hawaii. One small project.
- [ ] **Mū tōrere**: Māori. One small project.

## Africa and the Mediterranean

- [ ] **The mancala family**: Oware, Bao and others, with many regional
  rule sets; a collection of the real regional games, each with its own
  rules. A few terminal versions exist, the best a Go TUI of plain
  Mancala (Broderick-Westrope/mancala-go, GPL-3.0, 10 stars).
- [ ] **Fanorona**: Madagascar. One console project.
- [ ] **Alquerque**: the ancestor of draughts. One small console
  project.
- [ ] **Nine Men's Morris** and its kin (Three, Six and Twelve Men's
  Morris): a few console projects, the best with 11 stars
  (rajko-z/nine-mens-morris-python, MIT).

## Suggested order

1. Hounds and Jackals: a third Egyptian game.
2. A mancala collection, each region's honest rules.
3. The Roman games, which have the competing reconstructions this
   collection is built around.
4. Patolli and Liubo: two lost dice games from opposite sides of the
   world.
5. The morris family: the simplest to build.

Each new game follows the first five: its own binary in this workspace,
the computer at three strengths or two players, every honest rule set in
the menu, a rulebook chapter with sources, worked examples and screens,
and its own catalog title in the `ancient-games` collection.
