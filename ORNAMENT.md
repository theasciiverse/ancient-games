# Ornament

An idea, recorded 2026-10-26. Started the same day (David: "a nice
development we want to build on"): the technique in `core`, tried on Ur;
see "Done" at the end.

From the backgammon session (a separate title, which shares no code with
this collection): its board is decorated with braille ornament, and David
asked whether the Ancient Games should have some too. Here, the ornament
would be historical: many of these boards survive as decorated objects.

**The technique.** A braille character (U+2800-U+28FF) is a 2 x 4 grid
of dots in one cell, and in most terminal fonts the dots are close to
square, so fine curves and patterns can be worked out in dot space and
drawn cell by cell. About 50 lines: a function from a dot (x, y) to on or
off, packed into braille characters with one colour per cell. Write it
fresh here; nothing is copied from the backgammon repo.

**Where it goes.**
- The margins around the board, only when the terminal is larger than
  100x30, so the target size stays as it is.
- A band or rim inside the board's frame, where there is room.
- Never over the pieces, the highlights or the numbers.
- An off switch (a key and a `--plain` option): some fonts draw braille
  thin or spaced apart.

**In this collection's style.** Use the amber palette and these frames,
not the backgammon look (brass, walnut, turquoise). Ornament in dim
tones of the game's own colours, so it stays behind the play.

**Sources, as for the rules.** Each pattern names the object or tradition
it follows, in HELP.txt and the rulebook chapter, like the rule sets do.
Where a living tradition is involved (Māori, Hawaiian, Akan), find
published sources and, if possible, people from that tradition, before
drawing anything.

**What was not checked:** everything below is from memory, not read
against sources. Check each object and pattern before using it.

| Game | Ornament idea | Possible source (to check) |
| --- | --- | --- |
| Ur | Rosettes on the five rosette squares; shell and lapis tones; the board's eye and dot patterns | The Royal Game of Ur, British Museum, from the Royal Cemetery at Ur (Woolley) |
| Senet | Lotus or papyrus frieze; checker band; hieroglyphs on the last squares | The senet boxes from Tutankhamun's tomb (Egyptian Museum, Cairo) |
| Mehen | The coiled serpent itself, scale by scale | Surviving Early Dynastic mehen boards |
| Seega | Plain, as Lane describes it played in the sand; a sand texture at most | Lane, *Manners and Customs of the Modern Egyptians* (1836) |
| Tabula | Greek key, guilloche (twisted rope), mosaic borders | Roman floor mosaics; inscribed tabula boards |
| Morris | Guilloche or key border, or carved-stone plainness | Boards cut into Roman steps; the Gokstad ship board |
| Alquerque | Moorish tile stars and interlace; manuscript borders | Alfonso X, *Libro de los juegos* (1283), its illuminations |
| Hnefatafl | Knotwork and animal interlace (Borre, Urnes styles) | The Ballinderry board (National Museum of Ireland); the Gokstad board |
| Fox and Geese | Knotwork, or plain carved wood | European folk boards (needs a better source) |
| Shatranj | Persian arabesques and girih star patterns | Islamic geometric ornament; the shatranj manuscripts (to find) |
| Pachisi | Embroidered cloth: cross-shaped board, floral borders | Cloth pachisi boards; the courtyard board at Fatehpur Sikri |
| Patolli | Aztec step-fret (xicalcoliuhqui); the cross-shaped board | Codex Magliabechiano's patolli scene |
| Mancala | Carved wood; Akan adinkra symbols or kente-like bands, per rule set | Carved boards in museum collections; adinkra references |
| Kōnane | Kapa (bark cloth) patterns; the stone papamū | Hawaiian kapa collections (Bishop Museum); consult |
| Mū tōrere | Kōwhaiwhai rafter patterns; the eight-pointed board | Māori sources; consult before using |

**First steps, if David wants it.**
1. Try it on one game whose object is well documented: Ur (rosettes) or
   Hnefatafl (knotwork).
2. Check how it looks at 100x30 (where it must not appear or crowd) and
   at larger sizes, by painting frames to images.
3. Make sure the rulebook capture (`core/src/capture.rs`, SVG) draws
   braille properly.
4. Only then decide whether every game gets it.

## Done (2026-10-26)

- `core/src/ornament.rs`: `paint` (one colour) and `paint_in` (a colour
  per dot, a cell takes its majority colour) draw a dot pattern over an
  area on blank cells only, so ornament drawn last can never cover a
  piece, highlight or number; `dim` mixes a colour toward the background;
  `rosette` gives the petal of a dot. `--plain` and the O key (handled in
  `shell::run`, so every game has it) turn it off.
- `core/src/capture.rs`: braille drawn as round dots, evenly spaced, so
  the rulebook pictures do not depend on a font; `--size WxH` draws a
  picture at another size (with `--rulebook-screen`), to see larger
  terminals.
- Ur (`ur/src/ui.rs`, `draw_rim`, `draw_margins`): checked against the
  British Museum's record for 120834 (via Wayback; the live page blocks
  scripts) and a CC0 photograph on Commons, both in
  `~/references/ancient-games/ornament/ur/`. The rim carries a row of shell
  lozenges (the top's border); beyond 100x30, if the margins have room
  (about 140 columns and up), the rosettes in outline, eight petals red and
  blue in turn. The eye plaques and red and shell strips of the box's
  sides are not used yet. Chapter 01 has "The ornament", with the sources.
- Looked at as images (typst renders the SVG to PNG; snap's typst cannot
  read /tmp, so render under `target/`): 100x30, 120x34 (margins too narrow,
  no rosettes), 140x40, 180x50. Not yet seen in a real terminal font.
- Found on the way: Ur's layout put the key line one row below the screen
  at some heights (120x34); fixed.

**Next:** David to look at it in a terminal (`royal-ur`, a large window;
O toggles). Then Hnefatafl (knotwork, after the Ballinderry board) or the
rest one by one, each checked against its object first.
