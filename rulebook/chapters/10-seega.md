---
crate: seega
---

# Seega

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | Egypt; described by Edward Lane in 1836               |
| Players        | Two                                                   |
| Kind           | Placing and taking: kelbs set down, then moved to trap |
| Program        | `seega`                                               |
| Rule sets      | Seega of five (khamsáwee); of seven (seb'áwee); of nine (tis'áwee) |

::: brief
A square of holes, five, seven or nine a side. Each player has a kelb
(piece) for half the holes but the centre. Two each are set down on
fixed holes; then the players set down two at a time, anywhere but the
centre, until only the centre is empty. Then a move is one step, along
a row or a column, into an empty hole. A kelb that comes to stand so
that an enemy kelb lies between it and another of yours takes that
enemy kelb; as long as you can take again at once, you move again. If
you cannot move, the other must open a way by giving up a kelb of his.
The rules in short follow; the full rules, the source and the history
are in RULES.txt and the rulebook.
:::

## What survives

**The game.** Edward William Lane, who lived in Cairo in the 1820s and
1830s, describes seega in *An Account of the Manners and Customs of the
Modern Egyptians* (first published 1836). "Many of the felláheen of
Egypt also frequently amuse themselves with a game called that of the
'seega,' which may be described in a few words." The board "consists of
a number of holes, generally made in the ground; most commonly, of five
rows of five holes in each, or seven rows of seven in each, or nine
rows of nine in each: the first kind is called the 'khamsáwee seega;'
the second, the 'seb'áwee;' and the third, the 'tis'áwee'"
[@lane1836, 320-321].

**The pieces.** The holes are *'oyoon*, eyes (one *'eyn*), and the
pieces *kelbs*, dogs, "similar to those used in the game of the táb"
[@lane1836, 320-321]: there, "a little piece of stone, or dingy brick, about
the size of a walnut" for one player, and "a piece of red brick or tile"
for the other [@lane1836, 318]. The program calls the two sides Stone
and Brick after them.

**How old.** Lane says only that "several seegas have been cut upon the
stones on the summit of the great pyramid, by Arabs who have served as
guides to travellers" [@lane1836, 320-321]. Seega is often called an
ancient Egyptian game; this book has read no source that dates it
before Lane.

**Lane's rules,** in full [@lane1836, 320-321]:

> The players have each twelve "kelbs" [...]. One of them places two
> of his kelbs in the 'eyns marked *a*, *a*: the other puts two of his
> in those marked *b*, *b*: they then alternately place two kelbs in
> any of the 'eyns that they may choose, excepting the central 'eyn of
> the seega. All the 'eyns but the central one being thus occupied
> (most of the kelbs placed at random), the game is commenced. The
> party who begins moves one of his kelbs from a contiguous 'eyn into
> the central. The other party, if the 'eyn now made vacant be not next
> to any one of those occupied by his kelbs, desires his adversary to
> give him, or open to him, a way; and the latter must do so, by
> removing, and thus losing, one of his own kelbs. This is also done on
> subsequent occasions, when required by similar circumstances. The aim
> of each party, after the first disposal of the kelbs, is to place any
> one of his kelbs in such a situation that there shall be, between it
> and another of his, one of his adversary's kelbs. This, by so doing,
> he takes; and as long as he can immediately make another capture by
> such means, he does so, without allowing his adversary to move.—These
> are the only rules of the game.

For the larger boards, "in like manner", there are kelbs "to occupy all
the 'eyns excepting one" [@lane1836, 321, note 447]: twenty-four each
on the board of seven, forty on the board of nine.

## The board

This book names the holes by column, a letter from the left, and row,
a number from the top, as the program shows them beside the board; c3
is the centre of the board of five. Lane's figure of the khamsáwee
seega marks *a* on the middle hole of the top and bottom rows (c1 and
c5) and *b* on the middle hole of the outer columns (a3 and e3).

```{.example file="seega/start"}
```

## Seega of five

::: brief
- 5 by 5, twelve kelbs each. Stone has kelbs on c1 and c5, Brick on a3
  and e3. Then Stone sets down two, Brick two, and so on, anywhere but
  the centre c3.
- Brick, who placed last, moves first: a step into the centre. Our
  choice; Lane does not say who begins.
- A move: one step along a row or column into an empty hole.
- Taking: a kelb that steps so that an enemy kelb lies between it and
  another of yours, in a row or column, takes it; several at once if
  so placed. A kelb may step in between two enemies safely.
- After taking, if you can take again at once (with any kelb), you
  must move again, and only to take.
- No move: the other gives up a kelb next to one of yours, and you
  move.
- A side with no kelbs has lost. Our choice: after 40 moves with
  nothing taken or given, more kelbs win; equal is a draw.
:::

What Lane says is the game, and the program plays it as he gives it.
He leaves gaps; our choices fill them, as plainly as we could:

- **Who places first, who moves first.** Lane says neither. Stone
  places first (on *a*, *a*); Brick, who sets the last two kelbs down,
  makes the first move.
- **Steps.** Lane says only that the first move comes "from a
  contiguous 'eyn". A move here is one step to a hole next to it along
  a row or a column, never diagonally, as the holes stand in rows.
- **Taking.** The enemy kelb must lie between the kelb moved and
  another of the mover's in a row or a column, next to both. Only the
  kelb that moves takes, so a kelb may step in between two enemies
  unharmed. A step that traps two or three enemy kelbs at once takes
  them all.
- **Taking again.** "As long as he can immediately make another capture
  by such means, he does so": after a capture, if any of his kelbs can
  take with one step, he moves again and must take; when he cannot, the
  turn passes.
- **Opening a way.** If the side to move has no step at all, the other
  must give up one of his kelbs next to one of the blocked side's, which
  the blocked side can then step into. Lane gives the giver the choice.
- **The end.** Lane does not say how the game ends. A side whose kelbs
  are all taken has lost. Two kelbs can chase each other for ever, and
  a single kelb can never take, so a game in which nothing is taken or
  given for 40 moves ends there: the side with more kelbs wins, and
  equal numbers are a draw.

**Example.** Made by the program's own rules engine each time the book
is built. S is a kelb of Stone, B of Brick, . an empty hole.

```{.example file="seega/capture"}
```

Opening a way:

```{.example file="seega/way"}
```

![The program playing the board of five, the computer against itself.
](screens/seega/five.svg){width=100%}

## Seega of seven and of nine

::: brief
- The seb'áwee, 7 by 7, twenty-four kelbs each, and the tis'áwee, 9 by
  9, forty each. The same rules as on the board of five.
- The first four kelbs stand on the middle holes of the outer rows
  (Stone) and outer columns (Brick), as on Lane's board of five. Our
  choice.
:::

Lane names the two larger boards and says only that they need kelbs
for every hole but one [@lane1836, 321, note 447]. His figure shows the
board of five; on the larger boards the program sets the first four
kelbs on the same places, the middle holes of the edges.

## Playing it in the terminal

::: brief
- Arrows: move the cursor; Enter or Space: set a kelb down, give one
  up, or pick one up and put it on a lit hole; Esc: let go. Or click.
- The kelbs a step would take are marked ✕ when the cursor is on where
  it goes.
- When the game is over, Enter plays again.
- M: menu; F1: this help; Q: quit.
:::

Start `seega`, choose a board, whom to play against (the computer,
easy, normal or hard, or a friend at the same keyboard) and which side
you play.

| Key | |
|:--|:--|
| Arrows | Move the cursor |
| Enter or Space | Set a kelb down; give one up; pick one up, then put it on a lit hole |
| Esc | Let go of the kelb |
| Mouse | Click a hole, or a kelb and then a hole |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| Q | Quit |

The computer searches ahead and judges a position by the kelbs each
side has, then by the captures each could make next: easy one move
ahead and now and then any move at all, normal four, hard as deep as
it can in a second and a half (a quarter of that while placing).

`seega --demo` lets the computer play both sides; `--seed N` repeats a
game exactly.
