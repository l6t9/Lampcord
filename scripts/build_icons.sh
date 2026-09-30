#!/usr/bin/env bash

set -euo pipefail

cd "$(dirname "$0")/.."

SOURCE="desktopApp/src/jvmMain/resources/logo.png"
OUTPUT="desktopApp/src/jvmMain/resources/logo.icns"

if ! command -v iconutil > /dev/null 2>&1 || ! command -v sips > /dev/null 2>&1; then
  echo "sips and iconutil are required to build logo.icns (macOS only)." >&2
  exit 1
fi

if [[ ! -f "$SOURCE" ]]; then
  echo "$SOURCE is missing" >&2
  exit 1
fi

iconset="$(mktemp -d)/AppIcon.iconset"
mkdir -p "$iconset"
trap 'rm -rf "$(dirname "$iconset")"' EXIT

for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$SOURCE" --out "$iconset/icon_${size}x${size}.png" > /dev/null
  sips -z $((size * 2)) $((size * 2)) "$SOURCE" \
    --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
done

iconutil -c icns "$iconset" -o "$OUTPUT"
echo "Wrote $OUTPUT"