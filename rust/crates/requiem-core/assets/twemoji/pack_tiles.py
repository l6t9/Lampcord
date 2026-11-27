"""Splits atlas.png (63x63 grid of 72px Twemoji tiles) into tiles.bin: one small
PNG per grid cell (index = row * 63 + col), so the app decodes only the emoji
actually on screen instead of holding the whole 4536x4536 atlas (78MB) in RAM.

Layout: u32 LE tile count N, then N+1 u32 LE byte offsets (relative to the end
of this table), then the concatenated PNG blobs (empty cells have length 0).
Run: python pack_tiles.py
"""
import io, struct
from PIL import Image

TILE, COLS = 72, 63
atlas = Image.open("atlas.png").convert("RGBA")
blobs = []
for i in range(COLS * COLS):
    col, row = i % COLS, i // COLS
    tile = atlas.crop((col * TILE, row * TILE, (col + 1) * TILE, (row + 1) * TILE))
    if tile.getbbox() is None:
        blobs.append(b"")
        continue
    buf = io.BytesIO()
    tile.save(buf, "PNG", optimize=True)
    blobs.append(buf.getvalue())
offsets, pos = [], 0
for b in blobs:
    offsets.append(pos)
    pos += len(b)
offsets.append(pos)
with open("tiles.bin", "wb") as f:
    f.write(struct.pack("<I", len(blobs)))
    f.write(struct.pack(f"<{len(offsets)}I", *offsets))
    for b in blobs:
        f.write(b)
print(len(blobs), "tiles,", pos // 1024, "KiB of PNG data")
