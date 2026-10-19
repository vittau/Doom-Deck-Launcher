#!/usr/bin/env bash
# Convert an image to the DDL bundled-image rule (spec 035): JPEG, at most 1280x800, quality 60.
#
#   scripts/lean-image.sh <input> <output.jpg> [quality]
#
# Uses macOS `sips`; falls back to ImageMagick (`magick` or `convert`) on Linux. Images are only
# ever shrunk, never enlarged, and the aspect ratio is kept. Do not use this for art that needs
# transparency: it flattens alpha. Those files stay PNG (see CONTRIBUTING.md).
set -euo pipefail

MAX_W=1280
MAX_H=800
DEFAULT_QUALITY=60

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "usage: $0 <input> <output.jpg> [quality=$DEFAULT_QUALITY]" >&2
    exit 2
fi
in=$1
out=$2
quality=${3:-$DEFAULT_QUALITY}

[ -f "$in" ] || { echo "error: no such file: $in" >&2; exit 1; }
case "$out" in
    *.jpg) ;;
    *) echo "error: output must end in .jpg: $out" >&2; exit 2 ;;
esac
case "$quality" in
    '' | *[!0-9]*) echo "error: quality must be a number, got: $quality" >&2; exit 2 ;;
esac

if command -v sips >/dev/null 2>&1; then
    width=$(sips -g pixelWidth "$in" | awk '/pixelWidth/ { print $2 }')
    height=$(sips -g pixelHeight "$in" | awk '/pixelHeight/ { print $2 }')
    args=(-s format jpeg -s formatOptions "$quality" --deleteColorManagementProperties)
    # Fit inside MAX_W x MAX_H: shrink along whichever side is the tighter limit.
    if [ "$width" -gt "$MAX_W" ] || [ "$height" -gt "$MAX_H" ]; then
        if [ $((width * MAX_H)) -ge $((height * MAX_W)) ]; then
            args+=(--resampleWidth "$MAX_W")
        else
            args+=(--resampleHeight "$MAX_H")
        fi
    fi
    sips "${args[@]}" "$in" --out "$out" >/dev/null
elif command -v magick >/dev/null 2>&1 || command -v convert >/dev/null 2>&1; then
    im=magick
    command -v magick >/dev/null 2>&1 || im=convert
    "$im" "$in" -background black -alpha remove -resize "${MAX_W}x${MAX_H}>" \
        -strip -sampling-factor 4:2:0 -interlace none -quality "$quality" "$out"
else
    echo "error: need macOS sips or ImageMagick (magick/convert)" >&2
    exit 1
fi

echo "$out: $(wc -c <"$out" | tr -d ' ') bytes"
