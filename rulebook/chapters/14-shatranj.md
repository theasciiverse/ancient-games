---
crate: shatranj
---

# Shatranj

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | Persia from about 600, then the whole Islamic world and medieval Europe; its Indian parent, chaturanga, from the 7th century at the latest |
| Players        | Two                                                   |
| Kind           | War game: mate the shah, bare him, or shut him in     |
| Program        | `shatranj`                                            |
| Rule sets      | Shatranj after as-Suli; the Medinese victory; chaturanga with al-'Adli's Indian rules, and two other Indian elephants |

::: brief
Chess before the queen and the bishop. Eight by eight squares; each side
has a shah (king), a firzan (counsellor), two fils (elephants), two
faras (horses), two rukhs (chariots) and eight baidaqs (foot soldiers).
The shah, the faras and the rukh move as the king, the knight and the
rook do today; the firzan steps one square diagonally; the fil leaps
exactly two squares diagonally, over anything; the baidaq steps one
square forward and takes one diagonally forward, and on the far row
becomes a firzan. No castling, no double step. You win by mate, by
taking every man of his but the shah (baring him), or, in shatranj, by
leaving him without a move. The rules in short follow; the full rules,
the sources and the history are in RULES.txt and the rulebook.
:::

## What survives

Shatranj is the best documented game in this collection. Its rules were
written down by the masters who played it in Baghdad in the 9th and
10th centuries, and their books survive in later manuscripts, with
hundreds of problems and their solutions. H. J. R. Murray gathered them
in his *History of Chess* [@murray1913], and this chapter follows him
throughout. What is not known is how the game began.

**India.** The game's name comes from the Sanskrit *chaturanga*, "of
four members": the four arms of an Indian army, chariots, horse,
elephants and foot, a word for the army long before it was a game
[@murray1913, 42-44]. The first sure mention of the game is in Bana's
life of the king Harsha, who ruled from 606 to 648: under him "only
ashtapadas teach the positions of the chaturanga", the joke being that
the only armies drawn up in his peaceful realm were the ones on the
board [@murray1913, 52]. Murray found no other Indian reference before
the year 1000 [@murray1913, 56], and quotes Fiske on all that came
before the 7th century: "it is all impenetrable darkness"
[@murray1913, 48]. Dates for the invention, a Gupta origin, a fifth
century, are guesses; no source read for this book gives one.

**Persia.** The Persian *Karnamak* of Artakhshir, probably written
about 600, says its hero became "more skilled than them all in
ball-play, in horsemanship, in chess (chatrang), in hunting", the
earliest reference to chess in any literature [@murray1913, 149]. A
Persian text of the first centuries of Islam, the *Chatrang-namag*,
tells how a king of India sent the game to Khusraw I (531-579) to test
the wisdom of the Persians, and how the sage Wazurgmihr worked it out
and answered with a game of his own, nard [@murray1913, 150-153];
Firdausi retells the story in the *Shahnama* (1010), where the board
comes with men "half ... of ivory, and bright, the other half of teak"
[@warner1915, 385]. Murray takes the embassies for invention, but the
tradition behind them, that chess came to Persia from India in the
6th century, for history [@murray1913, 153-154].

**Islam.** The Arabs learned the game when they conquered Persia and
called it *shatranj*. Al-'Adli wrote on it about 840; as-Suli, who rose
at the court of the caliph al-Muktafi (902-908) and died at Basra in
946, was its greatest master, and the caliph ar-Radi said "As-Suli's
skill at chess charms me more than these flowers" [@murray1913,
199-200]. Their books, and those of their pupils, give the moves, the
endings and their verdicts, the openings, and problems (*mansubat*):
over 1,600 in the manuscripts, about 553 of them different
[@murray1913, 220-339, 270]. Muslim chessmen were abstract shapes, not
figures [@murray1913, 223-224].

![Chess set of glazed stonepaste, Nishapur, Iran, 12th century: the
abstract men of Islamic chess. The Metropolitan Museum of Art, New
York, 1971.193a-ff, CC0.](images/shatranj-nishapur-chess-set-met.jpg){width=60%} From Islam the game went on to Europe,
where in time the firzan and the fil became the queen and the bishop.

**Chaturanga.** Of the rules of the Indian game before the year 1000,
only what al-'Adli wrote of it survives: the Persians, he says,
"altered some of the rules", and he names the Indian ones (below)
[@murray1913, 56-57]. Al-Biruni, who was in India about 1030, describes
a four-handed game with dice and a different elephant [@sachau1888,
183-185]. Bell's chapter on "shaturanga" [@bell1969, 51-56] follows
Forbes [@forbes1860] in deriving chess from that four-handed game and
in placing its text in the *Bhavishya Purana*; Murray found the passage
in no manuscript of that Purana, and traces it only to a work of about
1500 [@murray1913, 44-49]. This book follows Murray.

## The men and the board

The board is the chessboard of eight by eight, but plain: the old
boards were not chequered [@murray1913, 220], and the program draws
them as a cloth ruled into squares. The manuscripts colour the sides
red and black, "probably because inks of these colours were most
easily procurable" [@murray1913, 224], and so does the program. The
squares are named as in chess today, a1 at Red's left.

```{.example file="shatranj/start"}
```

The manuscripts draw the shahs on the d file, facing each other, the
firzans beside them [@murray1913, 224, 247]. On a plain board that is
the same game as the shahs on e1 and e8, seen in a mirror; the program
sets them on the e file, as Murray writes his moves [@murray1913, 247].

| Man | Moves |
|:--|:--|
| Shah (K) | One square any way. He may not move into check, and must be got out of it [@murray1913, 225] |
| Firzan (F) | One square diagonally; it can reach only half the board [@murray1913, 225] |
| Fil (E) | A leap of exactly two squares diagonally, "over a diagonally adjacent square, whether occupied or empty"; each fil reaches only eight squares [@murray1913, 225-226] |
| Faras (N) | The knight's leap [@murray1913, 226] |
| Rukh (R) | Any distance along a rank or file [@murray1913, 226] |
| Baidaq (P) | One square forward, never two; takes one square diagonally forward; on the far rank it becomes a firzan, and only a firzan [@murray1913, 226] |

The letters are the program's: K, N and R as in chess today, F for the
firzan and E for the elephant. With so weak a firzan and fil, games are
slow to come to grips, and many are decided in the ending.

## Shatranj

::: brief
- Red moves first (our choice; the manuscripts give Red the move).
- Mate wins. A stalemated shah (no legal move, not in check) loses.
- Baring wins: take every man of his but the shah while keeping one of
  yours. But if he can take your last man on his very next move, he
  may, and both bare is a draw.
- A third repetition of a position is a draw; so are 70 moves each
  with nothing taken and no baidaq moved (our choice).
:::

The game of al-'Adli and as-Suli, as Murray gives it [@murray1913,
220-339].

- **Mate** (*shah mat*) wins [@murray1913, 225].
- **Stalemate** loses for the side stalemated. "The player who found
  himself in this predicament was held to have lost the game"; Murray
  has it from the solutions of the problems [@murray1913, 229 and n.
  12]. Bell, after Forbes, gives a rule by which a stalemated shah
  changes places with one of his men and plays on [@bell1969, 60];
  Murray shows that Forbes took it from Timur's chess, a later and much
  changed game [@murray1913, 229 n. 12]. The program follows Murray.
- **A bare shah** loses: "obtained whenever a player captured the whole
  of his opponent's army, the King excepted, and still retained some of
  his own men upon the board ... If the solitary King could take his
  opponent's last remaining man in his move following that in which he
  was bared, the game was considered drawn" [@murray1913, 228-229].
- **Draws**: by equal or too little force, by perpetual check, or by
  repeating the moves [@murray1913, 229, 268].

**Our choices.** The sources say neither who moves first nor how many
repetitions make a draw, and give no count of moves for a game that
goes nowhere.

- Red moves first.
- A position reached for the third time, the same side to move, is a
  draw; this covers perpetual check.
- After 70 moves each with nothing taken and no baidaq moved, the game
  is a draw. The endings of equal force, rukh against rukh or firzans
  that can never meet, would otherwise never end.
- When a shah is bared and could bare back, the game goes on one move,
  and his side decides; the computer always takes the chance.
- A move that bares and mates at once wins as mate.

**Dilaram's problem.** The most famous of the problems, as-Suli's,
copied into Alfonso X's *Book of Games* (1283) and the source of nearly 200 problems in
medieval Europe [@murray1913, 311-312]. A player, the story goes, had
staked his favourite, Dilaram, on the game, and when this position came
she cried out to him "Sacrifice your two Rooks, and not me"
[@murray1913, 311]. The program checks every move of the solution.

```{.example file="shatranj/dilaram"}
```

**The bare shah.** Murray's own illustration of the rule
[@murray1913, 229 n. 11], under both rules:

```{.example file="shatranj/bare"}
```

![The program playing shatranj, the computer against itself.
](screens/shatranj/shatranj.svg){width=100%}

![Men playing chess, with a harpist and women serving, *Libro de los
juegos* of Alfonso X, 1283, fol. 22r. Real Biblioteca del Monasterio de
San Lorenzo de El Escorial, MS T-I-6. Public
domain.](images/shatranj-libro-de-los-juegos-22r.jpg){width=90%}

## Shatranj, the Medinese victory

::: brief
- As shatranj, but whoever first bares the other's shah wins at once,
  even if the bared shah could take the last man back.
:::

Al-'Adli: "Of the Indian rules of chess, one is observed by the people
of Hijaz, and is called by them the Medinese Victory. If there be with
the Kings two pieces, and the King can take a piece, then which ever
first takes, so that the other is left with nothing, wins"
[@murray1913, 57]. "There a solitary King was defeated the instant that
he was bared, whether he could bare his opponent the following move or
not" [@murray1913, 229]. Everything else is as in shatranj.

## Chaturanga, the Indian rules

::: brief
- The men move as in shatranj.
- A stalemated shah wins: the side left without a move, not in check,
  has won (al-'Adli's Indian rule).
- Whoever first bares the other's shah wins (the Medinese rule, which
  al-'Adli calls Indian).
- Draws as in shatranj (our choice).
:::

The two-handed Indian game as far as it can be had: shatranj with the
rules al-'Adli calls Indian [@murray1913, 56-57, 60-61].

- **Stalemate wins** for the stalemated: "when the King cannot find a
  square into which to move, and the other King has nothing wherewith
  to checkmate him, the first has won. But this is not a Persian rule"
  [@murray1913, 57].
- **Baring** wins for whoever bares first, as in Medina
  [@murray1913, 57].
- **The elephant.** Murray found three moves recorded for it in India;
  the diagonal leap of two squares "became the widest spread, and it is
  probable that it is the oldest move. It is the only one which passed
  westwards" [@murray1913, 59-60]. The elephant of the ivory set once
  called Charlemagne's, made in southern Italy in the late 11th
  century, is still an elephant (pictured). This rule set uses it; the two
  others are the next two rule sets.

![Knight and elephant (the fil) of the so-called Charlemagne chessmen,
southern Italy, late 11th century, ivory. Cabinet des médailles, BnF,
Paris. Photograph: Poulpy, CC BY-SA
3.0.](images/shatranj-charlemagne-knight-elephant.jpg){width=55%}

Everything else (the setup, the baidaq's promotion, who moves first,
the draws) is shatranj, our choice: no Indian text gives it.

A stalemate under both rules:

```{.example file="shatranj/stalemate"}
```

## The elephant in the corner

::: brief
- Chaturanga, the Indian rules, but the elephant stands in the corner
  and leaps exactly two squares along a rank or file, over anything.
  The rukh (chariot) stands on c1 and f1.
:::

Al-'Adli: "Another Indian rule is that the Elephant is placed in the
corner, and omits one square in a straight line to jump into the second
in a straight line. And this it does in all the squares of the board.
Each Elephant has 16 squares" [@murray1913, 57]. With the elephant in
the corner, the piece with the rook's move goes where the shatranj fil
stands [@murray1913, 60].

```{.example file="shatranj/start-corner"}
```

## Al-Biruni's elephant

::: brief
- Chaturanga, the Indian rules, but the elephant steps one square
  diagonally or one straight forward: "the trunk and the four feet".
:::

Al-Biruni, about 1030: "they move the elephant straight on, not to the
other sides, one square at a time, like the pawn, and to the four
corners also one square at a time, like the queen (firzan). They say
that these five squares ... are the places occupied by the trunk and
the four feet of the elephant" [@sachau1888, 183].

## Not offered: chaturaji

Al-Biruni also describes a game for four, each with an army in a
corner of the board, the man to move chosen by the throw of dice
[@sachau1888, 183-185]; Raghunandana's *Tithitattva*, written in
Bengal about 1500, gives more of it
[@murray1913, 68-73]. So much of it is unknown (how the turn passes,
how the two dice are used, how pawns promote, how the game ends) that
it is not offered here.

## Playing it in the terminal

::: brief
- Arrows: move the cursor; Enter or Space: pick up a man, then put it
  on a lit square; Esc: let go. Or click.
- The panel shows the last moves: letter, from, to, x for a capture,
  =F for a baidaq made firzan, + for check.
- When the game is over, Enter plays again.
- M: menu; F1: this help; Q: quit.
:::

Start `shatranj`, choose the rules, whom to play against (the computer,
easy, normal or hard, or a friend at the same keyboard) and which side
you play.

| Key | |
|:--|:--|
| Arrows | Move the cursor |
| Enter or Space | Pick up a man; put it on a lit square |
| Esc | Let go of the man |
| Mouse | Click a man, then a square |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| Q | Quit |

The computer searches ahead, captures to the end, and judges a position
by its men (the rukh worth most, then the faras; the firzan and the
elephant little), by the baidaqs' advance and by where the men stand;
with few men left it hunts the other shah and his last men, since a
bare shah loses. Easy looks one move ahead and now and then plays any
move, normal three, hard as deep as it can in a second and a half.

`shatranj --demo` lets the computer play both sides; `--seed N` repeats
a game exactly.
