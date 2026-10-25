#!/usr/bin/env python3
"""Draws the rulebook's board figures, each as SVG (PDF, HTML) and as plain
text (the RULES.txt in the release archives).

The geometry is the same as the game's own (ur/src/rules.rs): row 0 is the
far side (Dark's row), row 1 the shared middle row, row 2 the near side
(Light's). Run from anywhere; the files are written beside this script.
"""

from pathlib import Path

HERE = Path(__file__).resolve().parent

SHELL = "#f4ecdc"
LINE = "#2b2b2b"
LAPIS = "#2b4c8c"
CARNELIAN = "#b5432a"
PATH = "#b5432a"
CELL = 64
PAD = 40


class Board:
    def __init__(self, name, cols, on_board, rosettes, path, marks=None, special=None):
        self.name = name
        self.cols = cols
        self.on_board = on_board
        self.rosettes = rosettes
        # Light's path: list of (row, col), position 1 first.
        self.path = path
        # Extra labels drawn in squares: (row, col) -> text.
        self.marks = marks or {}
        # Named squares, shaded, with a label: (row, col) -> text.
        self.special = special or {}

    def squares(self):
        return [(r, c) for r in range(3) for c in range(self.cols) if self.on_board(r, c)]


def ur_board():
    def on(r, c):
        return r == 1 or c not in (4, 5)

    rosettes = {(0, 0), (0, 6), (2, 0), (2, 6), (1, 3)}
    path = [(2, 3), (2, 2), (2, 1), (2, 0)] + [(1, c) for c in range(8)] + [(2, 7), (2, 6)]
    return Board("ur-board", 8, on, rosettes, path)


def twenty_squares():
    def on(r, c):
        return r == 1 or c < 4

    rosettes = {(0, 0), (2, 0), (1, 3), (1, 7), (1, 11)}
    path = [(2, 3), (2, 2), (2, 1), (2, 0)] + [(1, c) for c in range(12)]
    marks = {(2, 0): "Sw", (1, 0): "St", (1, 1): "Ra", (1, 2): "Ro", (1, 5): "Ea"}
    return Board("twenty-squares", 12, on, rosettes, path, marks)


def senet():
    # The S-shaped track: 1 to 10 on the top row, 11 to 20 back along the
    # middle, 21 to 30 on the bottom row (senet/src/rules.rs).
    path = [(0, c) for c in range(10)] + [(1, c) for c in range(9, -1, -1)] + [(2, c) for c in range(10)]
    names = {15: "Rebirth", 26: "Beauty", 27: "Water", 28: "III", 29: "II", 30: "I"}
    special = {path[h - 1]: name for h, name in names.items()}
    return Board("senet", 10, lambda r, c: True, set(), path, special=special)


def rosette_svg(cx, cy, r):
    out = []
    for i in range(8):
        out.append(
            f'<ellipse cx="{cx}" cy="{cy - r * 0.5:.1f}" rx="{r * 0.2:.1f}" ry="{r * 0.5:.1f}" '
            f'fill="{CARNELIAN}" transform="rotate({i * 45} {cx} {cy})"/>'
        )
    out.append(f'<circle cx="{cx}" cy="{cy}" r="{r * 0.22:.1f}" fill="{LAPIS}"/>')
    return "\n".join(out)


def centre(r, c):
    return PAD + c * CELL + CELL / 2, PAD + r * CELL + CELL / 2


def to_svg(board, entry_from, exit_to, row_labels):
    # Row labels go right of the board and of the "off" arrow.
    label_col = max(board.cols, exit_to[1] + 1)
    w = PAD * 2 + label_col * CELL + (120 if any(row_labels) else 0)
    h = PAD * 2 + 3 * CELL
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" '
        f'font-family="Libertinus Serif, Georgia, serif">',
        "<defs>",
        f'<marker id="arrow" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" markerHeight="7" '
        f'orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" fill="{PATH}"/></marker>',
        "</defs>",
    ]
    for r, c in board.squares():
        x, y = PAD + c * CELL, PAD + r * CELL
        parts.append(
            f'<rect x="{x}" y="{y}" width="{CELL}" height="{CELL}" fill="{SHELL}" '
            f'stroke="{LINE}" stroke-width="2"/>'
        )
        if (r, c) in board.rosettes:
            cx, cy = centre(r, c)
            parts.append(rosette_svg(cx, cy, CELL * 0.36))
        if (r, c) in board.special:
            parts.append(
                f'<rect x="{x + 3}" y="{y + 3}" width="{CELL - 6}" height="{CELL - 6}" '
                f'fill="{LAPIS}" opacity="0.15"/>'
            )
            parts.append(
                f'<text x="{x + CELL / 2}" y="{y + CELL - 9}" font-size="12" text-anchor="middle" '
                f'fill="{LAPIS}">{board.special[(r, c)]}</text>'
            )

    # Light's path, through the square centres, with arrows in and out.
    pts = [centre(*entry_from)] + [centre(r, c) for r, c in board.path] + [centre(*exit_to)]
    d = " ".join(f"{x:.1f},{y:.1f}" for x, y in pts)
    parts.append(
        f'<polyline points="{d}" fill="none" stroke="{PATH}" stroke-width="2.5" '
        f'stroke-dasharray="6 4" opacity="0.8" marker-end="url(#arrow)"/>'
    )

    for i, (r, c) in enumerate(board.path, start=1):
        x, y = PAD + c * CELL, PAD + r * CELL
        parts.append(
            f'<text x="{x + 5}" y="{y + 16}" font-size="14" fill="{LINE}">{i}</text>'
        )
        mark = board.marks.get((r, c))
        if mark:
            parts.append(
                f'<text x="{x + CELL - 5}" y="{y + CELL - 6}" font-size="13" text-anchor="end" '
                f'font-style="italic" fill="{LAPIS}">{mark}</text>'
            )

    ex, ey = centre(*entry_from)
    parts.append(f'<text x="{ex}" y="{ey + 30}" font-size="13" text-anchor="middle" fill="{LINE}">in</text>')
    ox, oy = centre(*exit_to)
    parts.append(f'<text x="{ox}" y="{oy + 30}" font-size="13" text-anchor="middle" fill="{LINE}">off</text>')

    lx = PAD + label_col * CELL + 14
    for r, label in enumerate(row_labels):
        parts.append(
            f'<text x="{lx}" y="{PAD + r * CELL + CELL / 2 + 5}" font-size="15" fill="{LINE}">{label}</text>'
        )
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def to_text(board, row_labels, entry_note):
    """A plain ASCII drawing: Light's path numbers, * for a rosette."""
    width = 5  # characters per cell including its left border

    def cell_text(r, c):
        mark = "*" if (r, c) in board.rosettes or (r, c) in board.special else ""
        if (r, c) in board.path:
            mark += str(board.path.index((r, c)) + 1)
        return mark.center(width - 1)

    lines = []
    for r in range(4):
        # Border line above row r (r == 3: the bottom border).
        line = ""
        for c in range(board.cols + 1):
            above = r > 0 and c > 0 and board.on_board(r - 1, c - 1) or r > 0 and c < board.cols and board.on_board(r - 1, c)
            below = r < 3 and (c > 0 and board.on_board(r, c - 1) or c < board.cols and board.on_board(r, c))
            line += "+" if above or below else " "
            if c < board.cols:
                edge = (r > 0 and board.on_board(r - 1, c)) or (r < 3 and board.on_board(r, c))
                line += "-" * (width - 1) if edge else " " * (width - 1)
        lines.append(line.rstrip())
        if r == 3:
            break
        row = ""
        for c in range(board.cols + 1):
            left = c > 0 and board.on_board(r, c - 1)
            right = c < board.cols and board.on_board(r, c)
            row += "|" if left or right else " "
            if c < board.cols:
                row += cell_text(r, c) if right else " " * (width - 1)
        lines.append(f"{row}  {row_labels[r]}".rstrip())
    lines.append("")
    lines.append(entry_note)
    return "\n".join(lines) + "\n"


def main():
    # The Fox and Geese cross has lines, not squares: its own drawing.
    import fox_and_geese

    fox_and_geese.write(HERE)

    # The mancala boards: pits and stores, their own drawing.
    import mancala

    mancala.write(HERE)

    # The morris boards: points and lines.
    import morris

    morris.write(HERE)

    # Mū tōrere's star and Kōnane's papamū.
    import small_games

    small_games.write(HERE)

    # The lined boards of Alquerque and Fanorona.
    import lined_boards

    lined_boards.write(HERE)

    # The boards of tabula and duodecim scripta.
    import tables

    tables.write(HERE)

    ur = ur_board()
    labels = ["Dark's row", "shared row", "Light's row"]
    (HERE / "ur-board.svg").write_text(to_svg(ur, (2, 4), (2, 5), labels))
    (HERE / "ur-board.txt").write_text(
        to_text(
            ur,
            labels,
            "Numbers: Light's path, 1 to 14; off the board is the 15th step.\n"
            "* rosette. Dark's path is the mirror image on the far row.",
        )
    )

    ts = twenty_squares()
    sn = senet()
    blank = ["", "", ""]
    (HERE / "senet-board.svg").write_text(to_svg(sn, (0, -1), (2, 10), blank))
    (HERE / "senet-board.txt").write_text(
        to_text(
            sn,
            blank,
            "Numbers: the houses, 1 to 30, along an S; both sides run the same way.\n"
            "* named houses: 15 Rebirth, 26 Beauty, 27 Water, 28 III, 29 II, 30 I.",
        )
    )

    (HERE / "twenty-squares.svg").write_text(to_svg(ts, (2, 4), (1, 12), labels))
    (HERE / "twenty-squares.txt").write_text(
        to_text(
            ts,
            labels,
            "Numbers: Light's houses, 1 to 16; off the board is the 17th step.\n"
            "* rosette. Birds come on at: Swallow 4, Storm-bird 5, Raven 6,\n"
            "Rooster 7, Eagle 10. Dark's houses mirror Light's on the far row.",
        )
    )


if __name__ == "__main__":
    main()
