#!/usr/bin/env python3
"""Renders the Android apps' KUNA mark and launcher icon from the brand file.

    pip install pillow
    python docs/marketing/make-kuna-mark.py

Input:  brand-assets/kuna-logo.jpg - the official artwork, a dark coin with a
        gold rim and a gold emblem on a light grey backdrop with a soft drop
        shadow.
Output: apps/android-wallet/app/src/main/assets/kuna-logo.webp  (the coin
        mark the balance card and the history rows draw)
        apps/android/app/src/main/assets/kuna-logo.webp         (same file)
        apps/android-wallet/app/src/main/res/drawable-nodpi/ic_kuna.webp
        apps/android/app/src/main/res/drawable-nodpi/ic_kuna.webp

Both apps are ink-dark and drew the native coin as an amber "K" disc, so the
backdrop has to go before the artwork can sit on the surface:

1. a pixel is backdrop when it is light and nearly colourless (the paper and
   the drop shadow are both: the shadow is grey, the coin is gold or dark);
2. only backdrop *connected to the image border* is cleared, so the white
   highlight inside the emblem stays opaque;
3. erode a couple of pixels and blur, which removes the light fringe where the
   coin's antialiased edge blends into the paper;
4. keep a square around the coin so it stays centred in a square view.

The launcher icon is the same coin flattened on the app ink tile, full-bleed
so the launcher's mask defines the shape.
"""

import os
from collections import deque

from PIL import Image, ImageFilter

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, "brand-assets", "kuna-logo.jpg")
MARK = [
    os.path.join(ROOT, "apps", "android-wallet", "app", "src", "main", "assets", "kuna-logo.webp"),
    os.path.join(ROOT, "apps", "android", "app", "src", "main", "assets", "kuna-logo.webp"),
]
ICON = [
    os.path.join(ROOT, "apps", "android-wallet", "app", "src", "main", "res", "drawable-nodpi", "ic_kuna.webp"),
    os.path.join(ROOT, "apps", "android", "app", "src", "main", "res", "drawable-nodpi", "ic_kuna.webp"),
]

MARK_PX = 256      # drawn at 28-40 dp, so 256 covers a 4x density
ICON_PX = 256      # launcher asks for 48 dp; 256 covers 4x as well
ICON_SHARE = 0.80  # coin diameter per tile side, inside the mask's safe zone
INK = (0x0E, 0x11, 0x16)  # WalletUi.INK


def backdrop(rgb, hsv):
    """Light, nearly colourless pixels: paper and shadow, never the coin."""
    _, saturation, value = hsv
    return value > 118 and saturation < 46


def coin_layer():
    """The artwork with the backdrop cleared and the edge softened."""
    im = Image.open(SRC).convert("RGB")
    width, height = im.size
    rgb = im.load()
    hsv = im.convert("HSV").load()

    is_backdrop = bytearray(width * height)
    for y in range(height):
        row = y * width
        for x in range(width):
            if backdrop(rgb[x, y], hsv[x, y]):
                is_backdrop[row + x] = 1

    cleared = bytearray(width * height)
    queue = deque()
    border = [(x, 0) for x in range(width)] + [(x, height - 1) for x in range(width)]
    border += [(0, y) for y in range(height)] + [(width - 1, y) for y in range(height)]
    for x, y in border:
        index = y * width + x
        if is_backdrop[index] and not cleared[index]:
            cleared[index] = 1
            queue.append((x, y))
    while queue:
        x, y = queue.popleft()
        for nx, ny in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
            if 0 <= nx < width and 0 <= ny < height:
                index = ny * width + nx
                if is_backdrop[index] and not cleared[index]:
                    cleared[index] = 1
                    queue.append((nx, ny))

    mask = Image.new("L", (width, height), 0)
    mask_px = mask.load()
    for y in range(height):
        row = y * width
        for x in range(width):
            if not cleared[row + x]:
                mask_px[x, y] = 255

    left, top, right, bottom = mask.getbbox()
    # Inside the coin's edge, so the light fringe never reaches the ink.
    mask = mask.filter(ImageFilter.MinFilter(5)).filter(ImageFilter.GaussianBlur(0.9))

    side = max(right - left, bottom - top) + 16
    x0 = int(round((left + right) / 2 - side / 2))
    y0 = int(round((top + bottom) / 2 - side / 2))
    box = (x0, y0, x0 + side, y0 + side)

    coin = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    coin.paste(im.crop(box), (0, 0))
    coin.putalpha(mask.crop(box))
    return coin


def main():
    coin = coin_layer()
    mark = coin.resize((MARK_PX, MARK_PX), Image.LANCZOS)
    for path in MARK:
        mark.save(path, "WEBP", quality=92, method=6)
        print(f"{os.path.relpath(path, ROOT)}: {os.path.getsize(path)} bytes")

    # Trim the transparent margin so ICON_SHARE is measured on the coin itself.
    art = coin.crop(coin.getbbox())
    side = int(round(ICON_PX * ICON_SHARE))
    art = art.resize((side, side), Image.LANCZOS)
    offset = (ICON_PX - side) // 2
    tile = Image.new("RGBA", (ICON_PX, ICON_PX), INK + (255,))
    tile.alpha_composite(art, (offset, offset))
    for path in ICON:
        tile.convert("RGB").save(path, "WEBP", quality=95, method=6)
        print(f"{os.path.relpath(path, ROOT)}: {os.path.getsize(path)} bytes")


if __name__ == "__main__":
    main()
