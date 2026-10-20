# Ancient games

Terminal versions of the classic board games of the world, starting with
the oldest: the Royal Game of Ur, Senet, Mehen, Hnefatafl and Fox &
Geese (David, 2026-10-20). More to follow.

**Status (2026-10-20):** the Royal Game of Ur is playable (`ur/`, binary
`royal-ur`) with four rule sets: Finkel 2017, British Museum 1990s,
R. C. Bell's betting game and the Babylonian tablet game; computer at
three strengths or two players, keys and mouse, help. Git repo here, branch
`main`, not on GitHub yet.

## Code

Cargo workspace; `ur/` is the first game. No shared core yet: it is
extracted when the second game needs it (validate before generalizing).

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
- Options: `--seed N`, `--demo` (hard against normal, restarts),
  `--quit-after SECONDS`; `RUST_LOG=debug royal-ur 2>ur.log`

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
