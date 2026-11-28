#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${REQUIEM_BINARY:-$repo_root/rust/target/release/requiem}"
out="${APPIMAGE_OUT:-$repo_root/dist/Lampcord-Requiem.AppImage}"
appimagetool="${APPIMAGETOOL:-/usr/bin/appimagetool}"
arch="${ARCH:-x86_64}"
libs="${REQUIEM_LIBS:-}"

usage() {
  cat <<'EOF'
usage: scripts/package_appimage.sh [options]

  --binary PATH      Requiem release binary (default: rust/target/release/requiem)
  --out PATH         AppImage output (default: dist/Lampcord-Requiem.AppImage)
  --appimagetool PATH
                     appimagetool executable (default: /usr/bin/appimagetool)
  --arch NAME        AppImage architecture (default: x86_64)
  --libs DIR         Shared libraries (e.g. FFmpeg .so files) to bundle in usr/lib
  -h, --help         Show this help
EOF
}

while (($# > 0)); do
  case "$1" in
    --binary) binary="$2"; shift 2 ;;
    --out) out="$2"; shift 2 ;;
    --appimagetool) appimagetool="$2"; shift 2 ;;
    --arch) arch="$2"; shift 2 ;;
    --libs) libs="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -x "$binary" ]] || { echo "Requiem binary not executable: $binary" >&2; exit 1; }
[[ -x "$appimagetool" ]] || { echo "appimagetool not executable: $appimagetool" >&2; exit 1; }
[[ -f "$repo_root/packaging/aur/lampcord.png" ]] || { echo "missing Lampcord icon" >&2; exit 1; }

appdir="$(mktemp -d "${TMPDIR:-/tmp}/lampcord-appimage.XXXXXX")"
trap 'rm -rf "$appdir"' EXIT

mkdir -p \
  "$appdir/usr/bin" \
  "$appdir/usr/share/applications" \
  "$appdir/usr/share/icons/hicolor/512x512/apps" \
  "$appdir/usr/share/icons/hicolor/scalable/apps"

install -m 755 "$binary" "$appdir/usr/bin/requiem"
ln -s requiem "$appdir/usr/bin/lampcord"

if [[ -n "$libs" ]]; then
  [[ -d "$libs" ]] || { echo "libs directory not found: $libs" >&2; exit 1; }
  mkdir -p "$appdir/usr/lib"
  cp -L "$libs"/*.so* "$appdir/usr/lib/"
fi

install -m 644 "$repo_root/packaging/aur/lampcord.png" \
  "$appdir/usr/share/icons/hicolor/512x512/apps/lampcord.png"
cp "$repo_root/packaging/aur/lampcord.png" "$appdir/lampcord.png"
if [[ -f "$repo_root/packaging/aur/lampcord.svg" ]]; then
  install -m 644 "$repo_root/packaging/aur/lampcord.svg" \
    "$appdir/usr/share/icons/hicolor/scalable/apps/lampcord.svg"
fi

cat > "$appdir/usr/share/applications/lampcord.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Lampcord
GenericName=Discord Client
Comment=Requiem Rust/Skia Discord client
Exec=AppRun %U
Icon=lampcord
Terminal=false
Categories=Network;Chat;
StartupWMClass=requiem
DESKTOP
cp "$appdir/usr/share/applications/lampcord.desktop" "$appdir/lampcord.desktop"

cat > "$appdir/AppRun" <<'APPRUN'
#!/bin/sh
set -e
APPDIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
export LD_LIBRARY_PATH="$APPDIR/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$APPDIR/usr/bin/requiem" "$@"
APPRUN
chmod 755 "$appdir/AppRun"

mkdir -p "$(dirname "$out")"
rm -f "$out"
ARCH="$arch" "$appimagetool" "$appdir" "$out"

echo "built $out"
