---
crate: konane
---

# Kōnane

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | Hawaiʻi; seen by Cook's expedition in 1779            |
| Players        | Two                                                   |
| Kind           | Jumping game: the last to move wins                   |
| Program        | `konane`                                              |
| Rule sets      | Kōnane after Ernst, 8 by 8; Modern Kōnane, 8 by 8; Puʻuhonua o Hōnaunau, 9 by 11; King's papamū, 14 by 17 |

::: brief
Every hollow of the board (papamū) is filled with black and white stones
in turn, so each diagonal is of one colour. Two stones are lifted out
to begin. Then a move is a jump: one of your stones
leaps an enemy stone next to it, along a row or a column, into the empty
hollow beyond, and takes it; it may jump on along the same line. Who
cannot jump has lost. The rules of each form in short follow; the full
rules, their sources and the history are in RULES.txt and the rulebook.
:::

## What survives

**The game.** Kōnane is the Hawaiian game of black and white stones on
a board of hollows, the *papamū*. The stones were "beach-worn pebbles,
coral for white, lava for black", and the hollows were cut in stone, or
marked in wood "by the insertion of bone, usually chicken (sometimes
human)" (W. T. Brigham, as quoted by Stewart Culin [@culin1899, 243]).
Boards cut into the lava survive at many places on the islands.

![A papamū cut into a lava-stone slab, with stumps for seats, in front of
the rebuilt Hale o Keawe at Puʻuhonua o Hōnaunau National Historical
Park, Hawaiʻi, 2020. Photograph: Nobody (not jj), CC BY-SA
4.0.](images/konane-papamu-honaunau.jpg){height=8cm}

**Cook's voyage.** The first European account is James King's, from
the third voyage of James Cook, in 1779: "They have a game very much
like our draughts; but, if one may judge from the number of squares, it
is much more intricate. The board is about two feet long, and is divided
into two hundred and thirty-eight squares, of which there are fourteen
in a row" [@cookking1784, 144-145]. Culin gives the end of the sentence:
"and they make use of black and white pebbles, which they move from
square to square" [@culin1899, 243].

**The sizes of the boards.** They varied. Brigham found "no definite
number of places or arrangement", and two boards in the Bishop Museum
have 180 and 83 places [@culin1899, 243]. Nathaniel Emerson, in his notes
to David Malo's *Hawaiian Antiquities*, had seen boards "with nine on a
side, making eighty-one in all" and others "longer in one direction
than the other" [@malo1903, 306]. Michael Ernst reports that the twenty
boards described by Kenneth Emory (1924) and Peter Buck (1957) average
134 hollows, from 8 to 13 rows and 8 to 20 columns, five of them square
[@ernst1995, 19, citing Emory and Buck].

**The rules.** None of the early writers gives them in full. Culin
gathers what there was in 1899 [@culin1899, 243-244]: in Lorrin
Andrews' dictionary, "one removes one and the other jumps, as in
checkers"; for Peter Corney, "the game is decided by one party losing
all his pieces". Emerson thought the game "consists in moving one's
pieces in such a way as to compel the opponent to take them"
[@malo1903, 306]. The rules played today, in which the side that cannot
jump loses, are set out by Ernst, with the Hawaiian words, after Emory
and Buck [@ernst1995, 1, 19-20], and by the National Park at
Puʻuhonua o Hōnaunau [@nps-konane]. They differ on the opening, and the
program offers both.

![Learning kōnane on a wooden board with light and dark stones, during
Merrie Monarch Festival week at Hawaiʻi Volcanoes National Park.
National Park Service photograph by Janice Wei. Public
domain.](images/konane-hawaii-volcanoes-nps.jpg){width=55%}

**Kamehameha.** The leaflet of Puʻuhonua o Hōnaunau tells that King
Kamehameha "was an excellent player, sometimes beating his opponent in
one move", and that "his chiefs and queens were no match for him"
[@nps-konane].

## The board

![The papamū of eight by eight, filled for the start: a1 black, and
each diagonal of one colour.](figures/konane-board.svg){width=45%}

This book names the hollows by column, a letter from the left, and row,
a number from the top, so a1 is the top left corner, as the program
shows them beside the board. The top left stone is black.

A jump goes along a row or a column, never diagonally: a stone leaps an
enemy stone next to it into the empty hollow straight beyond, and the
enemy stone is taken. Where several jumps in a row are allowed, each
lands on an empty hollow beyond an enemy stone, all in one straight
line; a move never turns a corner.

## Kōnane after Ernst

::: brief
- 8 by 8, every hollow filled. Black begins.
- Black lifts one of his stones in the middle (one of the central four)
  or at a corner; White then lifts one of his next to that hollow.
- Then each in turn jumps: over an enemy stone next to his, along a row
  or column, into the empty hollow beyond, taking it. He may jump on in
  the same direction, over stone after stone, and stop when he likes;
  never diagonally, never turning.
- Who cannot jump has lost. Jumping is never forced beyond that.
:::

Ernst's account [@ernst1995, 1, 20]:

1. "Initially, all the locations are occupied, with black and white
   pieces alternating."
2. The first stone lifted "must be the center stone, one laterally next
   to it, or one at a corner. The second player picks up an adjacent
   stone of the other color; if the first player selected a stone
   adjacent to the center, then the second player must take the center
   stone."
3. "A player moves by jumping a piece over an adjacent opposing piece
   into an empty location just beyond. The jumped piece is removed from
   the board. Multiple jumps are permitted within a single move so long
   as the jumped pieces lie in a straight line, each separated by
   exactly one empty location."
4. "The first player unable to move loses."

The Hawaiian words, as Ernst gives them: *piko*, the centre; *kahola*,
a jump over several stones; *make*, to lose, and *ai*, to win
[@ernst1995, 19-20].

Our choices:

- The board is 8 by 8. "Modern papamū are often configured with even,
  square numbers of rows," the park at Puʻuhonua o Hōnaunau writes,
  where "old papamū" were often "intentionally made to be asymmetrical"
  [@nps-papamu]; see the other rule sets.
- Black begins and lifts first; Ernst says the players decide who picks
  up the first stone. Black also jumps first.
- On a board of even sides there is no single centre: the "middle" is
  the four central hollows, and Black lifts one of his two there. On a
  board of odd sides, a stone next to the centre may be lifted, and then
  White must lift the centre, as Ernst says.

**Examples.** These are made by the program's own rules engine each
time the book is built. X is a black stone, O a white one, . an empty
hollow.

The opening:

```{.example file="konane/opening"}
```

Jumps in a line:

```{.example file="konane/jumps"}
```

![The program playing Ernst's rules, the computer against itself.
](screens/konane/ernst.svg){width=100%}

## Modern Kōnane

::: brief
- As Kōnane after Ernst, but a move is a single jump: one enemy stone
  taken, never more.
:::

What Ernst calls Modern Konane, after Sizer: "a simpler variant, which
we call Modern Konane (contrasted to Ancient Konane), prohibits multiple
jumps in a single move: Each move consists of jumping exactly one
adjacent enemy piece" [@ernst1995, 1]. Everything else is as above, with
the same choices.

## Puʻuhonua o Hōnaunau

::: brief
- 9 rows by 11, as the old stone papamū at Puʻuhonua o Hōnaunau.
- Black lifts any one of his stones; White any one of his.
- Then jumps as after Ernst: on along a line if you like, never
  turning. The player who makes the last jump wins.
:::

![A boy playing kōnane on a wooden papamū with pieces of dark lava stone
and white coral, Hawaiʻi, 2005. U.S. Marine Corps photograph by Cpl.
Rich Mattingly. Public domain.](images/konane-wooden-papamu-2005.jpg){width=38%}

The rules of the National Park at Puʻuhonua o Hōnaunau, from its
leaflet [@nps-konane]:

1. "Fill all the holes alternately with light and dark pebbles. To check
   readiness for play, look across the papamu from one corner to the
   opposite. Each diagonal row should be the same color."
2. "The player with the dark pebbles goes first. 'Black' removes a black
   stone from anywhere on the board. 'White' removes a white stone from
   anywhere on the board."
3. "In any move, a player moves only one pebble, jumping any direction
   but diagonally. You cannot move in more than one direction in a
   turn; you may choose to make multiple jumps but you don't have to."
4. "The player who makes the last jump wins."

The park's own old stone papamū has its hollows "in 9 rows by 11"
[@nps-papamu], and that is the board here.

Our choice: who begins comes from the leaflet, and so does everything
else; the program adds nothing.

## King's papamū

::: brief
- 14 rows by 17: the board of 238 squares, 14 in a row, that James King
  saw in 1779.
- Ernst's rules, as in the first rule set.
:::

The board King describes [@cookking1784, 144]: 238 squares, fourteen in
a row, so seventeen rows of fourteen, here turned to fourteen rows of
seventeen so that it fits the screen. King gives no rules; these are
Ernst's, with the same choices, on the long board.

## Playing it in the terminal

::: brief
- Arrows: move the cursor; Enter or Space: lift a lit stone (at the
  start), or pick up a stone and land it on a lit hollow; Esc: let go.
  Or click.
- The stones a jump would take turn red when the cursor is on its
  landing.
- When the game is over, Enter plays again.
- M: menu; F1: this help; Q: quit.
:::

Start `konane`, choose a rule set, whom to play against (the computer,
easy, normal or hard, or a friend at the same keyboard) and which side
you play.

| Key | |
|:--|:--|
| Arrows | Move the cursor |
| Enter or Space | Lift a stone; pick up a stone, then land it on a lit hollow |
| Esc | Let go of the stone |
| Mouse | Click a stone, then a hollow |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| Q | Quit |

The computer searches ahead and judges a position by how many moves each
side has left: easy one move ahead and now and then any move at all,
normal three, hard as deep as it can in a second and a half.

`konane --demo` lets the computer play both sides; `--seed N` repeats a
game exactly.
