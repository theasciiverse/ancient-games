---
crate: senet
---

# Senet

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | Egypt, from about 3100 BC to the Roman period          |
| Players        | Two                                                   |
| Kind           | Race game                                             |
| Program        | `senet`                                               |
| Rule sets      | Kendall with five pieces, Kendall with seven pieces, R. C. Bell 1960 |

## What survives

**The boards.** Senet is a board of thirty squares in three rows of ten.
Fragmentary boards that could be Senet have been found in First Dynasty
burials, about 3100 BC, and the first certain picture of the game is in
the Third Dynasty tomb of the official Hesy, about 2620 BC. Its name,
*znt*, means "passing" [@wiki-senet].

Many boards were made as boxes with a drawer for the pieces and dice,
and the underside often carries a second game [@bell1969, 25-26]; on the
box below it is Twenty Squares, the later form of the Royal Game of Ur. A board in the British
Museum marked with the cartouche of Queen Hatshepsut, about 1500 BC,
shows what most boards share: the fifth square from the end carries a
hieroglyph meaning a door or exit, and the last four are marked X, III,
II and I. Its drawer held twenty pieces with lions' heads, ten light and
nine dark (the twentieth, of ivory, perhaps a replacement), and four
throwing sticks. A board from Tutankhamun's tomb held two knucklebones
[@bell1969, 26].

![A game box from a burial at Thebes, about 1635 to 1458 BC: the Senet
side, three rows of ten. Its other side is the Twenty Squares board in
the chapter on the Royal Game of Ur. Metropolitan Museum of Art
16.10.475a, CC0.](images/senet-met-16-10-475.jpg){width=100%}

**The pieces and dice.** Sets often had two kinds of men, one side shaped
like pawns or cones, the other like spools [@bell1969, 26]. The game was
thrown with four sticks, and sometimes with knucklebones or a long die.

![Pieces found with that box: cones for one side, spools for the other,
and two knucklebones. Metropolitan Museum of Art 16.10.475b-k,
CC0.](images/senet-pieces-met-16-10-475.jpg){width=70%}

**Pictures of play.** A Fifth Dynasty tomb at Saqqara shows two men
playing, their pieces set alternately along the board. A tomb at Beni
Hasan of about 2000 BC shows two boards, one captioned "To play with
five", the other "Consumed", perhaps meaning the game is over
[@bell1969, 26-27]. Later the game took on a religious meaning: the dead
play it against an invisible opponent, the passage of the soul through
the underworld [@wiki-senet].

![Queen Nefertari playing Senet, from her tomb in the Valley of the
Queens (QV 66), about 1279 to 1213 BC; a facsimile at half size by Nina
de Garis Davies. Metropolitan Museum of Art 30.4.145,
CC0.](images/senet-nefertari-met-facsimile.jpg){width=60%}

**What is not known.** The rules. No text explains how Senet was played.
The game fell out of use in the Roman period [@wiki-senet]. Gustave
Jéquier proposed rules from the boards and pictures; they were later
replaced by the reconstructions of R. C. Bell and Timothy Kendall, which
the makers of modern sets have adopted [@wiki-senet]. Both are offered
here.

![Senet board of glazed faience inscribed for Amenhotep III, about 1390
to 1353 BC, with its sliding drawer and pieces. Brooklyn Museum
49.56a-b, no known copyright
restrictions.](images/senet-amenhotep-iii-brooklyn.jpg){width=65%}

## The board

![The thirty houses of Senet in the order the pieces travel, with the
named houses of Kendall's rules.](figures/senet-board.svg){#fig-senet-board width=100%}

All three rule sets number the houses the same way: 1 to 10 left to
right along the top row, 11 to 20 back along the middle row, 21 to 30
along the bottom row, then off the board. Both sides travel the same way,
which makes Senet a game of blocking and passing.

The last houses carry marks on the old boards. The rule sets name and use
them differently:

| House | Old boards | Kendall's name | Bell |
|:--|:--|:--|:--|
| 15 | | Rebirth | |
| 26 | a door | Beauty | the door: safe |
| 27 | X | Humiliation, or Water | marked: safe |
| 28 | III | the Three Truths | marked: safe |
| 29 | II | Re-Atoum | marked: safe |
| 30 | I | Horus, the last house | marked: safe |

Kendall's names are "the House of" each, as Walker and O'Keefe give them
[@walker-senet; @okeefe-senet]: Walker calls 27 the House of Humiliation
and O'Keefe the House of Water; Walker leaves 30 unnamed.

## The throws

**Four sticks** (both Kendall rule sets). Each stick has a light side and
a dark side. Count the light sides up; none up counts as 5. After moving
on a 1, 4 or 5, the player throws again.

| Light sides up | Move | Chance in 16 | Throw again |
|:--|:--|:--|:--|
| 1 | 1 | 4 | yes |
| 2 | 2 | 6 | no |
| 3 | 3 | 4 | no |
| 4 | 4 | 1 | yes |
| none | 5 | 1 | yes |

**A long die** (Bell). Four faces, 1 to 4, one throw a turn.

Our choice: Bell does not describe the die beyond its four faces; in the
program each comes up equally often.

## Kendall, five pieces

Timothy Kendall published his reconstruction in 1978 in a booklet sold
with his Senet set, *Passing Through the Netherworld*
[@kendall1978].^[Wikipedia dates Kendall's original rules to 1979
[@wiki-senet]; Walker and Finkel give 1978 [@walker-senet;
@finkel2007, 32].] The five-piece form below is what most modern sets and
online games play. Devin O'Keefe wrote it out exactly, with every
ambiguity settled, for his solution of the game, and the program follows
his text [@okeefe-senet].

1. Each side has five pieces. White starts on houses 1, 3, 5, 7 and 9,
   Black on 2, 4, 6, 8 and 10. White throws first.
2. Throw the sticks and move one piece forward that many houses. A
   player who can move must.
3. A piece may not land on its own side's piece.
4. A piece next to another of its side (on the house before or after) is
   protected and cannot be landed on.
5. Landing on a lone enemy piece swaps the two: the enemy goes to the
   house the mover came from.
6. Three or more pieces of one side in a row form a blockade: no enemy
   piece may pass over it.
7. Every piece must stop on the House of Beauty (26): no move may jump
   past it.
8. A piece that reaches the House of Water (27), from 26 on a 1, is sent
   back to the House of Rebirth (15), or if that is taken to the highest
   empty house below it. House 27 is therefore always empty.
9. Pieces on 28, 29 and 30 do not move on: they leave the board only on
   an exact 3, 2 or 1. A piece on 26 leaves on a 5.
10. Only if no piece can move forward must one move backward by the
    throw, under the same rules. Pieces on 28 to 30 never move backward.
11. After moving on a 1, 4 or 5 the player throws again; after a 2 or 3
    the turn passes. A throw with no move at all ends the turn, whatever
    it was.
12. The first side to bear off all five pieces wins.

O'Keefe computed the chance of winning from all 8,560,690,670 positions
of this game: with perfect play White, who throws first, wins 50.25% of
games [@okeefe-senet].

![The program playing Kendall's five-piece rules, the computer against
itself, thirty seconds in. Black (■) has just thrown 2: two of the four
sticks show their light sides.](screens/senet/kendall-5.svg){width=100%}

**Examples.** These are made by the program's own rules engine each
time the book is built: the position is set up, the engine lists the
moves it allows, and the first is played. ▲ is White, ■ Black, . an empty
house; only the houses that matter are shown.

Landing on a lone piece swaps the two:

```{.example file="senet/swap"}
```

A piece with a friend beside it is protected. White has no forward move,
so it must move backward:

```{.example file="senet/protected"}
```

Three in a row form a blockade that cannot be passed:

```{.example file="senet/blockade"}
```

No move may jump past the House of Beauty (26); again White must move
backward:

```{.example file="senet/beauty"}
```

From 26 a throw of 1 falls into the water (27). The piece goes back to
the House of Rebirth (15), or as here, with 15 taken, to the highest
empty house below it:

```{.example file="senet/water"}
```

## Kendall, seven pieces

Kendall's original rules, with seven pieces each, as Damian Walker sets
them out from Kendall's booklet [@walker-senet]. They differ from the
five-piece rules in these ways:

1. Each side has seven pieces, alternating along houses 1 to 14, with
   White on house 1. Black throws first.
2. A lone enemy piece that is landed on goes back to the house the mover
   came from (when moving backward, forward to it). There are no
   blockades.
3. Every piece must still stop on the House of Beauty (26).
4. A piece that lands in the House of Humiliation, the water (27), ends
   the turn, and none of that side's other pieces may move until it is
   rescued. To rescue it, move it to the House of Rebirth (15) instead of
   throwing, which ends the turn; this is not possible while house 15 is
   taken. Or throw, and bear it off on an exact 4; any other throw ends
   the turn.
5. A piece in the water is never protected. A piece landing on it
   swaps places with it, and is now in the water itself.
6. Pieces bear off from the last five houses on an exact throw. A piece
   on 28 leaves only on a 3, on 29 only on a 2; a piece on 30 leaves on
   any throw.
7. A piece landed on in houses 28 to 30 (only possible from 26) goes into
   the water, or back to 26 if the water is taken.
8. If a piece on 28 or 29 cannot bear off and there is no other piece to
   move, it moves back into the water.
9. Throws again and the winner as in the five-piece rules.

Our choice: Walker's "there are no other pieces on the board to move" is
read as "this is the side's last piece on the board". A piece on 28 or
29 falls back only then, and only if the water is empty.

**Example.** A piece in the water freezes the side's other pieces:

```{.example file="senet/drowning"}
```

## R. C. Bell 1960

R. C. Bell's suggested rules for "the Game of Thirty Squares", from
*Board and Table Games from Many Civilizations*, first published in 1960
and read here in the second edition [-@bell1969, 27-28]. A different
game from Kendall's: more pieces, captures, and a score.

1. Each side has ten men, placed alternately on the first and second
   rows (houses 1 to 20).
2. The players throw the long die; the higher throw moves first, and that
   player's men stand on the second house of each pair.
3. Throw and move one man forward that many houses. Men never move
   backward. A man may land on an empty house or on an enemy man, which
   is captured and removed from the game, but not on a man of its own.
4. The door (26) is safe from attack, but a man on it must move onto a
   marked house as soon as it can.
5. A man on a marked house (X, III, II, I: houses 27 to 30) cannot be
   captured. It stays there until an exact throw takes it off the board
   (from X a 4, from III a 3, from II a 2, from I a 1), which wins a
   point.
6. A man must leave a marked house at the first throw that allows it.
7. The players throw in turn, one move each.
8. The player who brings the most men safely off the board wins.

Our choices:

- Who throws first is decided by a coin, standing in for the throw of
  the long die.
- Men may pass over any house, marked or not.
- When a man on the door or a marked house can move, a move from such a
  house must be made before any other; if there are several, the player
  chooses.
- Bell does not say when the game ends. Here it ends when the board is
  empty, and equal scores are a draw.

![The program playing Bell's rules: ten men each, the long die below the
board, and the count of men borne off and taken by each player's
name.](screens/senet/bell.svg){width=100%}

**Examples.** Under Bell's rules a man is captured even when it has a
friend beside it:

```{.example file="senet/bell-capture"}
```

A man on the door (26) must move onto a marked house at the first
chance, before any other move:

```{.example file="senet/bell-forced"}
```

## Playing it in the terminal

Start `senet`, choose a rule set and who plays each side: the computer
(easy, normal or hard) or a person at the same keyboard.

| Key | |
|:--|:--|
| Space or Enter | Throw |
| Arrows or Tab | Choose a piece |
| Enter | Move it |
| R | Rescue a piece from the water (Kendall, seven pieces) |
| Mouse | Click a lit piece |
| F1 | These rules |
| M | Back to the menu |
| Q | Quit |

`senet --demo` lets the computer play both sides; `--seed N` repeats a
game exactly.
