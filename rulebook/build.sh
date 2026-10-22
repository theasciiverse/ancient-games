#!/bin/sh
# Builds the rulebook from chapters/*.md into out/:
#   ancient-games-rules.pdf    the whole book (pandoc, then Typst)
#   ancient-games-rules.html   the whole book, one self-contained page
#   <game>.html                one game's chapter, for its catalog page
#   <game>.txt                 one game's chapter as plain text, also copied
#                              to the game's crate as RULES.txt (part of its
#                              release archive)
#   <game>-help.txt            the chapter's short form (its "brief" blocks),
#                              copied to the crate as HELP.txt, the game's F1
# Needs pandoc 3.1.2 or newer, typst, python3 and cargo.
#
# RULEBOOK_OUT=dir builds into another folder and leaves the crates'
# RULES.txt and HELP.txt alone, so builds can run side by side.
set -eu

cd "$(dirname "$0")"
out=${RULEBOOK_OUT:-out}
rm -rf "$out"
mkdir -p "$out"

python3 figures/make_figures.py

# A game shows its chapter's short form under F1 (HELP.txt) and ships the
# whole chapter as RULES.txt. Every game's chapter must have a short form:
# fenced divs of class "brief", one at least under each rule set's
# heading. A new chapter's crate gets empty files to compile with; the
# real text replaces them below.
for chapter in chapters/[0-9][0-9]-*.md; do
    crate=$(sed -n '1,/^---$/{s/^crate: *//p}' "$chapter" | head -n 1)
    [ -n "$crate" ] || continue
    if ! grep -q '^::: *{\{0,1\}\.\{0,1\}brief' "$chapter"; then
        echo "error: $chapter has no short form for the game's F1 (::: brief blocks)" >&2
        exit 1
    fi
    for f in RULES.txt HELP.txt; do
        [ -f "../$crate/$f" ] || : > "../$crate/$f"
    done
done

# Worked examples and screens from the games themselves (pictures.txt).
cargo build --release --quiet --manifest-path ../Cargo.toml --workspace
bin=../target/release
grep -v '^#' pictures.txt | while read -r kind package name rules seed seconds; do
    case "$kind" in
        examples)
            rm -rf "examples/$package" && mkdir -p "examples/$package"
            "$bin/$package" --rulebook-examples | awk -v dir="examples/$package" '
                /^=== / { file = dir "/" substr($0, 5) ".txt"; next }
                { print > file }'
            ;;
        screen)
            mkdir -p "screens/$package"
            "$bin/$package" --seed "$seed" --rulebook-screen "$rules:$seconds" \
                > "screens/$package/$name.svg"
            ;;
    esac
done

date=$(git log -1 --format=%cs -- . 2>/dev/null || true)
[ -n "$date" ] || date=$(date +%F)

# references.bib holds the shared sources, bib/<game>.bib each chapter's own.
bibs="--bibliography=references.bib"
for bib in bib/*.bib; do
    [ -f "$bib" ] && bibs="$bibs --bibliography=$bib"
done

common="--metadata-file=metadata.yaml --metadata=date:$date $bibs --citeproc --lua-filter=filters/examples.lua --lua-filter=filters/glance.lua --lua-filter=filters/no-dashes.lua"
# The book, its pages and RULES.txt leave out the short form for F1.
full="$common --lua-filter=filters/no-brief.lua"

intro=chapters/00-introduction.md
games=$(ls chapters/[0-9][0-9]-*.md | grep -v "^$intro\$")

# The whole book. Chapters are top-level headings; the bibliography
# collects at the end.
pandoc $full $intro $games --toc --standalone \
    -t typst -o "$out/ancient-games-rules.typ"
# Typst reads images relative to the file and only inside its root, so
# compile a copy that sits here, beside figures/ and images/.
tmp=".book-$$.typ"
cp "$out/ancient-games-rules.typ" "$tmp"
typst compile --root . "$tmp" "$out/ancient-games-rules.pdf" || { rm -f "$tmp"; exit 1; }
rm -f "$tmp"
pandoc $full $intro $games --toc --standalone --embed-resources \
    --css=style/rulebook.css -o "$out/ancient-games-rules.html"

# One page and one text file per game: the chapter heading becomes the
# title, and the chapter gets its own references.
for chapter in $games; do
    game=$(basename "$chapter" .md | sed 's/^[0-9]*-//')
    # The first heading after the front matter.
    title=$(awk 'NR == 1 && /^---$/ { yaml = 1; next }
                 yaml && /^---$/ { yaml = 0; next }
                 !yaml && /^# / { sub(/^# /, ""); print; exit }' "$chapter")
    # The page keeps the chapter's own heading as its title, styled as in
    # the book, so no title block above it.
    pandoc $full --metadata=title: --metadata=pagetitle:"$title" \
        "$chapter" --standalone --embed-resources --css=style/rulebook.css \
        -o "$out/$game.html"
    pandoc $full --shift-heading-level-by=-1 --metadata=title:"$title" \
        --metadata=subtitle:"The rules, and where they come from" \
        --lua-filter=filters/text-figures.lua "$chapter" \
        --standalone -t plain --columns=76 -o "$out/$game.txt"
    # The game's help and the RULES.txt of its release archive.
    crate=$(sed -n '1,/^---$/{s/^crate: *//p}' "$chapter" | head -n 1)
    if [ -n "$crate" ] && [ -z "${RULEBOOK_OUT:-}" ]; then
        cp "$out/$game.txt" "../$crate/RULES.txt"
    fi
    # The short form for the game's F1: the "brief" blocks under their
    # section headings (filters/help.lua).
    if [ -n "$crate" ]; then
        pandoc $common --lua-filter=filters/help.lua --metadata=title:"$title" \
            "$chapter" --standalone -t plain --columns=76 -o "$out/$game-help.txt"
        if [ -z "${RULEBOOK_OUT:-}" ]; then
            cp "$out/$game-help.txt" "../$crate/HELP.txt"
        fi
    fi
done

# No en or em dashes anywhere (the filter should have caught them all).
if grep -l "$(printf '\342\200\223\|\342\200\224')" "$out"/*.typ "$out"/*.html "$out"/*.txt; then
    echo "error: a dash slipped into the files above" >&2
    exit 1
fi

ls -l "$out"
