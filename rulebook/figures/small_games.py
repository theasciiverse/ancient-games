"""Draws the Mū tōrere and Kōnane boards (SVG and plain text) from the same
geometry as mu-torere/src/rules.rs and konane/src/rules.rs.

Mū tōrere: eight kewai on a ring, numbered 1 to 8 clockwise from the top,
each joined to its neighbours and to the pūtahi in the middle. Kōnane: a
grid of hollows, filled in turn with black and white stones, a1 (the top
left) black.

make_figures.py calls write() with its own folder.
"""

import math
from pathlib import Path

LINE = "#2b2b2b"
BOARD = "#f4ecdc"
RED = "#b5432a"


def mu_torere_svg():
    size, r, c = 420, 150, 210
    at = [(c + r * math.cos(math.radians(k * 45 - 90)), c + r * math.sin(math.radians(k * 45 - 90))) for k in range(8)]
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="{size}" height="{size}" '
        f'font-family="Libertinus Serif, Georgia, serif">',
        f'<circle cx="{c}" cy="{c}" r="{r + 40}" fill="{BOARD}"/>',
    ]
    # The star: each kewai a point, the ring between them drawn through
    # the hollows of the star as on the old boards.
    inner = 0.42 * r
    star = []
    for k in range(8):
        x, y = at[k]
        star.append(f"{x:.1f},{y:.1f}")
        a = math.radians(k * 45 - 90 + 22.5)
        star.append(f"{c + inner * math.cos(a):.1f},{c + inner * math.sin(a):.1f}")
    p.append(f'<polygon points="{" ".join(star)}" fill="none" stroke="{LINE}" stroke-width="2.5"/>')
    for x, y in at:
        p.append(f'<line x1="{c}" y1="{c}" x2="{x:.1f}" y2="{y:.1f}" stroke="{LINE}" stroke-width="1.2" stroke-dasharray="4 4"/>')
    for k, (x, y) in enumerate(at):
        fill = "#ffffff" if k < 4 else LINE
        p.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="15" fill="{fill}" stroke="{LINE}" stroke-width="2.5"/>')
        lx = c + (r + 30) * math.cos(math.radians(k * 45 - 90))
        ly = c + (r + 30) * math.sin(math.radians(k * 45 - 90))
        p.append(f'<text x="{lx:.1f}" y="{ly + 6:.1f}" font-size="17" text-anchor="middle" fill="{RED}">{k + 1}</text>')
    p.append(f'<circle cx="{c}" cy="{c}" r="15" fill="{BOARD}" stroke="{LINE}" stroke-width="2.5"/>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def mu_torere_text():
    return "\n".join(
        [
            "  8 --- 1 --- 2        B --- W --- W",
            "  | \\   |   / |        | \\   |   / |",
            "  7 --- P --- 3        B --- . --- W",
            "  | /   |   \\ |        | /   |   \\ |",
            "  6 --- 5 --- 4        B --- B --- W",
            "",
            "The board drawn square: the kewai 1 to 8 round the edge, each joined",
            "to its two neighbours and to the pūtahi P in the middle. Right: the",
            "start, White (W) on 1 to 4, Black (B) on 5 to 8, the pūtahi empty.",
        ]
    ) + "\n"


def konane_svg(rows=8, cols=8):
    cell, pad = 44, 40
    w, h = 2 * pad + (cols - 1) * cell, 2 * pad + (rows - 1) * cell
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="-24 -26 {w + 24} {h + 26}" width="{w + 24}" '
        f'height="{h + 26}" font-family="Libertinus Serif, Georgia, serif">',
        f'<rect x="0" y="0" width="{w}" height="{h}" rx="14" fill="#9a958e"/>',
    ]
    for r in range(rows):
        for c in range(cols):
            x, y = pad + c * cell, pad + r * cell
            black = (r + c) % 2 == 0
            fill = "#1d1b1a" if black else "#f6f0e4"
            p.append(f'<circle cx="{x}" cy="{y}" r="15" fill="{fill}" stroke="#3a3836" stroke-width="1.5"/>')
    for c in range(cols):
        p.append(f'<text x="{pad + c * cell}" y="-8" font-size="15" text-anchor="middle" fill="{LINE}">{"abcdefghijklmnopq"[c]}</text>')
    for r in range(rows):
        p.append(f'<text x="-12" y="{pad + r * cell + 5}" font-size="15" text-anchor="middle" fill="{LINE}">{r + 1}</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def konane_text(rows=8, cols=8):
    lines = ["     " + " ".join("abcdefghijklmnopq"[:cols])]
    for r in range(rows):
        lines.append(f"  {r + 1:>2} " + " ".join("X" if (r + c) % 2 == 0 else "O" for c in range(cols)))
    lines += ["", "X a black stone, O a white one: every hollow filled, a1 black."]
    return "\n".join(lines) + "\n"


def write(out: Path):
    (out / "mu-torere-board.svg").write_text(mu_torere_svg())
    (out / "mu-torere-board.txt").write_text(mu_torere_text())
    (out / "konane-board.svg").write_text(konane_svg())
    (out / "konane-board.txt").write_text(konane_text())
