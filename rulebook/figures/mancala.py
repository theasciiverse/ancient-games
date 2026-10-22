"""Draws the mancala board figures (SVG and plain text) from the same
geometry as mancala/src/rules.rs and ui.rs: two rows of pits, South's along
the bottom named A, B, C ... in the order of sowing, North's along the top
named a, b, c ... the other way, so that pits facing each other stand one
above the other; South's store at the right, North's at the left. The
arrow shows the sowing, anticlockwise (a clockwise game is the mirror
image).

make_figures.py calls write() with its own folder.
"""

LINE = "#2b2b2b"
WOOD = "#e8d9bc"
PIT = "#f8f2e6"
ARROW = "#b5432a"
CELL = 70
PAD = 40


def svg(n, stores, label_store="store"):
    store_w = 64 if stores else 0
    gap = 14 if stores else 0
    w = 2 * PAD + n * CELL + 2 * (store_w + gap)
    h = 2 * PAD + 2 * CELL + 60
    x0 = PAD + store_w + gap
    y0 = PAD + 30
    p = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="{w}" '
        f'height="{h}" font-family="Libertinus Serif, Georgia, serif">',
        '<defs><marker id="head" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="7" '
        f'markerHeight="7" orient="auto-start-reverse"><path d="M0,0 L10,5 L0,10 z" fill="{ARROW}"/>'
        "</marker></defs>",
        f'<rect x="{PAD - 10}" y="{y0 - 14}" width="{w - 2 * PAD + 20}" height="{2 * CELL + 28}" '
        f'rx="30" fill="{WOOD}" stroke="{LINE}" stroke-width="2"/>',
    ]
    for row in range(2):
        for c in range(n):
            cx = x0 + c * CELL + CELL / 2
            cy = y0 + row * CELL + CELL / 2
            p.append(f'<circle cx="{cx}" cy="{cy}" r="{CELL / 2 - 7}" fill="{PIT}" stroke="{LINE}" stroke-width="2"/>')
            name = chr(ord("a") + n - 1 - c) if row == 0 else chr(ord("A") + c)
            ly = y0 - 22 if row == 0 else y0 + 2 * CELL + 34
            p.append(f'<text x="{cx}" y="{ly}" font-size="18" text-anchor="middle" fill="{LINE}">{name}</text>')
    if stores:
        for side, x in (("North", PAD), ("South", x0 + n * CELL + gap)):
            p.append(
                f'<rect x="{x}" y="{y0 + 4}" width="{store_w}" height="{2 * CELL - 8}" rx="28" '
                f'fill="{PIT}" stroke="{LINE}" stroke-width="2"/>'
            )
            p.append(
                f'<text x="{x + store_w / 2}" y="{y0 + CELL - 2}" font-size="14" text-anchor="middle" '
                f'fill="{LINE}">{side}</text>'
            )
            p.append(
                f'<text x="{x + store_w / 2}" y="{y0 + CELL + 16}" font-size="14" text-anchor="middle" '
                f'fill="{LINE}">{label_store}</text>'
            )
    # The sowing: along South's row to the right, up, back along North's.
    mid = y0 + CELL
    left = x0 + CELL * 0.3
    right = x0 + n * CELL - CELL * 0.3
    p.append(
        f'<path d="M {left} {mid + 10} L {right} {mid + 10} Q {right + 18} {mid} {right} {mid - 10} '
        f'L {left} {mid - 10}" fill="none" stroke="{ARROW}" stroke-width="2.5" marker-end="url(#head)"/>'
    )
    p.append(f'<text x="{w / 2}" y="{h - 12}" font-size="16" text-anchor="middle" fill="{LINE}">South</text>')
    p.append(f'<text x="{w / 2}" y="{PAD - 18}" font-size="16" text-anchor="middle" fill="{LINE}">North</text>')
    p.append("</svg>")
    return "\n".join(p) + "\n"


def text(n, stores, note):
    cell = "+----"
    rule = "     " + cell * n + "+"
    top = "     " + "".join(f"  {chr(ord('a') + n - 1 - c)}  " for c in range(n))
    bottom = "     " + "".join(f"  {chr(ord('A') + c)}  " for c in range(n))
    row = "|    " * n + "|"
    lines = ["North".center(len(rule)), top.rstrip(), rule]
    if stores:
        lines.append(" (N) " + row + " (S)")
    else:
        lines.append("     " + row)
    lines += [rule, "     " + row, rule, bottom.rstrip(), "South".center(len(rule)), "", note]
    return "\n".join(line.rstrip() for line in lines) + "\n"


def write(here):
    (here / "mancala-board.svg").write_text(svg(6, True))
    (here / "mancala-board.txt").write_text(
        text(
            6,
            True,
            "Sowing runs A to F, then a to f, and round again. (N) and (S): the\n"
            "stores, in the games that have them. a faces F, and A faces f.",
        )
    )
    (here / "toguz-board.svg").write_text(svg(9, True, "kazan"))
    (here / "toguz-board.txt").write_text(
        text(9, True, "Sowing runs A to I, then a to i. (N) and (S): the kazans, the stores.")
    )
