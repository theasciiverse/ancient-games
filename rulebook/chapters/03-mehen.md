---
# The game's crate: build.sh writes this chapter as <crate>/RULES.txt,
# which the game shows as its help and ships in its release archive.
crate: mehen
---

# Mehen

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | Egypt, about 3000 to 2200 BC                          |
| Players        | Two, four or six, in two teams                        |
| Kind           | Race game, moved by guessing                          |
| Program        | `mehen`                                               |
| Rule sets      | Masters 2024, for two, four or six players            |

::: brief
A race along a coiled snake, moved by guessing. Each team has one, two
or three pharaohs; the side to move hides 1 to 5 marbles and the other
side guesses how many. Squares run from 1 at the tail to 70 at the head.
The rules in short follow; the full rules, their sources and the history
are in RULES.txt and the rulebook.
:::

## What survives

**The name.** Mehen is the name of a snake god of ancient Egyptian
religion, and the game is named after him [@wiki-mehen]. In the myth,
the sun god Ra sailed through the underworld every night, and Mehen
coiled his body round him to protect him [@masters-tradgames].

![An alabaster Mehen board of the Old Kingdom, the grooved kind, its
spaces raised bars and its head in the middle. Institute for the Study
of Ancient Cultures, Chicago. Photograph: Daderot,
CC0.](images/mehen-board-chicago.jpg){height=8cm}

**The boards.** A Mehen board is a snake coiled in a spiral, its head in
the middle and its tail at the edge, its body divided into spaces. The
number of spaces varies greatly from board to board [@masters-mehen]; the
board shown in a relief on the causeway of King Sahure's pyramid has
perhaps 600 [@masters-mehen-evidence, 9]. Masters takes it as accepted that there was
a single coil, that neither the direction of the coil nor the number of
spaces mattered, and that the pieces most probably travelled from the
tail to the head [@masters-mehen-play, 57]. Some boards have a track of
separate pits, others raised bars with grooves between them
[@masters-mehen-play, 56]. Boards shown on tomb walls usually have a
trapezium sticking out of the rim; a few real boards have an animal's
head there, or a goose or duck round the edge [@wiki-mehen].

James Masters counts eight whole or partial boards, only one of them
with a known findspot [@masters-mehen-evidence, 3]. That one, now in the
Ashmolean Museum in Oxford, is a small limestone board from Grave 19 of
the cemetery at Ballas. Masters dates it to about 3650 to 3300 BC, older
than any other known game board, though it may have been a model rather
than a board for play [@masters-mehen-evidence, 2, 22]. The largest
board, in the Fitzwilliam Museum in Cambridge, is 44 cm across
[@masters-mehen-evidence, 20]. A flat slate board in the Metropolitan
Museum of Art, which long confused attempts to reconstruct the game, is
modern: the museum told Masters in 2021 that it is not an ancient object
[@masters-mehen-evidence, 19].

![The pitted board in the British Museum, with an eye in the middle in
place of the head. The four marbles were added for the display.
Photograph: Waltros, CC0.](images/mehen-board-british-museum.jpg){width=70%}

**The pieces.** The game was played with marbles and with lions and
lionesses carved lying down, large compared with the spaces
[@masters-mehen-evidence, 2]. The sets Masters counts as almost certainly
for Mehen come from First Dynasty tombs: three lions and three lionesses,
some found with small marbles, white, or red and white
[@masters-mehen-evidence, 33]. The lionesses of a set in the Louvre wear
collars [@masters-tradgames].

![An ivory lioness from a First Dynasty Mehen set, with her collar.
Louvre E 16670. Photograph: Rama, CC BY-SA 3.0
FR.](images/mehen-lioness-louvre-e16670.jpg){width=55%}

**Pictures of play.** A wall of the Third Dynasty tomb of Hesy at
Saqqara carries a painting of the equipment for three games, Mehen among
them: the board, three lions, three lionesses and marbles
[@quibell1913; as cited in @masters-mehen-evidence, 6]. The colour copy
published in 1913 shows six sets of six marbles in six colours, but the
excavator could make out only two colours on the wall, and Masters
thinks three colours more likely than six [@masters-mehen-evidence,
6-8]. Reliefs in Fifth and Sixth Dynasty tombs show people at the board.
In the tomb of Kaemankh a player holds a closed fist up towards the
board [@masters-mehen-evidence, 5]. In the tomb of Rashepses four people
sit at the game [@masters-mehen-evidence, 10], and a lion and a lioness
in the middle of the board point at pieces [@masters-tradgames]. A block
from Sahure's causeway, found in 2019, shows three people on each side
of the board, the only picture with more than four players. Its captions
read "Playing Mehen", and the words in front of one player may be read
"Four!" [@masters-mehen-evidence, 8-9].

![The games painted in the tomb of Hesy, as copied for the 1913 report,
probably by Annie Quibell: the Mehen board on the left with its trapezium, and in
the box on the right the three lions, three lionesses and six sets of
marbles. Masters warns that the colours of the marbles are not to be
trusted. Public domain.](images/mehen-hesy-quibell.jpg){width=100%}

**The end of the game.** No boards or scenes are known from the Middle
or New Kingdom, so the game seems to have gone out of use in Egypt after
the Old Kingdom [@wiki-mehen]. Two tombs of the Twenty-sixth Dynasty
show it in scenes thought to be copied from a Fifth Dynasty tomb
[@masters-mehen-evidence, 5]. Boards are also known from Cyprus, where
the game lasted longer [@wiki-mehen].

**The meaning.** Peter Piccione argued that a piece travelling along the
board stands for the soul of a dead person passing through the body of
the god, to be reborn into the afterlife from the serpent's head
[@piccione1990; as cited in @masters-mehen-evidence, 37]. Masters finds
that the corrected evidence supports him [@masters-mehen-evidence, 37].

**What is not known.** The rules. No text says how Mehen was played. Nor
is it known how many played: the six people at Sahure could be two teams
of three playing as two sides [@masters-mehen-evidence, 9-10]. How the
pieces were moved, and what the lions did, are open questions.

In two papers of 2024 James Masters went through the evidence and the
earlier theories. He argues that the lions are too big to have moved
along the track, that one marble per player or team was raced to the
head, and that the moves were decided by guessing how many marbles a
player hid in a fist: the marbles are the right size for it, no dice are
shown with the Hesy set, and the reliefs show players holding up a fist
or hiding something in their hands. He thinks the lions were set in the
middle of the board and pointed at pieces on the track
[@masters-mehen-play, 72-79]. Their exact part in the game he calls
speculation [@masters-tradgames].

## The board

The program's board is a coil of 70 squares, the number Masters
recommends for play [@masters-mehen]. It is laid out on a grid of 14 by
8: the tail at the top left, the body coiling inwards clockwise with an
empty row or column between the coils, and the head near the middle.
The squares in the order the pharaohs travel:

```
  1  2  3  4  5  6  7  8  9 10 11 12 13 14
                                         15
 39 40 41 42 43 44 45 46 47 48 49 50     16
 38                                51    17
 37    63 64 65 66 67 68 69 70     52    18
 36    62 61 60 59 58 57 56 55 54 53     19
 35                                      20
 34 33 32 31 30 29 28 27 26 25 24 23 22 21
```

Square 1 is the tail and square 70 the head, the snake's hood. The
pharaohs wait at the start, off the board beside the tail, and come onto
it with their first move. A pharaoh that reaches the head leaves the
board: it is reborn into the afterlife.

Our choice: the old boards are round and their spaces wedge-shaped. The
program draws the coil square, on a grid, to fit a terminal.

## Masters 2024

::: brief
- Two teams, lions and lionesses, each with three cats. All pharaohs
  start off the board beside the tail; a coin picks who goes first.
- Your turn: choose a pharaoh still on its way. You hide 1 to 5
  marbles; the other team guesses.
- Wrong guess: your pharaoh moves as many squares as you hid.
- Right guess: they move one of their own free pharaohs that many
  instead, and may set a cat on your pharaoh. If all theirs are held,
  nobody moves, but the cat may still be set.
- A held pharaoh must first kill its cat: the cat's team hides 1 to 3,
  you guess. Right: the cat dies and you take a normal turn. Wrong:
  your turn ends.
- Two right guesses in a row (for a move, not in a cat fight) bring
  back one of the guessing team's dead cats.
- Ending on an opposing pharaoh: swap; it goes back to the square you
  came from (to the start if you were entering), its cat with it.
  Ending on a teammate: step on one square.
- Reaching or passing square 70 is rebirth; no exact count needed.
  The first team with all its pharaohs reborn wins.
:::

James Masters' rules, published on his company's website
[@masters-mehen]. He bases them on the findings of his 2024 papers and
fills the gaps with what he calls educated speculation: the marble
guessing is his reading of the evidence, the lions' part his own idea.
The program offers them for two, four or six players.

**The sides.** Two teams, the lions and the lionesses. Each player has
one pharaoh, a marble racing along the snake: with two players each team
has one pharaoh, with four two, with six three. Each team owns three big
cats: the lions' team three lions, the lionesses' team three lionesses.
In Masters' set each player also keeps five marbles for guessing; in the
program the hiding player just chooses a number.

1. All pharaohs start off the board, beside the tail. The teams take
   turns.
2. On its turn a team chooses which of its pharaohs moves: any one that
   has not yet reached the head.
3. If no cat holds that pharaoh, its player hides 1 to 5 marbles in a
   fist, and the other team guesses how many.
4. A wrong guess: the pharaoh moves forward as many squares as there
   were marbles.
5. A right guess: the guessing team moves one of its own pharaohs that
   many squares instead, and may also set one of its cats on the
   pharaoh that was to move. The cat holds it from then on.
6. A pharaoh held by a cat can move only after killing it. When the team
   chooses it, the cat's team hides 1 to 3 marbles and the pharaoh's
   team guesses. Right: the cat is killed and goes to the graveyard, and
   the turn goes on as a normal move (rules 3 to 5). Wrong: the cat
   keeps its hold, and the turn ends with no move.
7. A team that guesses right for a move two turns in a row brings one of
   its dead cats back to life, if it has one. Guesses in a fight with a
   cat do not count.
8. A pharaoh that ends its move on an opposing pharaoh changes places
   with it: the one landed on goes back to the square the mover started
   from. If the mover was coming onto the board, the one landed on goes
   back to the start. A cat holding it goes with it and keeps its hold.
9. A pharaoh that ends its move on a pharaoh of its own team moves on to
   the next square, and if that square is taken, these rules apply
   again there.
10. A pharaoh reaches the head without an exact count, and is reborn.
    The first team with all its pharaohs reborn wins.

![The program playing Masters' rules for two, the computer against
itself: the snake coils from the tail at the top left to the head, the
hiding hand is below it.](screens/mehen/two-players.svg){width=100%}

**Examples.** These are made by the program's own rules engine each
time the book is built: the pharaohs are set on the snake, one moves,
and the squares that matter are drawn before and after. Four players
here, so each team has two pharaohs. L is a pharaoh of the lions' team,
S one of the lionesses', . an empty square. (These letters are the
book's; on the game's screen an L marks a cat, and the pharaohs are
drawn in their team's colour.)

Ending a move on an opposing pharaoh changes places with it (rule 8):

```{.example file="mehen/swap"}
```

Even when the mover is coming onto the board:

```{.example file="mehen/entering"}
```

A teammate is passed by, one square further (rule 9):

```{.example file="mehen/teammate"}
```

The head needs no exact count (rule 10):

```{.example file="mehen/home"}
```

Two right guesses in a row bring a dead cat back (rule 7):

```{.example file="mehen/lion-back"}
```

![Six players: three pharaohs a team.](screens/mehen/six-players.svg){width=100%}

**Four and six players.** Masters' game for more than two is played by
two teams: the team decides which of its pharaohs moves, that pharaoh's
player hides the marbles and a player of the other team guesses; on a
right guess the guessing player moves their own pharaoh
[@masters-mehen]. In his paper he notes that a game for two teams can
work like a game for two players, the players of a team taking turns to
play their side, and that in some games they may move any piece of the
team [@masters-mehen-evidence, 10].

Our choices:

- The team to move first is chosen by a coin.
- One person, or the computer, plays a whole team. After a right guess
  the guessing team chooses which of its pharaohs moves; Masters gives
  the move to the player who guessed.
- A pharaoh held by a cat cannot be the one moved by a right guess
  (Masters: a held pharaoh moves only after killing the cat). If every
  pharaoh of the guessing team still on its way is held, the right
  guess moves nobody, though a cat may still be set.
- A pharaoh can be held by only one cat at a time. A cat may be set on a
  pharaoh still waiting at the start.
- "Two turns in a row" counts each team's guesses for a move only: a
  turn in which the team makes no such guess (because the other side
  failed to kill a cat) does not break the run. After two right guesses
  the count starts again, whether or not a cat came back.
- Square 70 is the hood: landing on it, or counting past it, is
  rebirth.
- When two people play at one keyboard, the game asks the guessing team
  to look away before each hiding, and waits for Enter.
- The computer always sets a cat when it can; on easy it forgets one
  time in three.

## Other reconstructions

Most rule sets published for Mehen in the last thirty years, in books,
on the internet and with commercial sets, are based on the Hyena game
of Sudan [@masters-mehen-play, 77]. R. C. Bell describes it as played by
the Baggara Arabs on a spiral of holes traced in the sand, a village at
the start and a well at the centre. Each player's marker is his mother,
moved by throwing three split sticks; she must reach the well exactly,
wash her clothes there and walk back, and the first mother home lets
loose a hyena that runs at double speed and eats the women it overtakes
[@bell1969, 12-14]. Bell notes that the spiral boards from early Egypt
"may have been used for similar games" [-@bell1969, 14]. Timothy Kendall
built a Mehen reconstruction on it, with the pieces going to the head and
back to the tail [@kendall2007; as cited in @masters-mehen-play, 69,
78].

Masters rejects the comparison: Hyena is moved with dice sticks, its
pieces go to the centre and back while the story of Mehen ends at the
head, nothing in Hyena matches Mehen's two kinds of cat, and racing all
thirty-six marbles of a Mehen set would take many hours [@masters-mehen-play, 78]. R. C. Bell
also proposed a guessing game in which a move is the difference between
the guess and the number hidden; Masters found in play-testing that it
gives moves too short to be satisfying [@masters-mehen-play, 77].

None of these is in the program.

## The ornament

The rim of the program's board carries the serpent's scales, after the
alabaster mehen board in the Institute for the Study of Ancient Cultures,
Chicago (E16950). Masters lists it with 127 spaces in four coils; it was
bought in Luxor in 1932 and has no provenance, though he argues for its
authenticity [@masters-mehen-evidence]. In a photograph of it, the
outermost band, outside the track, is not cut into spaces but
cross-hatched: two sets of fine diagonal lines crossing in small
lozenges, the scales of the serpent's tail [@commons-chicago-mehen]. The
program draws that lattice round the rim in braille dots, in a dim tone,
never over a piece, a highlight or a number. O turns it off and on, and
`mehen --plain` starts without it.

## Playing it in the terminal

::: brief
- 1 to 5 (1 to 3 in a cat fight): hide or guess that many; or Left,
  Right, Up, Down, Tab to choose and Enter or Space to confirm.
- Choosing a pharaoh: arrows or Tab, then Enter; or click it.
- Set a cat? Y or Enter: yes; N: no. The mouse clicks numbers and
  answers too.
- Two people at one keyboard: the guessing team looks away, then Enter.
- After a game: Enter plays again. M: menu; F1: this help; Q: quit.
- O: the ornament off and on (the scales of the serpent's tail, after
  the alabaster board in Chicago); `--plain` starts without it.
:::

Start `mehen`, choose two, four or six players and who plays each team:
the computer (easy, normal or hard) or a person at the same keyboard.

| Key | |
|:--|:--|
| 1 to 5 | Hide or guess that many marbles (1 to 3 in a fight) |
| Arrows or Tab | Choose a pharaoh, or a number |
| Enter or Space | Confirm |
| Y or N | Set a cat on the pharaoh, or not |
| Mouse | Click a pharaoh, a number or an answer |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| O | The ornament off and on |
| Q | Quit |

`mehen --demo` lets the computer play both teams; `--seed N` repeats a
game exactly; `--plain` leaves out the ornament.
