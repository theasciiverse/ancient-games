---
crate: mu-torere
---

# Mū tōrere

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | The East Coast of the North Island of New Zealand, by Ngāti Porou; recorded by 1912 |
| Players        | Two                                                   |
| Kind           | Blocking game: no captures                            |
| Program        | `mu-torere`                                           |
| Rule sets      | The pūtahi beside an enemy; Mū tōrere after Best      |

::: brief
Two players have four men each on an eight-pointed star: eight points
(kewai) round a centre (pūtahi), each kewai joined to its two neighbours
and to the pūtahi. The men start on four neighbouring kewai each, the
pūtahi empty. In turn, each moves one man to the one empty point, if it
is joined to it. Nothing is taken: who cannot move has lost. The rules
of each form in short follow; the full rules, their sources and the
history are in RULES.txt and the rulebook.
:::

## What survives

**The game.** Mū tōrere, or tōrere, is a game of the Māori of the East
Coast of the North Island. Elsdon Best describes it in *Games and
Pastimes of the Maori* (1925), as it was explained to him by Tuta
Nihoniho of Ngāti Porou: "A board is marked with charcoal with a design
resembling an eight-pointed star", its centre "termed the *putahi*, the
radiating arms are termed *kawai* or tentacles, the design being
compared by the Maori to an octopus". Each player has four *perepere*,
"small stones so marked that each player's men may be readily
distinguished", and "there is no taking or crowning of men, it is simply
a question of blocking your opponent" [@best1925, 111]. Today the arms
are usually written *kewai*; this book follows that.

![Te Rangiua and his wife Te Urikore, of Tūhoe, playing mū tōrere on a
board marked on a plank, about 1910. Dominion Museum photograph,
published by Best in 1925 (his fig. 26). Alexander Turnbull Library,
Wellington, PAColl-5671-07. Public
domain.](images/mu-torere-players-1910s.jpg){height=8cm}

**The boards.** "The board used was sometimes a piece of hewn plank,
sometimes a piece of the inner bark of the totara tree, the inner side
of which was marked with the design while green", and kept flat with
sticks while it dried. "In some cases diagrams of a temporary nature
were marked on the earth with a pointed stick" [@best1925, 112].

**How old.** Nobody knows. Best was told by Mohi Turei of Ngāti Porou,
in 1912, that *mū tōrere* was the old name of the game, and old Māori
told him that so far as they knew it was a Māori game. But Tuta
Nihoniho said that draughts reached the district in his grandfather's
time; *mū* is also the Māori name for draughts, and may come from the
English "move"; and the game was known, Best found, only on the East
Coast. "Some natives maintained that it was a pre-European usage, others
denied this and attributed its introduction to early whalers." Best
himself was "not inclined to view *mu torere* as an old time Maori game"
[@best1925, 110-113]. Te Ara, the encyclopedia of New Zealand, gives
both views, and adds that tōrere was a popular game on the East Coast
"from the very early years of European contact" [@teara-boardgames].

One of Best's informants said the game was also played with one man a
side on a board of six points [@best1925, 113]. The program does not
offer that.

![The board as Best draws it, the points numbered (Best, *The Maori*,
1924, reprinted as fig. 25 of *Games and Pastimes of the Maori*). Scan:
New Zealand Electronic Text Collection, CC BY-SA 3.0
NZ.](images/mu-torere-board-best-1924.jpg){width=38%}

**What is not known.** Best's account is the one record of the rules
from a player of the game. It leaves open who begins, and how far one
rule reaches: the rule that keeps a man out of the pūtahi. The program
offers both readings of that rule (see below).

## The board

![The board, with the points numbered as in this book and the men as
they start: White on 1 to 4, Black on 5 to 8, the pūtahi empty. The
solid star joins each kewai to its neighbours; the dashed spokes join
each to the pūtahi.](figures/mu-torere-board.svg){width=45%}

Eight kewai on a ring, numbered 1 to 8 round it, and the pūtahi in the
middle. Each kewai is joined to the kewai on either side of it and to
the pūtahi; the pūtahi is joined to all eight. With eight men on nine
points, one point is always empty, and every move fills it.

In the program the star is drawn as a ring of dots with the spokes
across it; in the text of this book it is drawn as a square, which has
the same joins.

## The pūtahi beside an enemy

::: brief
- 4 men each: White on kewai 1 to 4, Black on 5 to 8. White begins.
- A move: a man goes to the empty point, along the ring to the next
  kewai, from a kewai into the pūtahi, or out of the pūtahi to any kewai.
- A man may enter the pūtahi only if one of the kewai beside it holds an
  enemy man, at any time in the game.
- Who cannot move has lost. The same position a third time: a draw.
:::

The rules as most accounts give them today:

1. Each player has four men, on four neighbouring kewai.
2. In turn, each moves one of his men to the empty point, if it is
   joined to the man's point.
3. A man may move from a kewai into the pūtahi only if at least one of
   the two kewai next to it holds a man of the other side.
4. A player who cannot move has lost.

Rule 3 is not in Best's words (see the next rule set). We have not found
who first gave it in this form; Wikipedia gives it without a source
[@wiki-mu-torere], and Cameron Browne reports that Marcia Ascher, in her
analysis of the game [@ascher1987, as cited in @browne2021], found that
"at least two historians' transcriptions of its rules simplified out an
apparently trivial starting rule, without which the game became
unplayable" [@browne2021, 2]. Without any such rule the first player
wins on his first move: a man whose neighbours are both his own steps
into the pūtahi, and the other side is blocked.

With this rule the game is a draw if both play well. The program's tests
work out every position (1,180 can be reached from the start) from the
ends back, and the start is drawn; Wikipedia gives the same count and the
same result [@wiki-mu-torere].

Our choices:

- White begins. Best's sample game is opened by the player on 5 to 8;
  he gives no rule.
- The same position with the same side to move, a third time, is a
  draw. No source says how a game that goes round in circles ends.

**Examples.** These are made by the program's own rules engine each
time the book is built. W is a white man, B a black one, . the empty
point; the square on the right numbers the points, P the pūtahi.

At the start only the men beside an enemy may enter the pūtahi:

```{.example file="mu-torere/start"}
```

A side that cannot move has lost:

```{.example file="mu-torere/blocked"}
```

![The program playing, the computer against itself.
](screens/mu-torere/beside-enemy.svg){width=100%}

## Mū tōrere after Best

::: brief
- As above, but the rule about the pūtahi holds only for the opening
  move: the first man to move must come from beside an enemy. After
  that, any man may enter the empty pūtahi.
- Read this way the side that moves second can win at once, as the
  program does; see RULES.txt.
:::

The rules exactly as Best gives them from Tuta Nihoniho [@best1925,
111-112]:

1. "One player has his four men on points 1, 2, 3 and 4, the other
   player puts his four on points 5, 6, 7 and 8."
2. "No jumping over an occupied point is allowed, to move a man from one
   point to another it must be moved to the next point, which must be
   unoccupied." A man may move "either way".
3. "A man can be moved to the *putahi* if it be unoccupied."
4. "B cannot open the game by moving either 6 or 7, as they are *tapu*
   (prohibited) for the time being. He can move 5 or 8 to the
   *putahi*." The reason: if he had, "A would have been blocked at the
   outset".
5. The player who is blocked "finds himself *piro*, or out", and "has to
   capitulate".

Read as it stands, rule 4 only forbids the opening move from the inner
men; after that rule 3 lets any man enter the empty pūtahi. But then the
game is soon over: after B opens with 5 and A answers 4 to 5, B must
bring his man out of the pūtahi to 4, and A wins by moving his man on 2,
whose neighbours are both his own, into the pūtahi. The program's tests
find the side that opens lost in two moves of its own whatever it does.

Tuta Nihoniho's sample game does not go that way. Replayed by the
program, with the opening player as Black:

```{.example file="mu-torere/best-game"}
```

Every man A and B move into the pūtahi comes from beside an enemy, but
for the last, which wins the game: so the sample fits neither reading
exactly. The program offers Best's words as they stand, as the second
rule set, and the stricter rule as the first.

Our choices: White begins, and a third repetition is a draw, as above.

## Playing it in the terminal

::: brief
- Arrows: move the cursor; Enter or Space: move the man under it (with
  one empty point, each man has at most one place to go). Or click a
  man. The men that may move are lit.
- When the game is over, Enter plays again.
- M: menu; F1: this help; Q: quit.
:::

Start `mu-torere`, choose a rule set, whom to play against (the
computer, easy, normal or hard, or a friend at the same keyboard) and
which side you play.

| Key | |
|:--|:--|
| Arrows | Move the cursor |
| Enter or Space | Move the man under the cursor |
| Mouse | Click a man to move it |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| Q | Quit |

The computer knows every position: hard plays perfectly, normal now and
then plays any move that does not lose at once, easy plays at random
but takes a win and avoids a loss one move away.

`mu-torere --demo` lets the computer play both sides; `--seed N` repeats
a game exactly.
