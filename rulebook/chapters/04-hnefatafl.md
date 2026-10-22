---
crate: hnefatafl
---

# Hnefatafl

|                |                                                       |
|:---------------|:------------------------------------------------------|
| Played         | Northern Europe, from before the Viking Age; Tablut in Lapland in 1732 |
| Players        | Two                                                   |
| Kind           | War game for two unequal sides: a king escaping, his enemies closing in |
| Program        | `hnefatafl`                                           |
| Rule sets      | Copenhagen 11x11, Fetlar 11x11, Brandubh 7x7, Tablut 9x9 (Linnaeus), Tablut 9x9 (Bell 1960) |

::: brief
Two unequal sides: the king and his defenders in the middle, twice as
many attackers round the edges. The attackers move first. Every piece
moves like a rook in chess, any number of empty squares along a row or
column, never jumping. You capture an enemy piece by closing it in
between two of yours along a row or column; only the moving side
captures. The king escapes to win; the attackers win by taking him. The
rules of each set in short follow; the full rules, their sources and the
history are in RULES.txt and the rulebook.
:::

## What survives

**The name.** *Tafl*, Old Norse for "table", was a general word for
board games. By the end of the Viking Age the game of the king was
called *hnefatafl* in Scandinavia, to tell it from *skáktafl* (chess),
*kvatrutafl* (tables) and *halatafl* (the fox game). *Hnefi*, "fist",
named the king piece [@wiki-tafl]. The tafl games were played in Norway,
Sweden, Denmark, Iceland, Britain, Ireland and Sápmi, the land of the
Sámi, and chess pushed them aside from the twelfth century [@wiki-tafl].

![A board of the 1100s found in a backyard in Trondheim during the
excavations of the 1980s, with bone pieces of the 1200s from elsewhere
in the town. NTNU University Museum. Photograph: Åge Hojem, NTNU
Vitenskapsmuseet, CC BY 2.0.](images/hnefatafl-trondheim-board.jpg){height=8cm}

**The boards and pieces.** Boards and pieces turn up across that world.
Bell describes a board fragment from a Roman Iron Age grave at Wimose on
Fyn in Denmark, before about AD 400, with at least eighteen squares
along one side, and a board from the Gokstad ship burial in Norway, cut
for Nine Men's Morris on one side and on the other for a game that may
have been hnefatafl [@bell1969, 79, 93-94]. A wooden board of seven by
seven found at Ballinderry in Ireland in 1932 has holes for pegged
pieces [@wiki-tafl]; it marks the centre and corner squares with
circles [@ashton2007, 15]. A glass piece found on Lindisfarne in 2019,
topped with white droplets thought to stand for a crown, is thought to
be a piece from hnefatafl or a game like it [@wiki-tafl].

**The sagas.** The Icelandic sagas mention the game often but never
explain it. In the *Orkneyinga saga* Earl Rögnvald Kali Kolsson puts
skill at tafl first among his nine boasts. In *Friðþjófs saga* the
king's men are red and the attackers white [@wiki-tafl]. One of the
riddles in the *Saga of Hervör and Heidrek* is answered with
hnefatafl: the darker pieces defend the *hnefi*, the white ones attack
[@cartier2011, 7; @wiki-tafl].
Whether a line of the same riddle calls the king or his maids
"weaponless" is disputed, and with it whether the king took part in
captures [@wiki-tafl].

**Other games of the family.** Brandubh, "black raven", was the Irish
form: two poems say it was played with five men against eight, one of
the five a chief, the *branán*. Tawlbwrdd was the Welsh form; Robert ap
Ifan wrote down an incomplete account of it with a drawing in a
manuscript of 1587, on a board of eleven by eleven with a king and
twelve men against twenty-four [@wiki-tafl]. No full rules of any of
these survive.

**Tablut.** The one eyewitness account is the botanist Carl Linnaeus'.
On his journey through Lapland in 1732 he saw the Sámi play a game he
called Tablut, and wrote its board, its pieces and fourteen laws into
his diary, in a mix of Swedish and Latin [@bell1969, 77-79;
@ashton2007, 5-6]. The defenders are eight Swedes and their king, the
attackers sixteen Muscovites. The board was a reindeer skin with
embroidered squares [@bell1969, 78; @wiki-tafl].

![The Tablut board as the 1811 English edition of Linnaeus' diary
printed it: the king's castle in the middle, the Swedes on the squares
marked 2 and 3, the Muscovites on the embroidered squares at each side.
Lachesis Lapponica, vol. 2, p. 55. Public
domain.](images/hnefatafl-tablut-1811-board.jpg){width=55%}

**The translation that went wrong.** The diary was first published in
English in 1811, edited by James Edward Smith, with the Tablut rules
translated by Charles Troilius [@smith1811; @wiki-tafl]. Linnaeus' ninth
law says that a piece standing between two enemies is killed, and ends
*item Rex*, "likewise the king". The translation gives it as "except
the king, who is not liable to this misfortune" [@smith1811, 57;
@ashton2007, 6, 12]. It also leaves out the last two of the fourteen
laws [@ashton2007, 1, 14]. H. J. R. Murray built his reconstruction on
that translation, and later game historians built on Murray; a king who
can only be taken by four is so strong that his side nearly always wins
[@ashton2007, 1].

![Laws 3 to 9 in the 1811 translation. The ninth ends "except the king,
who is not liable to this misfortune"; Linnaeus wrote "item Rex",
likewise the king. Lachesis Lapponica, vol. 2, p. 57. Public
domain.](images/hnefatafl-tablut-1811-rules.jpg){width=55%}

**What is reconstructed.** Everything played today. John C. Ashton
retranslated Linnaeus' Latin in 2007, Nicolas Cartier in 2011 and Olli
Salmi in 2013 [@ashton2007; @cartier2011; @wiki-tafl]; read so, the
king is taken by two like any other piece, except on and beside his
castle. The modern games grew out of the older reading: to make up
for its strong king they changed other rules, above all sending him to
the corners instead of the edge; Copenhagen builds on Fetlar
[@wiki-tafl]. The program offers both lines: the
three modern rule sets, Linnaeus' Tablut as it is now read, and R. C.
Bell's Tablut of 1960, which follows the 1811 translation.

## The boards

Every rule set here starts with the king on the middle square, the
throne, his defenders around him, and twice as many attackers at the
middle of each edge. The attackers are dark and the defenders light.

| Rule set | Board | Attackers | Defenders | King escapes to |
|:--|:--|:--|:--|:--|
| Copenhagen | 11x11 | 24 | 12 and king | a corner |
| Fetlar | 11x11 | 24 | 12 and king | a corner |
| Brandubh | 7x7 | 8 | 4 and king | a corner |
| Tablut (Linnaeus) | 9x9 | 16 | 8 and king | any edge square |
| Tablut (Bell 1960) | 9x9 | 16 | 8 and king | any edge square |

In the diagrams below A is an attacker, D a defender and K the king on
his throne; # marks a corner only the king may enter.

**Copenhagen and Fetlar, 11x11.** Five attackers along the middle of
each edge with a sixth in front of them; the defenders in a diamond
round the king. Both rule sets use the same position.

```
# . . A A A A A . . #
. . . . . A . . . . .
. . . . . . . . . . .
A . . . . D . . . . A
A . . . D D D . . . A
A A . D D K D D . A A
A . . . D D D . . . A
A . . . . D . . . . A
. . . . . . . . . . .
. . . . . A . . . . .
# . . A A A A A . . #
```

**Tablut, 9x9.** Linnaeus' setup, used by both Tablut rule sets: the
Muscovites in four groups of four shaped like a T, the Swedes in a
cross. The corners are ordinary squares.

```
. . . A A A . . .
. . . . A . . . .
. . . . D . . . .
A . . . D . . . A
A A D D K D D A A
A . . . D . . . A
. . . . D . . . .
. . . . A . . . .
. . . A A A . . .
```

**Brandubh, 7x7.** Two attackers on each side, four defenders round
the king.

```
# . . A . . #
. . . A . . .
. . . D . . .
A A D K D A A
. . . D . . .
. . . A . . .
# . . A . . #
```

The positions are those of OpenTafl's source [@opentafl]. The 7x7
agrees with the World Tafl Federation's Brandubh diagram, the 9x9 with
the 1811 drawing of Linnaeus' board [@wtf-brandubh; @smith1811, 55-56].

## Copenhagen 11x11

::: brief
- 11x11; 24 attackers against the king and 12 defenders. Attackers
  first.
- Only the king may stop on the throne and the corners; anyone may pass
  over the empty throne. The king helps capture.
- Corners are hostile to all; the throne always to the attackers, to
  the defenders when empty: they capture like an enemy piece.
- Shield wall: moving onto the edge to close a row of two or more enemy
  pieces along it, each with one of yours in front of it, takes the row
  (a corner may close the far end; the king is never taken this way).
- The king is taken by attackers on all four sides (three beside the
  throne), never on the edge.
- Defenders win when the king reaches a corner, or sits on the edge,
  able to move, in a fort the attackers can never break.
- Attackers win by taking the king or shutting in every defender. The
  same position a third time loses for the defenders; a side that
  cannot move loses.
:::

The rule set most played in tournaments today [@wiki-tafl]. Aage
Nielsen, Adam Bartley and Tim Millar drew up the Copenhagen rules in 2012; the program
follows them as set out on Nielsen's site [@nielsen-copenhagen]. These
are given in full; the other rule sets are told by how they differ.

1. The attackers move first; then the players take turns.
2. Every piece moves like a rook in chess: any number of empty squares
   along a row or a column. It cannot jump over pieces.
3. Captures: a piece other than the king is captured when the enemy
   closes it in between two of their pieces on opposite sides, along a
   row or a column (not diagonally), or between one of their pieces and
   a hostile square. Only the moving side captures, so a piece may move
   in between two enemies safely. One move can capture several pieces.
   The king takes part in captures.
4. Restricted squares: only the king may stop on the throne and the
   four corners. The king may go back onto the throne, and any piece may
   pass over the empty throne.
5. Hostile squares: the corners are hostile to every piece. The throne
   is always hostile to the attackers, and hostile to the defenders only
   when it is empty. The edge of the board is not hostile.
6. Shield wall: a row of two or more pieces along the edge, each with an
   enemy piece directly in front of it, is captured whole when the enemy
   closes both ends of the row. A corner may close one end. The king may
   help close a shield wall, as one of the pieces in front or at an end;
   a king inside an attacked row is not captured with it.
7. The king is captured when attackers stand on all four sides of him.
   Next to the throne, three attackers on the other three sides are
   enough. On the edge he cannot be captured.
8. The defenders win when the king reaches a corner, or when he builds
   an exit fort: he stands on the edge, is able to move, and the
   attackers can never break the fort.
9. The attackers win when they capture the king, or when they close an
   unbroken ring round the king and all his remaining defenders.
10. Perpetual repetition is forbidden: it loses for the defenders.
11. A side that cannot move loses.

Our choices:

- "Perpetual repetition" is read as the same position, with the same
  side to move, arising for the third time. The defenders then lose,
  whichever side made the repeating move.
- A shield wall is captured only when the moving piece closes one end
  of it, standing on the edge.
- Whether a fort can never be broken is judged this way: the squares
  the king can reach hold no attacker and no attacker touches them, and
  every defender in the wall has, along its row and along its column, a
  side that is the board's edge, the fort's inside or another wall
  piece, so that no enemy can close it in.
- A ring counts when no defender has a path to any edge square through
  empty squares and other defenders.
- The rules call a game a draw when it cannot be ended, for example
  when both sides have too few pieces. The program does not judge that:
  such a game ends by repetition, which here loses for the defenders.

![The program playing the Copenhagen rules, the computer against itself,
fifteen seconds in.](screens/hnefatafl/copenhagen.svg){width=100%}

**Examples.** These are made by the program's own rules engine each
time the book is built: the position is set up, the move is played by the
engine, and the board is drawn before and after. A is an attacker, D a
defender, K the king, + an empty throne or corner, . an empty square;
squares are named by file (a, b, c from the left) and rank (1, 2, 3 from
the bottom), as tafl players write them.

A defender caught between two attackers is captured:

```{.example file="hnefatafl/capture"}
```

The king reaching a corner wins for the defenders:

```{.example file="hnefatafl/king-escapes-corner"}
```

## Fetlar 11x11

::: brief
- As Copenhagen: board, setup, moves, captures, hostile squares, the
  ring, attackers first.
- No shield wall and no exit fort: the king wins only by a corner.
- The king is taken by attackers on all four sides (three beside the
  throne); on the edge only if he is the last of his side and attackers
  stand on every side of him.
- The same position a third time is a draw. A side that cannot move
  loses.
:::

The Fetlar Hnefatafl Panel drew up the Fetlar rules in 2007; they are
followed as Nielsen's site gives them [@nielsen-fetlar]. Fetlar is in
Shetland, where the World Quickplay Hnefatafl Championships are held
each summer; the Fetlar rules were for some time the international
standard before Copenhagen built on them [@wiki-tafl]. Board, setup,
moves, captures, restricted and hostile squares, the king's capture and
the ring are as in Copenhagen. The differences:

1. There is no shield wall and no exit fort. The king's side wins only
   by reaching a corner.
2. On the edge the king can still be captured if he is the last piece
   of his side and is surrounded on every side.
3. Repetitions are allowed; a game that cannot be ended, because of
   repetitions or because both sides have too few pieces, is a draw.

Our choice: a game is drawn when a position, with the same side to
move, arises for the third time. As in Copenhagen, the program does not
otherwise judge that a game cannot be ended.

## Brandubh 7x7

::: brief
- 7x7; 8 attackers against the king and 4 defenders. Attackers first.
- Moves and captures as Copenhagen; only the king may stop on the
  throne and the corners.
- The king escapes to any corner. Corners are hostile to everyone, the
  king too.
- The king is taken like any piece, between two attackers or an
  attacker and a corner; on the throne it takes all four sides. The
  empty throne is never hostile to him.
- Attackers also win by shutting in every defender. The same position a
  third time loses for whoever made it; a side that cannot move loses.
:::

The Irish game, in the World Tafl Federation's official rules of 2020:
an Irish king, the *branán*, and his four princes against eight Viking
invaders [@wtf-brandubh]. Moves, the throne and the ring work as in
Copenhagen. The differences:

1. The corners are the four Irish provinces. The king escapes to any of
   them. They are hostile to both sides, the king included.
2. The throne is never hostile to the king, always hostile to the
   attackers, and hostile to the defenders only when the king is not on
   it.
3. The king is captured like any other piece, between two attackers or
   an attacker and a hostile square. Only on his throne do the attackers
   need all four sides of him. He takes part in captures.
4. A ring round the king and all his defenders wins for the attackers
   only if it does not use the edge of the board.
5. A situation repeated for the third time in a row loses for the
   player who chose to repeat it. A player who cannot move loses.
6. A full match is two games, the players changing sides. The winner
   has won both, or has won his game in the fewest moves.

Our choices:

- Any third occurrence of a position, with the same side to move,
  counts, not only three in a row; the side whose move made it loses.
- The program plays single games, not the two-game match.
- The ring is judged as in Copenhagen: it counts when no defender has a
  path to any edge square. So a ring that uses the edge never counts, in
  Brandubh or in any other set.

![The program playing Brandubh.](screens/hnefatafl/brandubh.svg){width=100%}

**Example.** The corners are hostile: one attacker is enough to capture
a defender against a corner.

```{.example file="hnefatafl/corner-hostile"}
```

## Tablut 9x9 (Linnaeus)

::: brief
- 9x9; 16 attackers (Muscovites) against the king and 8 defenders
  (Swedes). Attackers first.
- No special corners. Only the king may stop on the throne; anyone may
  pass over it empty. It is hostile to the attackers, and to the
  defenders when empty.
- The king is taken like any piece, between two attackers; on the
  throne it takes four, beside it three with the throne the fourth.
- The king escapes to any square on the edge.
- Attackers also win by shutting in every defender. The same position a
  third time loses for the defenders; a side that cannot move loses.
:::

Tablut as Linnaeus recorded it, in today's reading of his diary
[@ashton2007; @cartier2011]. The program plays it by the historical
rules on Nielsen's site, which give the king a capture by two and
escape to the edge [@nielsen-historical]. Moves and captures are as in
Copenhagen; the defenders are the Swedes, the attackers the Muscovites.
The differences:

1. There are no restricted corners. The only special square is the
   throne, the *konakis*: only the king may stop there, and any piece
   may pass over it when it is empty. It is always hostile to the
   attackers and hostile to the defenders when empty.
2. The king is captured like any other piece, between two attackers,
   except on and next to the throne. On the throne the attackers must
   stand on all four sides of him; next to it, on the other three.
3. The Swedes win when the king reaches any square on the edge.
4. The Muscovites win by capturing the king, or by closing a ring round
   him and all the Swedes.
5. Perpetual repetition loses for the Swedes. A side that cannot move
   loses.

Linnaeus himself does not say who moves first, and reconstructions
differ: Ashton lets the Swedes start, Cartier the Muscovites
[@ashton2007, 3; @cartier2011, 6]; the historical rules followed here
give the first move to the attackers [@nielsen-historical]. They differ
on the edge too: Ashton reads the Muscovites' embroidered starting
squares as citadels the king may not escape to, Cartier and the rules
followed here make every edge square a way out [@ashton2007, 2;
@cartier2011, 6]. Linnaeus' law that no one may enter the castle is
read on Nielsen's site as allowing the king back onto it and others
over it when empty; Wikipedia notes that Tablut is played both ways
[@wiki-tafl].

Our choice: as in Copenhagen, the repetition rule is read as the third
occurrence of a position, which then loses for the Swedes.

**Examples.** The king is captured like any other piece, between two
attackers:

```{.example file="hnefatafl/king-two-linnaeus"}
```

The king escapes to any square on the edge:

```{.example file="hnefatafl/king-escapes-edge"}
```

## Tablut 9x9 (Bell 1960)

::: brief
- As Linnaeus' Tablut: board, setup, moves, captures, the throne, the
  king escaping to the edge, attackers first.
- The king is taken only by attackers on all four sides, or three with
  the empty throne the fourth.
- No ring rule: the attackers win only by taking the king.
- The game calls Raichi! when the king has one clear way to the edge,
  Tuichi! with two.
- The same position a third time is a draw. A side that cannot move
  loses.
:::

R. C. Bell's Tablut, from *Board and Table Games from Many
Civilizations*, first published in 1960 and read here in the second
edition [-@bell1969, 77-79]. Bell takes his account from Linnaeus'
diary in the 1811 translation [@bell1969], and his rule for the king
is that translation's: four to capture him.

1. Board and setup as in the other Tablut; only the Swedish king may
   occupy the central square, the *konakis*, or throne.
2. All pieces move like the rook in chess.
3. A piece is captured when the enemy occupies both squares next to it
   in a row or column. A piece may move safely between two enemies.
4. The king is captured when enemy pieces stand on all four squares
   round him, or on three with the throne on the fourth. Then the
   Muscovites win.
5. The Swedes win when the king reaches any square on the edge.
6. When the king has a clear way to the edge, his player must warn
   *Raichi!*; with two, *Tuichi!*, which is checkmate, since both cannot
   be blocked in one move. The program calls these out.

The 1811 text spells the second call *tuichu* [@smith1811, 57]; Bell
writes *Tuichi*, and so does the program.

Our choices, where Bell is silent:

- The Muscovites move first.
- Any piece may pass over the empty throne.
- The king takes part in captures.
- The empty throne is hostile, as in the other rule sets: always to the
  Muscovites, to the Swedes when empty. Bell's text names only enemy
  pieces as capturing; the program also lets the throne help.
- There is no ring rule. A side that cannot move loses.
- A position arising for the third time, with the same side to move,
  is a draw.

Played this way the game favours the Swedes: Ashton says games played
with a king taken only by four are greatly unbalanced towards the
defenders [@ashton2007, 1].

**Example.** The same position as for Linnaeus' Tablut: under Bell's
rules two attackers are not enough to capture the king.

```{.example file="hnefatafl/king-two-bell"}
```

## Playing it in the terminal

::: brief
- Arrows move the cursor; Enter or Space picks up a piece, then puts it
  on a lit square. Or click a piece, then a square. Esc puts it down.
- After a game: Enter or Space plays again; M or Esc goes to the menu.
- M: menu; F1 or ?: this help (Up, Down, PgUp, PgDn scroll); Q: quit.
:::

Start `hnefatafl`, choose a rule set, whether you play against the
computer (easy, normal or hard) or a person at the same keyboard, and
which side you take.

| Key | |
|:--|:--|
| Arrows | Move the cursor |
| Enter or Space | Pick up a piece, then put it on a lit square |
| Mouse | Click a piece, then a lit square |
| Esc | Put the piece down again |
| F1 or ? | The rules in short (F1, ? or Esc closes them) |
| Enter or Space, after a game | Play again |
| M, or Esc after a game | Back to the menu |
| Q | Quit |

After a game, Enter or Space starts another. `hnefatafl --demo` lets
the computer play both sides; `--seed N` repeats a game exactly.
