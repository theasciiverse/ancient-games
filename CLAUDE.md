# Ancient games

Terminal versions of the classic board games of the world, starting with
the oldest: the Royal Game of Ur, Senet, Mehen, Hnefatafl and Fox &
Geese (David, 2026-10-20), then the mancala games (2026-10-22). More to
follow.

**Status (2026-10-20):** four games playable, each its own binary, all
with the computer at three strengths or two players at one keyboard, keys
and mouse, a menu of rule sets with sources, and help:

- Royal Game of Ur (`ur/`, `royal-ur`): Finkel 2017, British Museum
  1990s, R. C. Bell's betting game, the Babylonian tablet game
- Senet (`senet/`, `senet`): Kendall with five pieces (the modern
  normative rules), Kendall with seven (Walker's account), Bell 1960
- Fox & Geese (`fox-and-geese/`): Bell's 13 and 17 geese
- Hnefatafl (`hnefatafl/`, `hnefatafl`): Copenhagen, Fetlar, Brandubh,
  Tablut after Linnaeus, Tablut after Bell 1960
- Mehen (`mehen/`, `mehen`): James Masters' 2024 reconstruction for two,
  four or six players (two teams; the person plays a whole team)
- Mancala (`mancala/`, `mancala`, 2026-10-22): nine two-rank sowing games
  as one title (David's choice over one title per game), each from a
  source: Oware Abapa, Wari after Bell, Lane's two Cairo games of
  mankal'ah, Kalah after Champion's patent, Congkak after Hellier,
  Pallanguli after Durai, Toguz Kumalak, Bohnenspiel; see "Mancala" below

All five games first planned are in, and Mancala since 2026-10-22 (on
the mockup as a sixth title, NEW! banner, cast `asciiverse/casts/mancala.cast`
seed 3, dev-releases archive built at `8a6be4a`, with the other five rebuilt there too under the same 0.1.0-av.1;
the pushed tag v0.1.0-av.1 predates it, so the next release needs a new
tag). On the website mockup since 2026-10-20 as five
separate titles in the collection `ancient-games` (hub's collections.toml;
entries in the mockup's dev/mock-titles.toml with the other Asciiverse
originals, licence undecided, NEW! banner), previews from each game's
`--demo` recorded by `asciiverse/record-preview.sh` (casts in
`asciiverse/casts/`, seed 3, Mehen seed 4), local Linux builds in the
website's dev-releases/ancient-games (0.1.0-av.1, built from a fresh clone
with --remap-path-prefix). Release workflow `.github/workflows/release.yml`:
one archive per game for static musl Linux, macOS and Windows, x86-64 and
ARM64; tag v0.1.0-av.1 pushed, no GitHub release while Actions billing
blocks builds. Licence: not chosen; David chooses licences case by case
(2026-10-20), so the Cargo.toml files name none until he does. Own git repo, branch `main`, private at github.com/theasciiverse/ancient-games.

## Mancala

One crate, one binary, one catalog title (David, 2026-10-22: "one title,
many rules", like Hnefatafl). Two-rank games first; the four-rank games
of East and Southern Africa (Chisolo, Omweso, Bao la Kujifunza, Bao la
Kiswahili) are version 2, in `ROADMAP.md`. Plan:
`~/.claude/plans/do-you-wan-tto-eager-finch.md`. Rulebook chapter
`rulebook/chapters/06-mancala.md`, sources in `rulebook/bib/mancala.bib`.

**Code.** `rules.rs` holds the rule sets as data (`Rules`: setup,
origin skipping, laps (single, relay, Lane's 3/5+, Indian), capture kind,
cow, tuzdyk, grand slam (allowed, no capture, illegal), ending (feed,
sweep, opponent takes, Lane's gift, Hellier's restock), match (single,
points, refill rounds), clockwise) and one engine. Pits are numbered in
the order of sowing, South 0 to n-1, North n to 2n-1; a clockwise game is
the same game drawn mirrored. Every change to the board goes through
`act`, which records a `Step`; `apply` replays it, so the animation
(`app.rs`), the rulebook examples (`examples.rs`) and the tests (each
checks the replay) cannot disagree with the rules. `ai.rs`: alpha-beta,
iterative deepening, transposition table with bounds; an extra move from
one's own store is searched as the same side again; the judgement is the
stores' difference. Root moves are searched against the best less one, so
random tie-breaking only ever chooses among equals (a first version
passed the best itself and let noise pick a fail-low move: hard lost to
easy 9 of 10 at Oware, found by the strength test, 2026-10-22).

**Sources read (2026-10-22), and what each settled:**
- Oware Abapa: the Oware Society's rules (oware.org/abapa.asp, Wayback
  2003 and 2026) and Aualé (joansala.com). Grand slam sown, no capture
  (the Society's "beyond five houses one forfeits everything"; we follow
  its stated reason, no capture whenever the other side would be left
  with nothing); feeding obligatory; no feed possible: each takes own
  side (Aualé); cycles: each keeps own side (we take a third repetition).
- Wari: Bell 1969 pp. 116-121; the move capturing everything is illegal
  (Bell; Ludii's summary of G. T. Bennett's chapter in Rattray 1927,
  pp. 382-390, not itself read); if every move does it, allowed (as
  Romein and Bal). Rattray's book is lending-only on archive.org.
- Lane 1836 (Gutenberg 70796, p. 315) read in full: two modes. Bell
  pp. 113-114 merges them and invents a rule; the program follows Lane.
- Kalah: Champion's patent US 2,720,362 (filed 1951, granted 1955):
  3 to 6 seeds, series to 40 by seeds won from the opponent, first move
  alternates; silent on an empty facing pit (we capture nothing).
- Congkak: Hellier 1907 (JSBRAS 49: 93-94), the Malay game: 7x7,
  simultaneous play (we take turns), capture of the facing hole only,
  a player whose row is empty refills from her house leaving ruined
  wells, until she cannot. Overbeck 1915 is the Ceylonese game; Skeat
  1900 has 8 holes; Culin 1900 (Philippines) has no captures; Parker 1909
  (Colombo) one round, no extra turn.
- Pallanguli: Durai 1928 (Man 28, no. 135, pp. 185-186) read in full:
  a capture ends the move (Bell, who follows her, continues: we follow
  Durai); cows of four to the side's owner; first move alternates by
  round. Modern pallanguzhi (12 seeds with 2 in the middle, pasu of six)
  has no good source and is not offered.
- Toguz Kumalak: the Kazakh competition rules, Astana 2013 (9kumalak.com
  via Wayback), our translation: first stone back in its hole, single
  stone moves on, even capture, tuzdyk (makes 3, not No. 9, once, not
  the mirror), atsyrau (no move: the opponent makes an extra move and
  takes all; we give him all at once). The 1949 Auezov codification on
  Wikipedia is not supported by that text's preface.
- Bohnenspiel: Jahn 1917 pp. 14-15 (Commons scan, Fraktur, transcribed):
  captures of 2, 4, 6 on either side in an unbroken run; the rest from
  igGameCenter.
- History: Depaulis 2020 and 2024 (BGSJ), de Voogt 2021 (BGSJ), Hyde
  1694, Culin 1896, Jobson 1623, Parker 1909, Alvares 1881, Christiano
  2010, Waweru 2024. Findings: 'Ain Ghazal, the Egyptian temple holes and
  three-row boards are not mancala; Culin's Ethiopian "mancal" is in
  Alvares a distance ("a game of manqual"), a horseshoe game; earliest
  likely boards Aksumite, 6th to 8th c. (de Voogt: speculative). The
  Kitab al-Aghani mention rests on Culin alone; not found.
- Not reached: Herskovits 1932, Townshend 1979, Pankhurst 1971, Murray
  1952 (lending-only), Bernardo 1937 (Sungka), Arbeiter and Ruhnke 1937,
  the Kyrgyz federation rules. Mancala World (fandom) blocks scripted
  fetches; the browser would be needed.

**Measured strengths** (2026-10-22, 10 single games each, sides
alternating; `cargo test --release -p mancala strengths -- --ignored
--nocapture`, about half an hour): normal beats easy 10/10 at Oware,
Toguz and Bohnenspiel, 9/10 at Kalah; hard beats easy 10/10 (Toguz 9/10);
hard beats normal 10/10 at Oware, Toguz and Bohnenspiel, but only 6/10
(1 drawn) at Kalah, where four moves already see most of it. Before the
root-window fix, hard beat easy 1/10 at Oware.

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
and written by subagents from the sources, then checked against the code).
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
the seedable RNG, players and strengths, drawing helpers and colours, and
`shell` (command line, the menu, the help overlay, the terminal loop
behind the `Program` trait). Each game has `rules.rs` (tested), `ai.rs`,
`app.rs` (phases, the computer thinking), `ui.rs` and `main.rs`.

- `ur/src/rules.rs`: board, Bell's path (positions 0 start, 1 to 14,
  15 home; 5 to 12 shared), lots, rule sets as data (`Rules`), moves;
  tests
- `ur/src/ai.rs`: `Strength::Easy` a beginner (takes, rosettes, else
  the furthest piece), `Normal` a one-move judgement (progress, rosettes,
  danger on the shared row), `Hard` expectimax two turns deep; extra
  throws are free only twice in a row (chains of rosettes made the search
  explode and the game stall, found in a demo run). Measured 2026-10-20:
  easy beats random 189/200, normal beats easy 162/200, hard beats easy
  81/100 and normal 53/100 (the dice decide much; a deeper search adds
  little)
- `ur/src/app.rs`: menu choices, phases (throw, tumble, choose, no move,
  computer thinking, over), seedable RNG; test plays computer games to
  the end through the phases
- `ur/src/ui.rs`: the board (lapis, shell, carnelian rosettes, the
  pieces' five dots), trays in the gaps of the outer rows, lots, menu,
  help; square size follows the terminal, 100x30 and up
- Hnefatafl: rules as data per rule set (`Rules`: escape to corners or
  edge, king captured by four or like any piece, shield wall, exit fort,
  encirclement, what a third repetition means); the throne may be passed
  when empty; corners hostile to all, the king too only in Brandubh.
  Setups from OpenTafl's source. Bell 1960 leaves out who moves first,
  passing the throne, an armed king and repetitions; the help says what
  was chosen. Copenhagen's "perpetual repetition" is read as a third
  repetition. AI: alpha-beta with iterative deepening (easy depth 1 with
  slips, normal 2, hard up to 8 in 1.5 s), judging king distance to an
  escape in king moves, clear ways out, attackers beside the king,
  material (a defender worth two attackers) and corner guards. Measured
  2026-10-20 (one game each): in Copenhagen, Brandubh and Linnaeus' Tablut
  hard beat easy and normal from both sides; in Fetlar hard attackers drew
  with normal defenders; Bell's Tablut favours the king (his side won
  every game but hard attackers against easy)
- Mehen: the 70-square coil is generated (14 by 8 grid, one gap between
  coils, head near the middle). Positions 0 (start), 1 to 70, 71 reborn.
  Choices where Masters is silent: the side to start by a coin; a lion
  sent back with its pharaoh stays on it; if every free pharaoh of the
  guessing team is held by a lion, a right guess moves nobody (the lion
  may still be set); the computer always sets a lion when it can (easy
  forgets a third of the time). Two people at one keyboard get a "look
  away" screen before each hiding. AI: the hiding and guessing mixes are
  the equilibrium of the one-hand game (fictitious play; a right guess
  costs the hider the steps plus about 6 squares for a lion), easy random,
  hard half the time best-responds to the opponent's habits; finding:
  with no lion at stake, hiding one marble never pays. Measured 2026-10-20
  (400 games each): two players: normal beats easy 59%, hard beats easy
  85% and normal 68%, about 60 hands a game; six players: hard beats easy
  97%, about 150 hands
- Options in every game: `--seed N`, `--demo` (computer against computer,
  restarts), `--quit-after SECONDS`; `RUST_LOG=debug <binary> 2>game.log`

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

## What survives of the rules (sources checked 2026-10-20)

- **Royal Game of Ur rule sets to offer** (read 2026-10-20: Finkel's paper,
  `references/ancient-games/Finkel - On the Rules for the Royal Game of
  Ur.pdf` with text in `finkel-ur.txt`; mastersofgames.com/rules/
  royal-ur-rules.htm; en.wikipedia.org/wiki/Royal_Game_of_Ur). Board for
  the first five: the 3rd-millennium Ur board (3x8 grid without the 5th and
  6th squares of the outer rows: 4x3 block, 2-square bridge, 2x3 block; five
  rosettes: outer rows 2nd from the right and 1st from the left, middle row
  4th from the left). Entry on the outer row at the 4th square from the left,
  moving left; at the corner rosette into the middle row the other way.
  - Finkel v. Tom Scott 2017 (the best known): Bell's path (14 squares,
    bearing off the 15th move), 7 pieces, four binary lots (0 to 4, 0
    loses the turn), any rosette gives another throw, the middle rosette
    is safe, one piece per square, enter any time onto a vacant square,
    must move if able, exact throw to bear off (Wikipedia: "exactly the
    number of spaces remaining until the end of the course plus one"),
    capture is optional
  - British Museum 1990s (Finkel, with the replica board): the same but 5
    pieces and no safe middle rosette
  - R. C. Bell: Bell's path, 7 pieces, three binary lots read as 0 = 4
    and throw again, 1 = no move and the turn ends, 2 = 1 and throw again,
    3 = 5 and throw again; a betting game: stakes in a pool, enter only on
    a 5, a fine into the pool for landing on a rosette, captured pieces
    re-enter only on a 5, first home takes the pool
  - H. J. R. Murray: round the outside of the 2x3 block, back down the
    middle row and off where it entered: 27 squares, bearing off the 28th
  - J. Masters: Murray's route shortened: at the penultimate outer square
    cross to the opposite row, round the 2x3 block, off from the starting
    square on the other side: 16 squares, bearing off the 17th; a rosette
    every fourth square; three lots read 0 = 4, 1, 2, 3; rosettes chain
    further throws (the same piece need not move); enter any time onto a
    vacant square; must move if able; exact throws to bear off
  - The Babylonian tablet game (BM 33333B, 177-176 BC), Finkel's
    reconstruction: on the later Twenty Squares board (two start rows of
    four feeding one central run of twelve; rosettes there), five
    different pieces each: Swallow enters on 2 (on the first rosette, or
    on the square before any rosette), Storm-bird on 5 (House 5, the
    first square of the central twelve), Raven on 6, Rooster on 7, Eagle
    on 10; dice: a sheep and an ox astragal (Finkel's "double or quits":
    the sheep die gives 1 to 4, the ox die's "yes" turns 1, 2, 3, 4 into
    5, 6, 7, 10); a betting game: tokens won from the pool when a piece
    lands on a rosette, paid in when it must pass one (Table 3.1 scale:
    Swallow 3, Storm-bird 4, Raven 4, Rooster 4, Eagle 5); capture sends
    a piece back (Finkel: "more than plausible"). The fortunes go with it:
    Swallow "a woman will love those who linger in a tavern", Storm-bird
    and Raven "enough food for the pack", Rooster "enough fine beer",
    Eagle "the pack will eat its fill of meat"; the twelve central squares
    carry the zodiac legends (I Pegasus "One who sits in a tavern" ... IX
    Scorpius "You will draw fine beer" ... XII Aquarius "You will cut
    meat"). Several details are Finkel's proposals; say so in the game.
- **Royal Game of Ur:** the best documented. A cuneiform tablet written
  by the Babylonian scribe Itti-Marduk-balāṭu in 177-176 BC (copied from
  an older tablet of Iddin-Bēl) describes the rules; it was catalogued as
  bird omens for decades. Irving Finkel (British Museum) reconstructed
  the rules from it and the board's shape. Race game, seven pieces each,
  four tetrahedral dice; the tablet adds fortunes for squares ("You will
  draw fine beer"). Movement direction is not fully clear from the
  tablet. Sources: en.wikipedia.org/wiki/Royal_Game_of_Ur, Finkel's "On
  the Rules for the Royal Game of Ur".
- **Senet rule sets to offer** (read 2026-10-20):
  - Kendall, seven pieces (Timothy Kendall, Passing Through the
    Netherworld, Kirk Game Company 1978; the booklet is not online, read
    through Damian Walker's account, psion.snigfarp.karoo.net/senet,
    printed 2007 and kept at cs.brandeis.edu/~storer/JimPuzzles/GAMES/
    Senet/SenetReferences/WalkersRulesKendal.pdf): 7 each alternating on
    1-14, white on 1, black first; sticks: light sides up, none = 5; land on
    an isolated enemy piece and it is bumped back to where the mover came
    from; a piece next to a friend is protected; only if no forward move,
    a piece must move backwards; 26 House of Beauty must be landed on; 27
    House of Humiliation (water): the turn ends, no other piece of that
    player may move until it is rescued (to 15 instead of throwing, unless
    15 is occupied, or borne off on an exact 4); bear off from the last
    five squares on an exact throw; 28 only off on 3, 29 only on 2, 30 on
    any throw; bumping on 28-30 sends the piece to 27 (or 26 if 27 is
    taken); pieces on 27 never protected; throw again after moving on 1,
    4, 5; first with all off wins
  - Modern Kendall, five pieces (devinokeefe/senet RULES.md, MIT, 2026:
    "the exact, normative ruleset"; most modern sets and online versions):
    5 each, white on 1 3 5 7 9, black on 2 4 6 8 10, white first;
    protection by an adjacent friend, blockade by three or more in a row
    (no passing), swap capture, backwards only if no forward move, 26 must
    be landed on, 27 sends the piece to 15 (or the highest empty square
    below), 28/29/30 leave only on exact 3/2/1, 26 bears off on 5; throw
    again after 1, 4, 5; a forfeit ends the turn. That project solved the
    game (8.56 billion positions; White, throwing first, wins 50.25% with
    perfect play) and has a 0.8 MB network that plays almost perfectly;
    MIT, so usable as the hard opponent for this rule set, credited
  - R. C. Bell 1960 ("The Game of Thirty Squares", Board and Table Games
    from Many Civilizations, 2nd ed. 1969, pp. 27-28): 10 men each placed
    alternately on the first two rows; a long die (1 to 4); forward only;
    landing on an enemy man captures and removes it; not onto your own;
    squares X, III, II, I (27-30) are safe and a man leaves from them on
    an exact throw; the door square (26) is safe but must be left for a
    marked square as soon as possible; a man must leave a marked square
    at the first possible throw; most men off wins
  - R. C. Bell 1979 (The Boardgame Book): a different game (sticks 1-4,
    none = 5; men enter on the marked squares; odd and even goals), known
    here only from the Game Cabinet's short summary; read the book before
    building it
- **Senet:** no rules survive. Jéquier's conjectural rules were replaced
  by R. C. Bell's and Timothy Kendall's reconstructions (Kendall's
  published 1978, not verified), which sellers of sets use. Source:
  en.wikipedia.org/wiki/Senet.
- **Mehen:** rules lost. Most reconstructions used the Sudanese Hyena
  game (recorded 1925); James F. R. Masters' 2024 papers argue Mehen was
  very unlikely played like Hyena. Player count uncertain (a Hesy-re tomb
  painting may show up to six).
  Masters' reconstruction (read 2026-10-20 at
  mastersofgames.com/rules/mehen-rules.htm and
  tradgames.org.uk/games/mehen-snake-game-ancient-egyptian-serpent.htm,
  linked by David): spiral snake, tail outside, head (hood) in the centre,
  70 squares recommended (boards had about 70 to several hundred); 2, 4
  or 6 players in two teams; each player six marbles, one the pharaoh
  racing the track, five for guessing; 3 lions for one team, 3 lionesses
  for the other. A turn: the team picks a pharaoh; the mover hides 1 to 5
  marbles in a fist, an opponent guesses the number; wrong: the pharaoh
  moves that many; right: the guesser moves their own pharaoh that many
  and may unleash a lion on the other's piece. An attacked pharaoh moves
  only after killing the lion: the lion's owner hides 1 to 3 marbles,
  right guess kills it (to the lion graveyard) and the pharaoh moves
  normally, wrong guess ends the turn. Two right guesses in a row by a
  team revive one of its dead lions. Landing on an opposing pharaoh swaps
  them (the other goes back to the mover's starting square); landing on a
  teammate moves one further. Reaching the hood: exact throw not needed;
  first team with all pharaohs home wins. Plus clarifications on lion
  placement (page). Evidence for marble guessing he calls "pretty strong"
  (marble sizes, tomb images of fists and open palms); the lions' role is
  his speculation. Papers: "Mehen, The Ancient Egyptian Serpent Game: A
  Reappraisal of the Evidence Set", Interdisciplinary Egyptology (Vienna)
  2024; "... A Reappraisal of Game-Play Theories", Birmingham Egyptology
  Journal 2024.
  Copyright: the rules page says "Copyright James Masters, 2025. All
  rights reserved." That covers his text, not the method of play: the US
  Copyright Office games circular (FL-108) and 17 U.S.C. 102(b) exclude a
  game's idea and method of play; only the wording is protected. So: our
  own implementation and our own wording of the rules, crediting Masters
  and his papers. EU wording not checked.
- **Hnefatafl / tafl:** reconstructed from Linnaeus' July 1732 Lapland
  diary (Tablut, 9x9, king and eight defenders against sixteen),
  Latin with some Swedish. E. L. Smith's 1811 English translation read
  "likewise the king" as "except the king", favouring the king's side;
  Murray's 1913 reconstruction built on it. Living sets: Copenhagen,
  Fetlar, Brandubh, Tablut retranslated. Neil Peterson argues Linnaeus'
  king capture is wrong (Welsh Tawlbrydd, 1587 or 1597). Sources:
  en.wikipedia.org/wiki/Tafl_games, aagenielsen.dk (rules of each
  variant: read there before building).
- **Fox & Geese:** medieval; variants in number of geese and movement
  (to be checked before building).

## Next for Ur

- [x] R. C. Bell's betting game, from his own text (Board and Table Games
      from Many Civilizations, 2nd ed. 1969, pp. 24-25, read on the Internet
      Archive, item B-001-002-771, not kept here): three lots, jewelled tips
      up 3 = 5 and again, 0 = 4 and again, 2 = 1 and again, 1 = nothing and
      the turn ends; enter only on 5; a rosette makes the OPPONENT pay a
      fine (Masters' summary says the mover pays; Bell's text wins); eight
      fighting squares, no safe ones; exact throw off; first home takes the
      pool. Bell is silent on where a piece entered on 5 stands (here: its
      first square) and whether an unusable throw still throws again
      (here: yes); the help says so. Games run about 220 throws against
      about 150 for Finkel 2017
- [ ] Murray's and Masters' paths: need a square-by-square diagram first
      (the Masters page describes them only in words)
- [x] The Babylonian tablet game: Finkel's Fig. 3.4 (rendered from the
      PDF) shows the later board, rosettes at houses 4, 8, 12, 16 and
      immunity from capture on them. Swallow enters on 2 on the FIRST
      rosette (his other reading, before any rosette, let it enter on 15
      and be home two throws later, found in a playtest); others on 5, 6,
      7, 10. Knucklebones: sheep 1 to 4, then dare the ox bone (yes:
      5, 6, 7, 10; no: wasted); the computer dares by expected value.
      Tokens (Finkel's Table 3.1 values) won on landing, paid for passing;
      purse 40 (20 ran dry within a few moves). About 80 throws a game
- [ ] Move animation, sound, preview, release workflow, hub entry

## Open questions (David's to decide)

- [x] Stack, name, first game, network play: see "Settled"
