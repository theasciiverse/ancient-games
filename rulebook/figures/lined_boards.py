"""Draws the Alquerque and Fanorona boards (SVG and plain text) from the same
geometry as alquerque/src/rules.rs: a grid of points joined along the rows
and columns, and along the diagonals through every point whose row and
column add up to an even number (the corners among them). Each board is
drawn with the men at the start: White at the bottom, Black at the top.

make_figures.py calls write() with its own folder.
"""

from pathlib import Path

LINE = "#2b2b2b"
BOARD = "#e9e1cf"
WHITE = "#ffffff"
BLACK = "#2b2b2b"
RED = "#b5432a"


def start(cols, rows, kind):
    """The men at the start, row by row from the top: 'W', 'B' or None."""
    mid = rows // 2
    at = []
    for r in range(rows):
        row = []
        for c in range(cols):
            if r < mid:
                row.append("B")
            elif r > mid:
                row.append("W")
            elif c == cols // 2:
                row.append(None)
            elif kind == "alquerque":
                row.append("W" if c > cols // 2 else "B")
            else:
                k = c if c < cols // 2 else c - 1
                row.append("B" if k % 2 == 0 else "W")
        at.append(row)
    return at


def svg(cols, rows, kind):
    cell, pad = 64, 44
    w, h = 2 * pad + (cols - 1) * cell, 2 * pad + (rows - 1) * cell
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="-26 0 {w + 26} {h + 22}" width="{w + 26}" '
        f'height="{h + 22}" font-family="Libertinus Serif, Georgia, serif">',
        f'<rect x="0" y="0" width="{w}" height="{h}" rx="10" fill="{BOARD}"/>',
    ]
    xy = lambda r, c: (pad + c * cell, pad + r * cell)
    for r in range(rows):
        x0, y = xy(r, 0)
        x1, _ = xy(r, cols - 1)
        p.append(f'<line x1="{x0}" y1="{y}" x2="{x1}" y2="{y}" stroke="{LINE}" stroke-width="2"/>')
    for c in range(cols):
        x, y0 = xy(0, c)
        _, y1 = xy(rows - 1, c)
        p.append(f'<line x1="{x}" y1="{y0}" x2="{x}" y2="{y1}" stroke="{LINE}" stroke-width="2"/>')
    for r in range(rows - 1):
        for c in range(cols - 1):
            if (r + c) % 2 == 0:
                (x0, y0), (x1, y1) = xy(r, c), xy(r + 1, c + 1)
            else:
                (x0, y0), (x1, y1) = xy(r, c + 1), xy(r + 1, c)
            p.append(f'<line x1="{x0}" y1="{y0}" x2="{x1}" y2="{y1}" stroke="{LINE}" stroke-width="2"/>')
    for r, row in enumerate(start(cols, rows, kind)):
        for c, man in enumerate(row):
            x, y = xy(r, c)
            if man is None:
                p.append(f'<circle cx="{x}" cy="{y}" r="5" fill="{LINE}"/>')
            else:
                fill = WHITE if man == "W" else BLACK
                p.append(f'<circle cx="{x}" cy="{y}" r="17" fill="{fill}" stroke="{LINE}" stroke-width="2"/>')
    for c in range(cols):
        x, _ = xy(rows - 1, c)
        p.append(f'<text x="{x}" y="{h + 16}" font-size="16" text-anchor="middle" fill="{RED}">{"abcdefghi"[c]}</text>')
    for r in range(rows):
        _, y = xy(r, 0)
        p.append(f'<text x="-12" y="{y + 5}" font-size="16" text-anchor="middle" fill="{RED}">{rows - r}</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def text(cols, rows, kind, note):
    lines = []
    at = start(cols, rows, kind)
    for r in range(rows):
        row = f"  {rows - r} " + "---".join("o" if m is None else m for m in at[r])
        lines.append(row)
        if r + 1 < rows:
            links = "    "
            for c in range(cols):
                links += "|"
                if c + 1 < cols:
                    links += " \\ " if (r + c) % 2 == 0 else " / "
            lines.append(links)
    lines.append("    " + "   ".join("abcdefghi"[:cols]))
    lines += ["", note]
    return "\n".join(lines) + "\n"


def write(out: Path):
    (out / "alquerque-board.svg").write_text(svg(5, 5, "alquerque"))
    (out / "alquerque-board.txt").write_text(
        text(5, 5, "alquerque", "W a white man, B a black one, o the empty centre: 12 men each.")
    )
    (out / "fanorona-board.svg").write_text(svg(9, 5, "fanorona"))
    (out / "fanorona-board.txt").write_text(
        text(9, 5, "fanorona", "W a white man, B a black one, o the empty centre: 22 men each.")
    )
