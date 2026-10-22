"""Draws the Fox and Geese board figure (SVG and plain text) from the same
geometry as fox-and-geese/src/rules.rs: a 7x7 grid, points where the row
or the column is 2 to 4, lines along rows and columns only. Labels follow
Bell's fig. 103: columns A to G, rows 1 to 7 from the top.

make_figures.py calls write() with its own folder.
"""

from pathlib import Path

LINE = "#2b2b2b"
BOARD = "#f4ecdc"
FOX = "#b5432a"
CELL = 64
PAD = 48


def on_board(r, c):
    return 0 <= r < 7 and 0 <= c < 7 and (2 <= r <= 4 or 2 <= c <= 4)


THIRTEEN = [(4, c) for c in range(7)] + [(r, c) for r in (5, 6) for c in (2, 3, 4)]
EXTRA = [(2, 0), (2, 6), (3, 0), (3, 6)]
FOX_START = (2, 3)


def svg():
    w = h = 2 * PAD + 6 * CELL
    at = lambda r, c: (PAD + c * CELL, PAD + r * CELL)
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h + 10}" width="{w}" '
        f'height="{h + 10}" font-family="Libertinus Serif, Georgia, serif">'
    ]
    # The cross itself, as a light field behind the lines.
    for r in range(7):
        for c in range(7):
            if on_board(r, c):
                x, y = at(r, c)
                p.append(
                    f'<rect x="{x - CELL / 2}" y="{y - CELL / 2}" width="{CELL}" height="{CELL}" '
                    f'fill="{BOARD}"/>'
                )
    for r in range(7):
        for c in range(7):
            if not on_board(r, c):
                continue
            x, y = at(r, c)
            if on_board(r, c + 1):
                x2, _ = at(r, c + 1)
                p.append(f'<line x1="{x}" y1="{y}" x2="{x2}" y2="{y}" stroke="{LINE}" stroke-width="2"/>')
            if on_board(r + 1, c):
                _, y2 = at(r + 1, c)
                p.append(f'<line x1="{x}" y1="{y}" x2="{x}" y2="{y2}" stroke="{LINE}" stroke-width="2"/>')
    for r in range(7):
        for c in range(7):
            if not on_board(r, c):
                continue
            x, y = at(r, c)
            if (r, c) == FOX_START:
                p.append(f'<circle cx="{x}" cy="{y}" r="17" fill="{FOX}" stroke="{LINE}" stroke-width="2"/>')
                p.append(
                    f'<text x="{x}" y="{y + 6}" font-size="17" text-anchor="middle" fill="#ffffff" '
                    f'font-weight="bold">F</text>'
                )
            elif (r, c) in THIRTEEN:
                p.append(f'<circle cx="{x}" cy="{y}" r="15" fill="#ffffff" stroke="{LINE}" stroke-width="2.5"/>')
            elif (r, c) in EXTRA:
                p.append(
                    f'<circle cx="{x}" cy="{y}" r="15" fill="#ffffff" stroke="{LINE}" stroke-width="2.5" '
                    f'stroke-dasharray="5 4"/>'
                )
            else:
                p.append(f'<circle cx="{x}" cy="{y}" r="4" fill="{LINE}"/>')
    for c in range(7):
        x, _ = at(0, c)
        p.append(f'<text x="{x}" y="{PAD - 30}" font-size="16" text-anchor="middle" fill="{LINE}">{"ABCDEFG"[c]}</text>')
    for r in range(7):
        _, y = at(r, 0)
        p.append(f'<text x="{PAD - 34}" y="{y + 5}" font-size="16" text-anchor="middle" fill="{LINE}">{r + 1}</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def text():
    rows = ["      A   B   C   D   E   F   G"]
    for r in range(7):
        line = f"  {r + 1}   "
        below = "      "
        for c in range(7):
            if not on_board(r, c):
                line += "    "
                below += "    "
                continue
            if (r, c) == FOX_START:
                mark = "F"
            elif (r, c) in THIRTEEN:
                mark = "O"
            elif (r, c) in EXTRA:
                mark = "o"
            else:
                mark = "+"
            line += mark + ("---" if on_board(r, c + 1) else "   ")
            below += ("|" if on_board(r + 1, c) else " ") + "   "
        rows.append(line.rstrip())
        if r < 6:
            rows.append(below.rstrip())
    rows.append("")
    rows.append("F the fox. O the thirteen geese. o the four more geese of the")
    rows.append("seventeen-geese game. + an empty point.")
    return "\n".join(rows) + "\n"


def write(out: Path):
    (out / "fox-and-geese-board.svg").write_text(svg())
    (out / "fox-and-geese-board.txt").write_text(text())
