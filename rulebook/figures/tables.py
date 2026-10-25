"""Draws the boards of tabula and duodecim scripta (SVG and plain text) with
the points numbered as tabula/src/rules.rs numbers them.

Tabula: two rows of twelve points, 1 to 12 along the bottom from the right,
13 to 24 along the top from the left. Duodecim scripta: three rows of
twelve, 1 to 12 along the middle row from the left, 13 to 24 back along the
top, 25 to 36 along the bottom (Murray's route, after the lettered board
from Ostia).

make_figures.py calls write() with its own folder.
"""

from pathlib import Path

LINE = "#2b2b2b"
BOARD = "#e9e1cf"
RED = "#b5432a"
DARK = "#5a6b4c"

W, H = 40, 120


def board_svg(rows, label):
    """rows: list of lists of point numbers, left to right, top row first."""
    gap = 26
    pad = 30
    width = 2 * pad + 12 * W + gap
    rowh = H if len(rows) == 2 else 84
    height = 2 * pad + len(rows) * rowh + (len(rows) - 1) * 18
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" '
        f'height="{height}" font-family="Libertinus Serif, Georgia, serif">',
        f'<rect x="0" y="0" width="{width}" height="{height}" rx="10" fill="{BOARD}" stroke="{LINE}" stroke-width="2"/>',
        f'<rect x="{pad + 6 * W}" y="{pad - 10}" width="{gap}" height="{height - 2 * pad + 20}" fill="{LINE}" opacity="0.15"/>',
    ]
    for r, row in enumerate(rows):
        y0 = pad + r * (rowh + 18)
        down = r < len(rows) - 1 if len(rows) == 2 else r != 2
        for i, n in enumerate(row):
            x0 = pad + i * W + (gap if i >= 6 else 0)
            fill = RED if n % 2 == 0 else DARK
            if down:
                pts = f"{x0 + 3},{y0} {x0 + W - 3},{y0} {x0 + W / 2},{y0 + rowh - 20}"
                ty = y0 + rowh - 4
            else:
                pts = f"{x0 + 3},{y0 + rowh} {x0 + W - 3},{y0 + rowh} {x0 + W / 2},{y0 + 20}"
                ty = y0 + 14
            p.append(f'<polygon points="{pts}" fill="{fill}" opacity="0.85"/>')
            p.append(f'<text x="{x0 + W / 2}" y="{ty}" font-size="14" text-anchor="middle" fill="{LINE}">{n}</text>')
    p.append(f'<text x="{width / 2}" y="{height - 8}" font-size="13" text-anchor="middle" fill="{LINE}">{label}</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def board_text(rows, note):
    lines = []
    for row in rows:
        cells = [f"{n:>3}" for n in row]
        lines.append("  " + " ".join(cells[:6]) + " | " + " ".join(cells[6:]))
    lines += ["", note]
    return "\n".join(lines) + "\n"


def write(out: Path):
    tab = [list(range(13, 25)), list(range(12, 0, -1))]
    (out / "tabula-board.svg").write_text(board_svg(tab, "White's way: 1 to 24, then off"))
    (out / "tabula-board.txt").write_text(
        board_text(
            tab,
            "The points as White counts them, 1 to 24, then off. Black counts the\n"
            "other way (from 24) when the colours run opposite ways, the same way\n"
            "when they run alike.",
        )
    )
    xii = [list(range(24, 12, -1)), list(range(1, 13)), list(range(25, 37))]
    (out / "duodecim-board.svg").write_text(board_svg(xii, "Both sides' way: 1 to 36, then off"))
    (out / "duodecim-board.txt").write_text(
        board_text(
            xii,
            "Murray's route: in along the middle row (1 to 12), back along the top\n"
            "(13 to 24), along the bottom (25 to 36) and off. Both sides the same.",
        )
    )
