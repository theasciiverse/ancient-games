#!/bin/sh
# Records a game's catalog preview (hub/RECORDING.md): its own --demo, the
# computer playing both sides, at 100x30, headless.
#
# Usage: asciiverse/record-preview.sh <binary> <seed> <seconds> <title> [out.cast]
#
# The program runs by name from a throwaway folder, with a throwaway home
# and a clean environment, so nothing of this machine ends up in the cast.
# Needs asciinema 3.
set -eu

binary=$1
seed=$2
seconds=$3
title=$4
name=$(basename "$binary")
out=${5:-$name.cast}

asciinema=$(command -v asciinema)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/home"
cp "$binary" "$work/bin/$name"

env -i \
  HOME="$work/home" PATH="$work/bin:/usr/bin:/bin" \
  TERM=xterm-256color COLORTERM=truecolor LANG=C.UTF-8 \
  USER=player LOGNAME=player \
  "$asciinema" rec --headless --overwrite \
    --window-size 100x30 \
    --idle-time-limit 2 \
    --capture-env "TERM,COLORTERM,LANG" \
    --title "$title" \
    --command "$name --demo --seed $seed --quit-after $seconds" \
    "$out"
