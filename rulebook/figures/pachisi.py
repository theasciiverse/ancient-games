"""Draws the pachisi board (SVG and plain text) from the same geometry as
pachisi/src/rules.rs: a cross of four arms of three columns by eight rows
round a middle square, Red's arm the lower one. Red's way is numbered:
out down the middle row (1 to 8), round the cross against the sun (9 to
76; 76 is 8 again) and back up the middle row (77 to 83) into the middle
(84). The castles of both layouts are marked: Tylor's and Falkener's.

make_figures.py calls write() with its own folder.
"""

from pathlib import Path

LINE = "#2b2b2b"
CLOTH = "#eadcbc"
MID = "#8a3326"
RED = "#b5432a"
TYLOR = [8, 13, 20, 25, 30, 37, 42, 47, 54, 59, 64, 71]
FALKENER = [8, 12, 21, 25, 29, 38, 42, 46, 55, 59, 63, 72]


def cell(t):
    """Red's step t as (row, column) on the 19 by 19 grid."""
    if 1 <= t <= 8:
        return 10 + t, 9
    if 9 <= t <= 16:
        return 18 - (t - 9), 10
    if 17 <= t <= 24:
        return 10, 11 + (t - 17)
    if t == 25:
        return 9, 18
    if 26 <= t <= 33:
        return 8, 18 - (t - 26)
    if 34 <= t <= 41:
        return 7 - (t - 34), 10
    if t == 42:
        return 0, 9
    if 43 <= t <= 50:
        return t - 43, 8
    if 51 <= t <= 58:
        return 8, 7 - (t - 51)
    if t == 59:
        return 9, 0
    if 60 <= t <= 67:
        return 10, t - 60
    if 68 <= t <= 75:
        return 11 + (t - 68), 8
    return 18, 9


def svg():
    s, pad = 30, 20
    size = 19 * s + 2 * pad
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}" '
        f'font-family="Libertinus Serif, Georgia, serif">'
    ]
    labels = {cell(t): str(t) for t in range(1, 76)}
    for r in range(19):
        for c in range(19):
            if not (8 <= r <= 10 or 8 <= c <= 10):
                continue
            x, y = pad + c * s, pad + r * s
            middle = 8 <= r <= 10 and 8 <= c <= 10
            if middle:
                continue
            p.append(f'<rect x="{x}" y="{y}" width="{s}" height="{s}" fill="{CLOTH}" stroke="{LINE}" stroke-width="1"/>')
            here = [t for t in TYLOR if cell(t) == (r, c)]
            there = [t for t in FALKENER if cell(t) == (r, c)]
            if here:
                p.append(f'<path d="M{x + 4},{y + 4} L{x + s - 4},{y + s - 4} M{x + s - 4},{y + 4} L{x + 4},{y + s - 4}" stroke="{RED}" stroke-width="2"/>')
            if there:
                p.append(f'<circle cx="{x + s / 2}" cy="{y + s / 2}" r="{s / 2 - 3}" fill="none" stroke="{LINE}" stroke-width="2"/>')
            if (r, c) in labels:
                p.append(f'<text x="{x + s / 2}" y="{y + s / 2 + 4}" font-size="11" text-anchor="middle" fill="{LINE}">{labels[(r, c)]}</text>')
    x0 = pad + 8 * s
    p.append(f'<rect x="{x0}" y="{x0}" width="{3 * s}" height="{3 * s}" fill="{MID}" stroke="{LINE}" stroke-width="1"/>')
    p.append(f'<text x="{x0 + 1.5 * s}" y="{x0 + 1.6 * s}" font-size="12" text-anchor="middle" fill="#ffffff">middle</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def text():
    grid = [["   " for _ in range(19)] for _ in range(19)]
    for r in range(19):
        for c in range(19):
            if 8 <= r <= 10 or 8 <= c <= 10:
                grid[r][c] = " . "
    for r in range(8, 11):
        for c in range(8, 11):
            grid[r][c] = "###"
    for t in TYLOR:
        r, c = cell(t)
        grid[r][c] = " X "
    for t in FALKENER:
        r, c = cell(t)
        grid[r][c] = " O " if grid[r][c] != " X " else "XO "
    lines = ["".join(row).rstrip() for row in grid]
    lines += [
        "",
        "The cross, Red's arm at the bottom; ### the middle. X: Tylor's castles",
        "(the 4th square from the middle on the outer rows); O: Falkener's (the",
        "4th from the outer end); XO: both, at the end of each middle row.",
    ]
    return "\n".join(lines) + "\n"


def write(out: Path):
    (out / "pachisi-board.svg").write_text(svg())
    (out / "pachisi-board.txt").write_text(text())
