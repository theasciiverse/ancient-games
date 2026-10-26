---
crate: tabula
---

# Tabula

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | The Roman world: duodecim scripta from the 1st century BC, tabula by the 5th century AD |
| Players        | Two                                                   |
| Kind           | Race game with three dice: fifteen men round the board and off |
| Program        | `tabula`                                              |
| Rule sets      | Tabula with the colours running opposite ways (Becq de Fouquières, Austin); the same way (Bell); ludus duodecim scriptorum (Murray) |

::: brief
The Roman ancestor of backgammon. Each side has fifteen men, all in hand
at the start, and three dice. Each die moves one man that many points
along its way round the board: into the board, round it, and off the
far end. A point with two or more men of one side is closed to the
other; a single man (a blot) is hit when an enemy man lands on it and
must start again. You must use the whole throw if you can. A man may
leave the board only when all your men are in the last six points. The
first to bear off all fifteen wins. The rules in short follow; the full
rules, the sources and the history are in RULES.txt and the rulebook.
:::

## What survives

No ancient text gives the rules of either Roman game. What survives is
boards by the hundred, dice and dice towers, a few words used by the
players, and one game position, written in verse. Everything else is
reconstruction, mostly by analogy with backgammon, and its makers say
so: R. G. Austin's account "is based entirely on legitimate analogy
with the rules common to the backgammon family of games, and on
inference from the type of board used" [@austin1934rbg1, 34].

**Duodecim scripta**, "twelve lines" or "twelve marks", is named by
Cicero, and Ovid describes "another sort [of game] divided exactly into
as many markings (*scriptula*) as the rolling year has months"
[@austin1934rbg1, 31]. Its boards have three rows of twelve, each row
split in the middle; many are cut in stone, and many spell out six
words of six letters, two to a row. One from Timgad reads VENARI
LAVARI / LVDERE RIDERE / OCCEST VIVERE: "to hunt, to bathe, to play,
to laugh: that is living" [@austin1934rbg1, 31].
A board found at
Qustul in Nubia, of about the 4th century, lay with fifteen men of
ivory and fifteen of ebony, a dice tower and five dice
[@bell1969, 28-29].

![Marble board for the game of twelve lines, found near the agora of
Aphrodisias, Turkey. Photograph: wneuheisel, CC BY
2.0.](images/tabula-xii-scripta-aphrodisias.jpg){width=70%}

**Tabula** is the later game on two rows of twelve: the twenty-four
points of backgammon. When the middle row was given up is not known
[@austin1935rbg2, 78]; boards of two rows are known from Sardis,
Didyma, Ostia and Rome [@durnagolu2025, 30]. Isidore of Seville, early in the 7th
century, writes that "tabula is played with a dice tower, men and dice",
three dice, and that some men move "in order", some "at random", and
some cannot move at all [@isidore-etym, XVIII.60-67]. The game passed
to Byzantium as *tavli* and to medieval Europe as the game of tables,
whose rules Alfonso X had written down in 1283 [@golladay2007, 413-495].

![The Vettweiß-Froitzheim dice tower, 4th century, with Roman counters;
its inscription reads PICTOS VICTOS HOSTIS DELETA LVDITE SECVRI.
LVR-LandesMuseum Bonn. Photograph: Carole Raddato, CC BY-SA
2.0.](images/tabula-dice-tower-bonn.jpg){height=8cm}

**Players.** Suetonius says the emperor Claudius "played dice most
eagerly", wrote a book on the art, and had a board fitted to his
carriage so that he could play while driven [@suetonius-claudius, 33.2];
the word is *alea*, and which game he played is not said. Martial
describes a board with duodecim scripta on one side and a war game on
the other [@martial-14, XIV.17]. A painted wall of a tavern at Pompeii
shows two men at a board, one crying "Exsi" (I'm out), the other "Not
three, it's two", and the landlord telling them to fight outside
[@bell1969, 33].

![Two dice players quarrel: "Exsi" (I'm out), "Non tria, duas est"
(not three, it's two). Wall painting from the Caupona of Salvius,
Pompeii, before AD 79. Museo Archeologico Nazionale di Napoli, 111482.
Photograph: ArchaiOptix, CC BY-SA
4.0.](images/tabula-pompeii-dice-players.jpg){width=50%}

**Zeno's game.** Agathias, a poet of the 6th century, wrote an epigram
on a game the emperor Zeno (who reigned 474-491) played and lost. It
names the points each man stood on and the throw, and ends: "Avoid
backgammon, as the king himself did not escape from its blind chance"
[@paton1917, 269]. Louis Becq de Fouquières worked the position out in
1869 [@becq1869, 371-380], and Austin in 1934 [@austin1934zeno]. It is
the one place where the rules of a Roman board game can be tested, and
it holds: see the example below. The position only works out if a
point of two men is closed, if each die moves a man a step of its own
(every point it stops on open), if the whole throw must be used, and
if no man may leave the board before all are in the last quarter
[@austin1934zeno, 203-204]. Bell calls that last rule "not mentioned
by classical writers" and adds it because it improves the game
[@bell1969, 35]; Austin had already shown that Zeno's position needs
it. The program, playing Becq's and Austin's rules, finds that the
throw 2, 5, 6 leaves Zeno exactly one play, and that it leaves the
eight single men the poem counts:

```{.example file="tabula/zeno"}
```

What the epigram does not decide is which way Black goes: it counts
Black's men on the same numbers as White's, and either reading fits.

**Corrections.** Bell's chapter [@bell1969, 30-35] is the most read
account; some of it does not hold. The throws he gives Cicero, *Venus*
and *Canis*, are throws of four knucklebones, not of three dice; Zeno
reigned 474-491, not 475-481; Pompeii was not buried "with lava in
A.D. 63"; and Suetonius' *alea* does not say that Claudius played
tabula.

## The boards

![The board of tabula, its points numbered along White's way.
](figures/tabula-board.svg){width=80%}

The program numbers the points along White's way, 1 to 24, and names
them so in its messages. Black's points are named the same, whichever
way Black goes.

![The board of duodecim scripta with Murray's route.
](figures/duodecim-board.svg){width=80%}

## Rules every set shares

::: brief
- 15 men each, all in hand at the start; three dice; White throws first
  (our choice).
- Entering: a die of n enters a man on your nth point. Men may enter and
  men on the board may move in the same turn.
- Each die moves one man its number of points; one man may take two or
  three dice in turn, but every point it stops on must be open.
- Two or more men of one side close a point. A single man is hit by an
  enemy landing on it and goes back to hand; while you have a hit man,
  you must enter it before anything else moves.
- Use as many dice as you can; if not all, the highest you can (our
  choice, after Alfonso X). Each die counts once, doubles and triples
  too (our choice).
- Bearing off: only when all your men are in your last six points; a
  man leaves on an exact throw, or a higher one if no man of yours
  stands further back (our choice).
:::

What the sources give, and what they do not:

- **Fifteen men and three dice**: the counts in Zeno's position, the
  Qustul find and Isidore's "three dice" [@becq1869, 373;
  @bell1969, 28-29; @isidore-etym, XVIII.64].
- **Each die a separate step, onto open points; two men close a point;
  the whole throw used; bearing off only from the last quarter**: all
  needed for Zeno's position [@austin1934zeno, 203-204]; Bell gives the
  same as rules [@bell1969, 34-35].
- **Hitting** a single man, and its entering again before its owner
  moves anything else: Austin's and Bell's inference, by analogy
  [@austin1934rbg1, 34; @bell1969, 35].

**Our choices**, where every source is silent:

- **Who begins**: White.
- **Entering**: a die of *n* puts a man on its owner's *n*th point.
  Austin says the men are entered "from his own entry table", not how
  [@austin1935rbg2, 78]. Becq instead stacks all fifteen on the first
  point at the start [@becq1869, 377]; the program enters them.
- **A hit man that cannot enter**: those dice are lost.
- **When not every die can be played**: as many as can be, and of
  those plays the highest numbers, as Alfonso's three-dice game *buffa
  de baldrac* has it: "they use the largest number they can"
  [@golladay2007, 494].
- **Doubles and triples**: each die once; nothing ancient says they
  count twice.
- **Bearing off**: an exact throw, or a higher one for the man furthest
  back; a man hit while bearing off must come round again before
  bearing off goes on.

```{.example file="tabula/entering"}
```

A hit, and what it costs:

```{.example file="tabula/hit"}
```

## Tabula, the colours opposite

::: brief
- Contrary motion, as in backgammon: White enters on points 1 to 6 and
  bears off from 19 to 24; Black enters on 24 to 19 and bears off from
  6 to 1.
:::

Becq: White "made his way from the white men's quarter towards the
rear road, while his opponent made his way from the black men's
quarter towards the front road" [@becq1869, 374; our translation]; Austin: each player
moves "from his own entry table ... through his opponent's tables, so
as to reach the latter's entry table" [@austin1935rbg2, 78], arrows
and all in his drawing of Zeno's board [@austin1934zeno, 203]. The
medieval *buffa de baldrac*, with three dice, men entered from the
hand and the two sides "the one against the other through all the
tables of the board", is the same game rule for rule [@golladay2007,
494].

![The program playing tabula, the colours opposite, the computer against
itself.](screens/tabula/tabula.svg){width=100%}

## Tabula, the colours the same way

::: brief
- Parallel motion: both sides enter on points 1 to 6, run the same way
  round the board and bear off from 19 to 24. Every point is contested
  from the start.
:::

Bell: "The pieces were entered on the board in the first quarter and
travelled anti-clockwise round the track. Apparently both colours
travelled in the same direction" [@bell1969, 35]. Ulrich Schädler reads
the epigram the same way [@schadler2020]. Bell also suggests, as his
own improvement, that no man pass the first half until all are entered
[@bell1969, 35]; the program does not, and Schädler's further rules
(a middle row for hit men, doubles played twice) are his own, by his
own account oriented on the medieval game of Puff [@schadler2020].

## Ludus duodecim scriptorum

::: brief
- Three rows of twelve, both sides the same way: in along the middle
  row (points 1 to 12), back along the top (13 to 24), along the bottom
  (25 to 36) and off.
- No man may go beyond point 12 while any of yours is still in hand.
- Bearing off from the last six points, 31 to 36. Otherwise as tabula.
:::

H. J. R. Murray's reconstruction, which Austin printed
[@austin1934rbg1, 33-34; @murray1952 as cited in @austin1934rbg1]. It
rests on one board, from Ostia, lettered row by row in a way that
"strongly suggests that the direction of play followed the letters of
the alphabet" [@austin1934rbg1, 33]: the men enter on the middle row,
then go "through B to C, then down to D" and off. "Neither player could
begin to advance his pieces until all were so entered on the middle
row"; blots could be hit while entering too, "which would thus often
occupy a considerable time" [@austin1934rbg1, 34].

```{.example file="tabula/duodecim"}
```

**Our choices** beyond tabula's: three dice, as Austin has it
[@austin1934rbg1, 34]; Schädler argues for two, reading *scripta* as
the pips of the dice [@schadler2020]. Ovid's "modo tres iactet numeros"
(let her throw three numbers now) in the passage on women's games
[@ovid-ars, III.355] fits three.

![The program playing duodecim scripta, the computer against itself.
](screens/tabula/duodecim.svg){width=100%}

## The ornament

Many Roman boards of three rows of twelve mark the middle of each row
with a sign; Ihm, who collected them in 1890, took these signs for
decoration only: circles, half circles, stars, branches, ivy leaves,
three as a rule [@ihm1890]. The program's ornament follows one board: a
marble base at Aphrodisias given "under Flavius Photius, scholasticus and
pater", late fifth or sixth century, with "three parallel rows of twelve
small circles, with the central point in each row marked by a rosette,
described with calipers" [@insaph-ala069]. A photograph of it shows the
marks: a six-petalled compass rosette in a circle in the middle row, and
a half rosette in a half circle in each outer row
[@commons-aphrodisias-board].

On a terminal larger than 100 by 30 the margins beside the board carry
those three marks, half, whole and half, drawn with the compass as on the
stone, in braille dots and a dim tone. The rims of real boards are plain
mouldings, so the program's board has none. O turns the ornament off and
on, and `tabula --plain` starts without it.

## Playing it in the terminal

::: brief
- Enter or Space: throw. Then arrows (or the mouse) to a lit man, Enter
  to pick it up, a lit point (or "Borne off") to put it down; one die
  at a time. Esc: let go, or take back the last step.
- In hand and Borne off stand above and below the board.
- When the game is over, Enter plays again.
- M: menu; F1: this help; Q: quit.
- O: the ornament off and on (the rosettes of a marble board at
  Aphrodisias); `--plain` starts without it.
:::

Start `tabula`, choose the rules, whom to play against (the computer,
easy, normal or hard, or a friend at the same keyboard) and which side
you play.

| Key | |
|:--|:--|
| Enter or Space | Throw the dice; pick up a man; put it on a lit point |
| Arrows | Move the cursor between points, the hand and the men borne off |
| Esc | Let go of the man, or take back the last step of the turn |
| Mouse | Click to throw, to pick up and to put down |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| O | The ornament off and on |
| Q | Quit |

The computer: easy hits when it can and otherwise moves its furthest
men, and now and then plays any move at all; normal judges each play by
the race, the points it holds, the blots it leaves and how many enemy
men could reach them; hard also weighs every throw of the other side
and its best answer.

`tabula --demo` lets the computer play both sides; `--seed N` repeats a
game exactly; `--plain` leaves out the ornament.
