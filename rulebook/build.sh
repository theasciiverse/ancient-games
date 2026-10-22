#!/bin/sh
# Builds the rulebook from chapters/*.md into out/:
#   ancient-games-rules.pdf    the whole book (pandoc, then Typst)
#   ancient-games-rules.html   the whole book, one self-contained page
#   <game>.html                one game's chapter, for its catalog page
#   <game>.txt                 one game's chapter as plain text, for RULES.txt
#                              in the release archives
# Needs pandoc 3.1.2 or newer, typst and python3.
set -eu

cd "$(dirname "$0")"
out=out
rm -rf "$out"
mkdir -p "$out"

python3 figures/make_figures.py

date=$(git log -1 --format=%cs -- . 2>/dev/null || true)
[ -n "$date" ] || date=$(date +%F)

common="--metadata-file=metadata.yaml --metadata=date:$date --citeproc --lua-filter=filters/no-dashes.lua"

intro=chapters/00-introduction.md
games=$(ls chapters/[0-9][0-9]-*.md | grep -v "^$intro\$")

# The whole book. Chapters are top-level headings; the bibliography
# collects at the end.
pandoc $common $intro $games --toc --standalone \
    -t typst -o "$out/ancient-games-rules.typ"
# Typst reads images relative to the project root given by --root.
sed -i 's|image("figures/|image("/figures/|' "$out/ancient-games-rules.typ"
typst compile --root . "$out/ancient-games-rules.typ" "$out/ancient-games-rules.pdf"
pandoc $common $intro $games --toc --standalone --embed-resources \
    --css=style/rulebook.css -o "$out/ancient-games-rules.html"

# One page and one text file per game: the chapter heading becomes the
# title, and the chapter gets its own references.
for chapter in $games; do
    game=$(basename "$chapter" .md | sed 's/^[0-9]*-//')
    title=$(sed -n 's/^# //p' "$chapter" | head -n 1)
    pandoc $common --shift-heading-level-by=-1 --metadata=title:"$title" \
        --metadata=subtitle:"The rules, and where they come from" \
        "$chapter" --standalone --embed-resources --css=style/rulebook.css \
        -o "$out/$game.html"
    pandoc $common --shift-heading-level-by=-1 --metadata=title:"$title" \
        --metadata=subtitle:"The rules, and where they come from" \
        --lua-filter=filters/text-figures.lua "$chapter" \
        --standalone -t plain --columns=76 -o "$out/$game.txt"
done

# No en or em dashes anywhere (the filter should have caught them all).
if grep -l "$(printf '\342\200\223\|\342\200\224')" "$out"/*.typ "$out"/*.html "$out"/*.txt; then
    echo "error: a dash slipped into the files above" >&2
    exit 1
fi

ls -l "$out"
