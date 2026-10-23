"""Draws the morris boards (SVG and plain text) from the same geometry as
morris/src/rules.rs: points on a square grid, row 0 at the top, joined by
lines; every line of three points is a mill. Points are named as players
of Nine Men's Morris write them: the column as a letter from the left, the
row as a number from the bottom (a1 is the bottom left corner).

make_figures.py calls write() with its own folder.
"""

from pathlib import Path

LINE = "#2b2b2b"
BOARD = "#f4ecdc"
CELL = 56
PAD = 44


def squares(size, count):
    lines = []
    for i in range(count):
        lo, hi, mid = i, size - i, size // 2
        lines += [
            [(lo, lo), (lo, mid), (lo, hi)],
            [(hi, lo), (hi, mid), (hi, hi)],
            [(lo, lo), (mid, lo), (hi, lo)],
            [(lo, hi), (mid, hi), (hi, hi)],
        ]
    return lines


def three(diagonals):
    lines = [[(i, c) for c in range(3)] for i in range(3)] + [[(r, i) for r in range(3)] for i in range(3)]
    if diagonals:
        lines += [[(0, 0), (1, 1), (2, 2)], [(0, 2), (1, 1), (2, 0)]]
    return 3, lines


def six():
    lines = squares(4, 2) + [[(0, 2), (1, 2)], [(2, 0), (2, 1)], [(2, 3), (2, 4)], [(3, 2), (4, 2)]]
    return 5, lines


def nine(twelve=False):
    lines = squares(6, 3) + [
        [(0, 3), (1, 3), (2, 3)],
        [(3, 0), (3, 1), (3, 2)],
        [(3, 4), (3, 5), (3, 6)],
        [(4, 3), (5, 3), (6, 3)],
    ]
    if twelve:
        lines += [
            [(0, 0), (1, 1), (2, 2)],
            [(0, 6), (1, 5), (2, 4)],
            [(4, 2), (5, 1), (6, 0)],
            [(4, 4), (5, 5), (6, 6)],
        ]
    return 7, lines


BOARDS = [
    ("morris-three", "Three Men's Morris", three(True)),
    ("morris-three-plain", "Three Men's Morris, without diagonals", three(False)),
    ("morris-six", "Six Men's Morris", six()),
    ("morris-nine", "Nine Men's Morris", nine()),
    ("morris-twelve", "Twelve Men's Morris", nine(True)),
]


def points(lines):
    return sorted({p for line in lines for p in line})


def svg(size, lines):
    span = (size - 1) * CELL
    w = h = 2 * PAD + span
    at = lambda r, c: (PAD + c * CELL, PAD + r * CELL)
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" height="{h}" '
        f'font-family="Libertinus Serif, Georgia, serif">',
        f'<rect x="{PAD / 3}" y="{PAD / 3}" width="{w - 2 * PAD / 3}" height="{h - 2 * PAD / 3}" '
        f'fill="{BOARD}"/>',
    ]
    for line in lines:
        for a, b in zip(line, line[1:]):
            (x1, y1), (x2, y2) = at(*a), at(*b)
            out.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{LINE}" stroke-width="2.5"/>')
    for r, c in points(lines):
        x, y = at(r, c)
        out.append(f'<circle cx="{x}" cy="{y}" r="6.5" fill="{LINE}"/>')
    for c in range(size):
        x, _ = at(0, c)
        out.append(
            f'<text x="{x}" y="{h - PAD / 3 + 18}" font-size="15" text-anchor="middle" fill="{LINE}">'
            f'{"abcdefg"[c]}</text>'
        )
    for r in range(size):
        _, y = at(r, 0)
        out.append(
            f'<text x="{PAD / 3 - 12}" y="{y + 5}" font-size="15" text-anchor="middle" fill="{LINE}">'
            f'{size - r}</text>'
        )
    out[0] = out[0].replace(f'viewBox="0 0 {w} {h}"', f'viewBox="-20 0 {w + 20} {h + 24}"').replace(
        f'width="{w}" height="{h}"', f'width="{w + 20}" height="{h + 24}"'
    )
    out.append("</svg>")
    return "\n".join(out) + "\n"


def text(size, lines):
    wide, tall = 4 * (size - 1) + 1, 2 * (size - 1) + 1
    grid = [[" "] * wide for _ in range(tall)]
    for line in lines:
        for (r0, c0), (r1, c1) in zip(line, line[1:]):
            y0, x0, y1, x1 = 2 * r0, 4 * c0, 2 * r1, 4 * c1
            dy, dx = (y1 > y0) - (y1 < y0), (x1 > x0) - (x1 < x0)
            if dy and dx:
                glyph = "\\" if dy == dx else "/"
                for j in range(abs(y1 - y0) // 2):
                    grid[y0 + dy * (2 * j + 1)][x0 + dx * (4 * j + 2)] = glyph
            else:
                glyph = "-" if dy == 0 else "|"
                for i in range(1, max(abs(y1 - y0), abs(x1 - x0))):
                    grid[y0 + dy * i][x0 + dx * i] = glyph
    for r, c in points(lines):
        grid[2 * r][4 * c] = "+"
    rows = []
    for y, row in enumerate(grid):
        number = str(size - y // 2) if y % 2 == 0 else ""
        rows.append(f"  {number:>1}  {''.join(row)}".rstrip())
    rows.append("     " + "   ".join("abcdefg"[:size]))
    return "\n".join(rows) + "\n"


def write(out: Path):
    for name, _title, (size, lines) in BOARDS:
        (out / f"{name}.svg").write_text(svg(size, lines))
        (out / f"{name}.txt").write_text(text(size, lines) + "\n+ a point. Neighbours on a line are joined.\n")
