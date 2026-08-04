#!/usr/bin/env python3
"""
generate_material_symbols.py — fetch EXACT Material Symbols icons from Google Fonts
and emit Kotlin ImageVector definitions for Lampcord.

The icons on https://fonts.google.com/icons are Material Symbols, distributed as
variable fonts (960-unit design grid) in Google's material-design-icons repository.
This script downloads those fonts (cached in scripts/.fonts/), converts glyph
outlines into the same `materialSymbol(name, pathData)` format used by the generated
files in shared/.../ui/icons/, and can verify existing files against the font.

The path data uses the font's native 960-unit grid with the Y axis flipped to
Compose's top-down coordinate system, exactly like the existing generated icons
(the `materialSymbol` helper in MaterialSymbol.kt applies translationY=960f).

The "Filled" style is the Outlined variable font with the FILL axis baked in
(FILL=1). To keep instancing fast, the font is subset to the needed glyphs
before the FILL axis is instantiated.

Usage
-----
Regenerate every icon in a generated file from the current fonts (the robust way
to refresh all icons at once):

    python3 scripts/generate_material_symbols.py regen

Add one or more icons (exact Google Fonts glyphs):

    python3 scripts/generate_material_symbols.py add LightMode DarkMode --style filled

Verify every icon in the generated files matches the font:

    python3 scripts/generate_material_symbols.py verify

Bake the FILL axis explicitly (0 or 1) for any style:

    python3 scripts/generate_material_symbols.py regen --style filled --fill 1
"""

from __future__ import annotations

import argparse
import re
import sys
import urllib.parse
import urllib.request
from pathlib import Path

try:
    from fontTools.pens.basePen import BasePen
    from fontTools.ttLib import TTFont
except ImportError:  # pragma: no cover
    sys.exit("This script needs fonttools. Install it with: pip install fonttools")

ROOT = Path(__file__).resolve().parent.parent
ICON_DIR = ROOT / "shared/src/commonMain/kotlin/me/lampu/lampcord/shared/ui/icons"
FONT_DIR = Path(__file__).resolve().parent / ".fonts"

REPO = "https://github.com/google/material-design-icons/raw/master/variablefont"

# The Material Symbols variable fonts served on fonts.google.com/icons. The repo
# ships no separate "Filled" font file; "filled" is the Outlined font with the
# FILL axis baked in (see DEFAULT_FILL).
STYLE_FONTS = {
    "filled": "MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf",
    "outlined": "MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf",
    "rounded": "MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf",
    "sharp": "MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf",
}

OUTPUT_FILES = {
    "filled": "IconsFilled.kt",
    "outlined": "IconsOutlined.kt",
    "rounded": "IconsRounded.kt",
    "sharp": None,
}

OBJECT_NAMES = {
    "filled": "IconsFilled",
    "outlined": "IconsOutlined",
    "rounded": "IconsRounded",
    "sharp": "IconsSharp",
}

# The "Filled" style means the FILL axis baked in. Outlined/Rounded/Sharp use the
# default FILL=0 instance.
DEFAULT_FILL = {"filled": 1.0}

# Classic Material Icons names that no longer exist as glyphs in the current
# Material Symbols font, mapped to the current equivalent glyph.
LEGACY_GLYPHS = {
    "Clear": "close",                # X shape -> current "close"
    "FavoriteBorder": "favorite",    # border heart -> FILL=0 "favorite"
    "NewReleases": "release_alert",  # burst badge + checkmark
    "NotificationsNone": "notifications",
    "PeopleOutline": "group",
}

# Icons Google marks as auto-mirrored (they flip in RTL layouts). The AutoMirrored
# facade objects in LampcordIcons.kt are hand-maintained; this list only drives a
# reminder when you add one of these.
AUTO_MIRRORED = {
    "arrow_back", "arrow_back_ios", "arrow_back_ios_new", "arrow_forward",
    "arrow_forward_ios", "arrow_left", "arrow_right", "arrow_right_alt",
    "arrow_upward", "arrow_downward", "backspace", "chevron_left", "chevron_right",
    "format_list_bulleted", "keyboard_arrow_left", "keyboard_arrow_right",
    "keyboard_backspace", "keyboard_return", "list", "login", "logout", "menu_open",
    "navigate_before", "navigate_next", "open_in_new", "playlist_add",
    "playlist_add_check", "queue_music", "reply", "send", "start", "trending_up",
    "undo", "volume_down", "volume_mute", "volume_off", "volume_up",
}


def icon_to_glyph(name: str) -> str:
    """AccountCircle -> account_circle ; BarChart4Bars -> bar_chart_4_bars."""
    s = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", name)
    s = re.sub(r"(?<=[A-Z])(?=[A-Z][a-z])", "_", s)
    s = re.sub(r"(?<=[a-z])(?=[0-9])", "_", s)
    return s.lower()


def glyph_for(name: str) -> str:
    return LEGACY_GLYPHS.get(name, icon_to_glyph(name))


class CompactPen(BasePen):
    """BasePen emitting absolute path data in the compact 960-grid format used by the
    generated icon files (Y flipped: font space is Y-up, Compose is Y-down)."""

    def __init__(self, glyphSet):
        super().__init__(glyphSet)
        self.commands: list[str] = []
        self._pos = (0.0, 0.0)

    @staticmethod
    def _num(value: float) -> str:
        text = f"{value:.3f}".rstrip("0").rstrip(".")
        return text if text not in ("", "-0") else "0"

    def _moveTo(self, pt):
        self.commands.append(f"M{self._num(pt[0])}{self._num(-pt[1])}")
        self._pos = pt

    def _lineTo(self, pt):
        x, y = pt
        px, py = self._pos
        if x == px:
            self.commands.append(f"V{self._num(-y)}")
        elif y == py:
            self.commands.append(f"H{self._num(x)}")
        else:
            self.commands.append(f"L{self._num(x)}{self._num(-y)}")
        self._pos = pt

    def _curveToOne(self, p1, p2, p3):
        self.commands.append(
            f"C{self._num(p1[0])}{self._num(-p1[1])} {self._num(p2[0])}{self._num(-p2[1])} "
            f"{self._num(p3[0])}{self._num(-p3[1])}"
        )
        self._pos = p3

    def _qCurveToOne(self, p1, p2):
        self.commands.append(
            f"Q{self._num(p1[0])}{self._num(-p1[1])} {self._num(p2[0])}{self._num(-p2[1])}"
        )
        self._pos = p2

    def _closePath(self):
        self.commands.append("Z")

    def path(self) -> str:
        return "".join(self.commands)


_font_cache: dict[tuple[str, float | None], TTFont] = {}


def get_font(style: str, fill: float | None, glyphs: set[str] | None = None) -> TTFont:
    """Download (once, cached) and load the variable font for a style.

    When `fill` is set, the font is first subset to `glyphs` (fast) and then the
    FILL axis is baked in. Instancing the full-size variable font is very slow, so
    callers should always pass the set of glyphs they need.
    """
    key = (style, fill)
    if key in _font_cache:
        return _font_cache[key]
    FONT_DIR.mkdir(parents=True, exist_ok=True)
    filename = STYLE_FONTS[style]
    path = FONT_DIR / filename
    if not path.exists():
        url = f"{REPO}/{urllib.parse.quote(filename)}"
        print(f"[*] Downloading {filename} ...")
        request = urllib.request.Request(url, headers={"User-Agent": "lampcord-icon-generator"})
        with urllib.request.urlopen(request, timeout=180) as response, open(path, "wb") as out:
            out.write(response.read())
        if path.read_bytes()[:4] != b"\x00\x01\x00\x00":
            sys.exit(f"Downloaded file is not a valid font: {path}")
    font = TTFont(path)
    if fill:
        from fontTools.subset import Options, Subsetter
        from fontTools.varLib.instancer import instantiateVariableFont

        if glyphs:
            available = set(font.getGlyphOrder())
            wanted = sorted(glyphs & available)
            if wanted:
                opt = Options()
                opt.notdef_outline = True
                opt.recalc_bounds = True
                subsetter = Subsetter(options=opt)
                subsetter.populate(glyphs=wanted)
                subsetter.subset(font)
        font = instantiateVariableFont(font, {"FILL": float(fill)}, inplace=True)
    _font_cache[key] = font
    return font


def extract_glyph(font: TTFont, glyph: str) -> str:
    pen = CompactPen(font.getGlyphSet())
    font.getGlyphSet()[glyph].draw(pen)
    return pen.path()


# ---------------------------------------------------------------------------
# Existing-file parsing / writing
# ---------------------------------------------------------------------------

_ENTRY_RE = re.compile(
    r"val\s+(\w+):\s*ImageVector\s+by\s+lazy\s*\{.*?pathData\s*=\s*\"([^\"]+)\"",
    re.S,
)


def parse_existing(path: Path) -> dict[str, str]:
    """Return {iconName: pathData} for a generated LampcordIcons*.kt file."""
    if not path.exists():
        return {}
    text = path.read_text()
    return dict(_ENTRY_RE.findall(text))


def write_file(style: str, entries: dict[str, str]) -> None:
    out_path = ICON_DIR / OUTPUT_FILES[style]
    obj = OBJECT_NAMES[style]
    lines = [
        '@file:Suppress("ktlint:standard:max-line-length")',
        "",
        "package me.lampu.lampcord.shared.ui.icons",
        "",
        "import androidx.compose.ui.graphics.vector.ImageVector",
        "",
        "// Generated by scripts/generate_material_symbols.py from Google Material Symbols.",
        f"object {obj} {{",
    ]
    for name, path_data in entries.items():
        lines.append(f"    val {name}: ImageVector by lazy {{")
        lines.append("        materialSymbol(")
        lines.append(f'            name = "{style.capitalize()}.{name}",')
        lines.append(f'            pathData = "{path_data}",')
        lines.append("        )")
        lines.append("    }")
        lines.append("")
    lines.append("}")
    out_path.write_text("\n".join(lines) + "\n")
    print(f"[*] Wrote {out_path.relative_to(ROOT)} ({len(entries)} icons)")


# ---------------------------------------------------------------------------
# Geometry comparison
# ---------------------------------------------------------------------------

_TOKEN_RE = re.compile(r"[a-zA-Z]|-?\d*\.?\d+(?:[eE][+-]?\d+)?")


def _fmt(v: float) -> str:
    return f"{v:.3f}".rstrip("0").rstrip(".")


def normalize_path(path: str) -> str:
    """Expand SVG path data to absolute M/L/Q/C/Z only, for geometry comparison.

    Handles implicit repeated coordinate groups (e.g. "l 1 2 3 4" is two lineto
    segments) and drops a redundant explicit closing segment that returns exactly
    to the start point (some generated files emit one before "Z").
    """
    tokens = _TOKEN_RE.findall(path)
    out: list[str] = []
    i = 0
    cur = (0.0, 0.0)
    start = (0.0, 0.0)
    last_ctrl: tuple[float, float] | None = None  # previous Q/C control point

    def nums(n: int) -> list[float]:
        nonlocal i
        vals = []
        for _ in range(n):
            vals.append(float(tokens[i]))
            i += 1
        return vals

    def is_num() -> bool:
        return i < len(tokens) and not tokens[i][:1].isalpha()

    while i < len(tokens):
        cmd = tokens[i]
        i += 1
        if cmd in ("Z", "z"):
            # A line that ends exactly at the start point is redundant before Z.
            if out and out[-1][0] == "L" and cur == start:
                out.pop()
            out.append("Z")
            cur = start
            last_ctrl = None
            continue
        rel = cmd.islower()
        cmd = cmd.upper()
        if cmd == "M":
            x, y = nums(2)
            if rel:
                x, y = cur[0] + x, cur[1] + y
            cur = (x, y)
            start = cur
            out.append(f"M{_fmt(x)} {_fmt(y)}")
            last_ctrl = None
            # Implicit lineto pairs after an M
            while is_num():
                x, y = nums(2)
                if rel:
                    x, y = cur[0] + x, cur[1] + y
                cur = (x, y)
                out.append(f"L{_fmt(x)} {_fmt(y)}")
                last_ctrl = None
        elif cmd == "L":
            while is_num():
                x, y = nums(2)
                if rel:
                    x, y = cur[0] + x, cur[1] + y
                cur = (x, y)
                out.append(f"L{_fmt(x)} {_fmt(y)}")
                last_ctrl = None
        elif cmd == "H":
            while is_num():
                (x,) = nums(1)
                if rel:
                    x = cur[0] + x
                cur = (x, cur[1])
                out.append(f"L{_fmt(x)} {_fmt(cur[1])}")
                last_ctrl = None
        elif cmd == "V":
            while is_num():
                (y,) = nums(1)
                if rel:
                    y = cur[1] + y
                cur = (cur[0], y)
                out.append(f"L{_fmt(cur[0])} {_fmt(y)}")
                last_ctrl = None
        elif cmd == "Q":
            while is_num():
                x1, y1, x, y = nums(4)
                if rel:
                    x1, y1 = cur[0] + x1, cur[1] + y1
                    x, y = cur[0] + x, cur[1] + y
                cur = (x, y)
                last_ctrl = (x1, y1)
                out.append(f"Q{_fmt(x1)} {_fmt(y1)} {_fmt(x)} {_fmt(y)}")
        elif cmd == "T":
            while is_num():
                x, y = nums(2)
                if rel:
                    x, y = cur[0] + x, cur[1] + y
                if last_ctrl is None:
                    x1, y1 = cur
                else:
                    x1, y1 = 2 * cur[0] - last_ctrl[0], 2 * cur[1] - last_ctrl[1]
                cur = (x, y)
                last_ctrl = (x1, y1)
                out.append(f"Q{_fmt(x1)} {_fmt(y1)} {_fmt(x)} {_fmt(y)}")
        elif cmd == "C":
            while is_num():
                x1, y1, x2, y2, x, y = nums(6)
                if rel:
                    x1, y1 = cur[0] + x1, cur[1] + y1
                    x2, y2 = cur[0] + x2, cur[1] + y2
                    x, y = cur[0] + x, cur[1] + y
                cur = (x, y)
                last_ctrl = (x2, y2)
                out.append(f"C{_fmt(x1)} {_fmt(y1)} {_fmt(x2)} {_fmt(y2)} {_fmt(x)} {_fmt(y)}")
        elif cmd == "S":
            while is_num():
                x2, y2, x, y = nums(4)
                if rel:
                    x2, y2 = cur[0] + x2, cur[1] + y2
                    x, y = cur[0] + x, cur[1] + y
                if last_ctrl is None:
                    x1, y1 = cur
                else:
                    x1, y1 = 2 * cur[0] - last_ctrl[0], 2 * cur[1] - last_ctrl[1]
                cur = (x, y)
                last_ctrl = (x2, y2)
                out.append(f"C{_fmt(x1)} {_fmt(y1)} {_fmt(x2)} {_fmt(y2)} {_fmt(x)} {_fmt(y)}")
        else:
            raise ValueError(f"Unknown path command {cmd}")
    return " ".join(out)


def _seg_pts(seg: tuple) -> tuple[tuple[str, str], ...]:
    """All coordinate points of a segment tuple (M/L: 1, Q: 2, C: 3)."""
    return tuple(zip(seg[1::2], seg[2::2]))


def _reversed_subpath(sub: tuple) -> tuple:
    """The same geometric subpath traversed in the opposite direction."""
    start = sub[0][1:]
    pts = [seg[1:] for seg in sub]  # pts[i] = endpoint of segment i
    rev = [("M",) + pts[-1]]
    for i in range(len(sub) - 1, 0, -1):
        seg = sub[i]
        prev = pts[i - 1]
        if seg[0] == "L":
            rev.append(("L",) + prev)
        elif seg[0] == "Q":
            rev.append(("Q", seg[1], seg[2]) + prev)
        elif seg[0] == "C":
            rev.append(("C", seg[3], seg[4], seg[1], seg[2]) + prev)
    return tuple(rev)


def canonical_path(path: str) -> frozenset:
    """Render-equivalent canonical key for a path.

    Splits the path into subpaths, drops degenerate (zero-area) ones, normalizes
    each subpath's orientation, and returns the set of unique contours. Two paths
    that render identically produce the same key, even if one draws redundant
    contours (e.g. the FILL=1 instance of a glyph draws the same notch twice with
    opposite winding, which cancels out under the nonzero fill rule).
    """
    tokens = normalize_path(path).split()
    subpaths: list[tuple] = []
    cur: list[tuple] | None = None
    i = 0
    while i < len(tokens):
        cmd = tokens[i]
        i += 1
        if cmd == "M":
            if cur:
                subpaths.append(tuple(cur))
            x, y = tokens[i], tokens[i + 1]
            i += 2
            cur = [("M", x, y)]
        elif cmd == "L":
            x, y = tokens[i], tokens[i + 1]
            i += 2
            cur.append(("L", x, y))
        elif cmd == "Q":
            x1, y1, x, y = tokens[i], tokens[i + 1], tokens[i + 2], tokens[i + 3]
            i += 4
            cur.append(("Q", x1, y1, x, y))
        elif cmd == "C":
            x1, y1, x2, y2, x, y = tokens[i : i + 6]
            i += 6
            cur.append(("C", x1, y1, x2, y2, x, y))
        elif cmd == "Z":
            if cur:
                subpaths.append(tuple(cur))
            cur = None
    if cur:
        subpaths.append(tuple(cur))

    keys: set[tuple] = set()
    for sub in subpaths:
        if len(sub) < 2:
            continue
        start = sub[0][1:]
        if all(seg[1:] == start for seg in sub[1:]):
            continue  # zero-area subpath (e.g. repeated single point)
        keys.add(min(sub, _reversed_subpath(sub)))
    return frozenset(keys)


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def _resolve_fill(style: str, fill: float | None) -> float | None:
    if fill is not None:
        return fill
    return DEFAULT_FILL.get(style)


def verify(styles: list[str], fill: float | None) -> int:
    total = mismatches = missing = legacy_kept = 0
    for style in styles:
        path = ICON_DIR / OUTPUT_FILES[style]
        if path is None:
            print(f"[ ] {style}: no generated file")
            continue
        entries = parse_existing(path)
        if not entries:
            print(f"[ ] {style}: no entries found in {OUTPUT_FILES[style]}")
            continue
        resolved = _resolve_fill(style, fill)
        glyphs = {glyph_for(name) for name in entries}
        font = get_font(style, resolved, glyphs)
        print(
            f"[*] Verifying {len(entries)} icons in {OUTPUT_FILES[style]} "
            f"(fill={resolved if resolved is not None else 0}) ..."
        )
        for name, existing in entries.items():
            total += 1
            glyph = glyph_for(name)
            if glyph not in glyphs:
                continue
            try:
                generated = extract_glyph(font, glyph)
            except KeyError:
                if name in LEGACY_GLYPHS:
                    legacy_kept += 1
                    print(f"    LEGACY {name}: no current glyph for '{glyph}'")
                else:
                    missing += 1
                    print(f"    MISSING glyph for {name} ({glyph})")
                continue
            if canonical_path(existing) != canonical_path(generated):
                mismatches += 1
                print(f"    MISMATCH {name} ({glyph})")
        exact = len(entries) - mismatches - missing - legacy_kept
        print(f"    {exact}/{len(entries)} exact")
    print(f"\n{total} icons checked, {mismatches} mismatched, {missing} missing, {legacy_kept} legacy-kept")
    return mismatches + missing


def regen(styles: list[str], fill: float | None) -> int:
    problems = 0
    for style in styles:
        out_path = ICON_DIR / OUTPUT_FILES[style]
        if out_path is None:
            print(f"[!] No generated file for style '{style}'")
            problems += 1
            continue
        entries = parse_existing(out_path)
        if not entries:
            print(f"[!] {OUTPUT_FILES[style]} has no icons to regenerate")
            problems += 1
            continue
        resolved = _resolve_fill(style, fill)
        names = list(entries)
        glyphs = {glyph_for(name) for name in names}
        font = get_font(style, resolved, glyphs)
        new_entries: dict[str, str] = {}
        kept: list[str] = []
        print(
            f"[*] Regenerating {len(names)} icons in {OUTPUT_FILES[style]} "
            f"(fill={resolved if resolved is not None else 0}) ..."
        )
        for name in names:
            glyph = glyph_for(name)
            try:
                new_entries[name] = extract_glyph(font, glyph)
            except KeyError:
                kept.append(name)
                new_entries[name] = entries[name]
        write_file(style, dict(sorted(new_entries.items())))
        for name in kept:
            print(f"    kept {name} (no current glyph '{glyph_for(name)}')")
        if kept:
            problems += len(kept)
    return problems


def add_icons(style: str, names: list[str], fill: float | None) -> int:
    out_path = ICON_DIR / OUTPUT_FILES[style]
    if out_path is None:
        sys.exit(f"No generated file for style '{style}'")
    entries = parse_existing(out_path)
    resolved = _resolve_fill(style, fill)
    valid = [n for n in names if re.match(r"^[A-Za-z0-9]+$", n)]
    glyphs = {glyph_for(n) for n in valid}
    font = get_font(style, resolved, glyphs)
    changed = 0
    for name in valid:
        glyph = glyph_for(name)
        try:
            path_data = extract_glyph(font, glyph)
        except KeyError:
            print(f"[!] No glyph '{glyph}' in {style} style; skipping {name}")
            continue
        is_new = name not in entries
        entries[name] = path_data
        changed += 1
        print(f"[+] {'Added' if is_new else 'Updated'} {style}.{name} ({glyph})")
        if glyph in AUTO_MIRRORED:
            print(
                f"    note: '{glyph}' is an RTL auto-mirrored icon; also add it to the "
                "AutoMirrored object in LampcordIcons.kt if needed."
            )
    if changed:
        # Keep entries alphabetically sorted (matches how new icons are expected).
        write_file(style, dict(sorted(entries.items())))
    return changed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    add = sub.add_parser("add", help="add/refresh icons in the generated files")
    add.add_argument("names", nargs="+", help="icon names, e.g. LightMode DarkMode")
    add.add_argument("--style", choices=STYLE_FONTS, default="filled")
    add.add_argument("--fill", type=float, default=None, help="bake the FILL axis (0 or 1)")

    ver = sub.add_parser("verify", help="check existing icons against the fonts")
    ver.add_argument("--style", choices=STYLE_FONTS, nargs="+", default=["filled", "outlined", "rounded"])
    ver.add_argument("--fill", type=float, default=None)

    re = sub.add_parser("regen", help="regenerate all icons from the current fonts")
    re.add_argument("--style", choices=STYLE_FONTS, nargs="+", default=["filled", "outlined", "rounded"])
    re.add_argument("--fill", type=float, default=None)

    args = parser.parse_args()
    if args.command == "add":
        add_icons(args.style, args.names, args.fill)
    elif args.command == "regen":
        sys.exit(1 if regen(args.style, args.fill) else 0)
    else:
        sys.exit(1 if verify(args.style, args.fill) else 0)


if __name__ == "__main__":
    main()
