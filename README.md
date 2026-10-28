# Ancient Games

The oldest board games of the world, in the terminal, each with every
honest reconstruction of its rules to choose from and a computer opponent
at three strengths.

| Game | Binary | Rule sets |
|---|---|---|
| Royal Game of Ur | `royal-ur` | Finkel 2017, British Museum 1990s, R. C. Bell's betting game, the Babylonian tablet game |
| Senet | `senet` | Kendall with five pieces, Kendall with seven, Bell 1960 |
| Mehen | `mehen` | James Masters' 2024 reconstruction, for two, four or six players |
| Hnefatafl | `hnefatafl` | Copenhagen, Fetlar, Brandubh, Tablut after Linnaeus, Tablut after Bell 1960 |
| Fox and Geese | `fox-and-geese` | R. C. Bell's 13 and 17 geese |
| Mancala | `mancala` | Oware (Abapa), Wari after Bell, Lane's Mankal'ah of the ignorant and of the wise, Kalah (Champion 1955), Congkak (Hellier 1907), Pallanguli (Durai 1928), Toguz Kumalak, Bohnenspiel |
| Morris | `morris` | Nine Men's Morris (today's rules, and after Bell), Morabaraba (Mind Sports South Africa), Six Men's Morris after Bell, Three Men's Morris after Bell, and placing only |
| Mū tōrere | `mu-torere` | The pūtahi beside an enemy (as most give it today), and Best 1925 as he wrote it |
| Kōnane | `konane` | Ernst 1995 (8 by 8), Modern Kōnane, Puʻuhonua o Hōnaunau (9 by 11), King's papamū of 1779 (14 by 17) |
| Seega | `seega` | Lane 1836, on his boards of five, seven and nine |
| Alquerque and Fanorona | `alquerque` | Alquerque after Alfonso X (1283) and after Bell; Fanorona today (Schadd et al. 2008) and after Montgomery 1886; the vela; Fanoron-dimy |
| Tabula | `tabula` | Becq de Fouquières and Austin (the colours opposite), Bell (the same way), Ludus duodecim scriptorum after Murray |
| Pachisi and chaupar | `pachisi` | Pachisi after Tylor 1879 and after Falkener 1892 (and Bell); chaupar with long dice after Abu'l-Fazl |
| Shatranj | `shatranj` | As-Suli's shatranj, the Medinese victory, chaturanga with al-'Adli's Indian rules and three Indian elephants |
| Patolli | `patolli` | R. C. Bell's suggested rules, on Durán's equipment |

Every game: play the computer (easy, normal or hard) or a friend at the
same keyboard; keys and mouse; F1 for the rules, where each one comes
from, and what is known of the game. Made for terminals of 100x30 and
up, with true colour and UTF-8.

The full rules with their sources are in the rulebook (`rulebook/`, a
PDF and HTML book); a game whose chapter is written shows it under F1
and ships it as `RULES.txt` beside the program.

Options: `--seed N`, `--demo` (the computer plays both sides),
`--quit-after SECONDS`. For a session log:
`RUST_LOG=debug <binary> 2>game.log`.

Build: `cargo build --release` (Rust 1.88 or newer).

Part of [Asciiverse](https://asciiverse.sh).

## Licence

GNU General Public License, version 3 or (at your option) any later
version; see [LICENSE](LICENSE).
