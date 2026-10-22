#!/bin/sh
# Builds the rulebook from chapters/*.md into out/:
#   ancient-games-rules.pdf    the whole book (pandoc, then Typst)
#   ancient-games-rules.html   the whole book, one self-contained page
#   <game>.html                one game's chapter, for its catalog page
#   <game>.txt                 one game's chapter as plain text, also copied
#                              to the game's crate as RULES.txt (its F1 help,
#                              and part of its release archive)
# Needs pandoc 3.1.2 or newer, typst and python3.
#
# RULEBOOK_OUT=dir builds into another folder and leaves the crates'
# RULES.txt alone, so builds can run side by side.
set -eu

cd "$(dirname "$0")"
out=${RULEBOOK_OUT:-out}
rm -rf "$out"
mkdir -p "$out"

python3 figures/make_figures.py

date=$(git log -1 --format=%cs -- . 2>/dev/null || true)
[ -n "$date" ] || date=$(date +%F)

# references.bib holds the shared sources, bib/<game>.bib each chapter's own.
bibs="--bibliography=references.bib"
for bib in bib/*.bib; do
    [ -f "$bib" ] && bibs="$bibs --bibliography=$bib"
done

common="--metadata-file=metadata.yaml --metadata=date:$date $bibs --citeproc --lua-filter=filters/no-dashes.lua"

intro=chapters/00-introduction.md
games=$(ls chapters/[0-9][0-9]-*.md | grep -v "^$intro\$")

# The whole book. Chapters are top-level headings; the bibliography
# collects at the end.
pandoc $common $intro $games --toc --standalone \
    -t typst -o "$out/ancient-games-rules.typ"
# Typst reads images relative to the file and only inside its root, so
# compile a copy that sits here, beside figures/ and images/.
tmp=".book-$$.typ"
cp "$out/ancient-games-rules.typ" "$tmp"
typst compile --root . "$tmp" "$out/ancient-games-rules.pdf" || { rm -f "$tmp"; exit 1; }
rm -f "$tmp"
pandoc $common $intro $games --toc --standalone --embed-resources \
    --css=style/rulebook.css -o "$out/ancient-games-rules.html"

# One page and one text file per game: the chapter heading becomes the
# title, and the chapter gets its own references.
for chapter in $games; do
    game=$(basename "$chapter" .md | sed 's/^[0-9]*-//')
    # The first heading after the front matter.
    title=$(awk 'NR == 1 && /^---$/ { yaml = 1; next }
                 yaml && /^---$/ { yaml = 0; next }
                 !yaml && /^# / { sub(/^# /, ""); print; exit }' "$chapter")
    pandoc $common --shift-heading-level-by=-1 --metadata=title:"$title" \
        --metadata=subtitle:"The rules, and where they come from" \
        "$chapter" --standalone --embed-resources --css=style/rulebook.css \
        -o "$out/$game.html"
    pandoc $common --shift-heading-level-by=-1 --metadata=title:"$title" \
        --metadata=subtitle:"The rules, and where they come from" \
        --lua-filter=filters/text-figures.lua "$chapter" \
        --standalone -t plain --columns=76 -o "$out/$game.txt"
    # The game's help and the RULES.txt of its release archive.
    crate=$(sed -n '1,/^---$/{s/^crate: *//p}' "$chapter" | head -n 1)
    if [ -n "$crate" ] && [ -z "${RULEBOOK_OUT:-}" ]; then
        cp "$out/$game.txt" "../$crate/RULES.txt"
    fi
done

# No en or em dashes anywhere (the filter should have caught them all).
if grep -l "$(printf '\342\200\223\|\342\200\224')" "$out"/*.typ "$out"/*.html "$out"/*.txt; then
    echo "error: a dash slipped into the files above" >&2
    exit 1
fi

ls -l "$out"
