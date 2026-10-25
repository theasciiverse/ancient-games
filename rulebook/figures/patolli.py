"""Draws the patolli board (SVG and plain text) after Bell's figure, from
the same geometry as patolli/src/rules.rs: a cross of four arms, each two
rows of seven compartments (the last a rounded end), round four squares in
the middle; the compartments 5 and 6 from the middle of every row pinched
by black triangles. Drawn upright, as the program draws it; on the mat it
lies corner to corner (Durán's "aspa"). Red's way is numbered from its
square of the middle.

make_figures.py calls write() with its own folder.
"""

from pathlib import Path

LINE = "#2b2b2b"
MAT = "#e6d3a3"
MID = "#9a6a3e"
RED = "#b5432a"


def kind(sq):
    k = sq % 15
    if k == 0:
        return "middle"
    if k in (5, 6, 9, 10):
        return "pinched"
    if k in (7, 8):
        return "end"
    return "plain"


def cell(sq):
    limb, k = divmod(sq, 15)
    if k == 0:
        r, c = 8, 7
    elif k <= 7:
        r, c = 8 + k, 7
    else:
        r, c = 15 - (k - 8), 8
    y, x = 2 * r - 15, 2 * c - 15
    for _ in range(limb):
        y, x = -x, y
    return (y + 15) // 2, (x + 15) // 2


def svg():
    s, pad = 32, 16
    size = 16 * s + 2 * pad
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}" '
        f'font-family="Libertinus Serif, Georgia, serif">',
        f'<rect x="0" y="0" width="{size}" height="{size}" fill="{MAT}"/>',
    ]
    for sq in range(60):
        r, c = cell(sq)
        x, y = pad + c * s, pad + r * s
        k = kind(sq)
        fill = MID if k == "middle" else "#f3e6c4"
        p.append(f'<rect x="{x}" y="{y}" width="{s}" height="{s}" fill="{fill}" stroke="{LINE}" stroke-width="1.5"/>')
        if k == "pinched":
            p.append(f'<path d="M{x},{y} L{x + s},{y} L{x + s / 2},{y + s / 2} Z M{x},{y + s} L{x + s},{y + s} L{x + s / 2},{y + s / 2} Z" fill="{LINE}" opacity="0.75"/>')
        if k == "end":
            p.append(f'<circle cx="{x + s / 2}" cy="{y + s / 2}" r="{s / 2 - 4}" fill="none" stroke="{LINE}" stroke-width="1.5"/>')
        colour = "#ffffff" if k == "middle" else (RED if k != "pinched" else "#ffffff")
        p.append(f'<text x="{x + s / 2}" y="{y + s / 2 + 4}" font-size="11" text-anchor="middle" fill="{colour}">{sq}</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def text():
    grid = [["   " for _ in range(16)] for _ in range(16)]
    for sq in range(60):
        r, c = cell(sq)
        grid[r][c] = {"middle": " # ", "pinched": " X ", "end": " O ", "plain": " . "}[kind(sq)]
    lines = ["".join(row).rstrip() for row in grid]
    lines += [
        "",
        "The cross drawn upright. #: the four squares of the middle; X: the",
        "pinched compartments (two forfeits); O: the rounded ends (another",
        "throw). Red enters on the lower left square of the middle, Blue on",
        "the upper right.",
    ]
    return "\n".join(lines) + "\n"


def write(out: Path):
    (out / "patolli-board.svg").write_text(svg())
    (out / "patolli-board.txt").write_text(text())
