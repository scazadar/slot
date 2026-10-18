"""Draw the SNES cartridge templates into assets/.

    python tools/snes_cart.py

The cart is the shape Nintendo sold in Germany and the rest of the PAL world, the same mould
as the Super Famicom's, seen from the front: a wide landscape shell with soft corners, grip
ridges down both upper sides, a broad label across the top, and below it a raised centre
panel with a three-part finger groove, between two screws. It writes

    snes_cart.png      white, the silhouette in the alpha (the mask and the paint)
    snes_neutral.png   the 50% grey plastic the game's colour is overlaid on
    snes_interior.png  the back half, shown through a clear shell
    snes_board.png     the board behind a clear shell, the Game Boy board widened

all 360x232, the size `bitmaps::SNES_CART_W/H` declares. The label well is
x 45..315, y 8..102, the rect `cart::SNES_LABEL_*` declares.
"""

from pathlib import Path

import numpy as np
from PIL import Image

W, H = 360, 232
SS = 4  # supersampling for the silhouette's edge
TOP_R = 22
FOOT_R = 14
LABEL = (45, 8, 315, 102)  # 270x94: a PAL label is 103.5 x 36 mm
LABEL_R = 10
PANEL = (52, 316 - 8)  # the centre panel's two side seams
GROOVE = (84, 128, 276, 146)
LABEL_GREY = 119
PLASTIC = 124
OUT = Path(__file__).resolve().parent.parent / "assets"


def rounded_rect(xs, ys, x0, y0, x1, y1, r_top, r_bottom):
    inside = (xs >= x0) & (xs < x1) & (ys >= y0) & (ys < y1)
    for cx, cy, r, top in (
        (x0 + r_top, y0 + r_top, r_top, True),
        (x1 - r_top, y0 + r_top, r_top, True),
        (x0 + r_bottom, y1 - r_bottom, r_bottom, False),
        (x1 - r_bottom, y1 - r_bottom, r_bottom, False),
    ):
        if r == 0:
            continue
        corner = ((xs < cx) if cx < (x0 + x1) / 2 else (xs >= cx)) & (
            (ys < cy) if top else (ys >= cy)
        )
        inside &= ~corner | ((xs - cx) ** 2 + (ys - cy) ** 2 <= r * r)
    return inside


def silhouette(xs, ys):
    return rounded_rect(xs, ys, 0, 0, W, H, TOP_R, FOOT_R)


def coverage():
    ys, xs = np.mgrid[0 : H * SS, 0 : W * SS]
    hit = silhouette((xs + 0.5) / SS, (ys + 0.5) / SS)
    return hit.reshape(H, SS, W, SS).mean(axis=(1, 3))


def edges(mask):
    """Distance in px (up to 6) from each inside pixel to the outline, per side."""
    solid = mask >= 0.5
    near = {}
    for name, dy, dx in (("top", -1, 0), ("left", 0, -1), ("bottom", 1, 0), ("right", 0, 1)):
        d = np.full(mask.shape, 99)
        for k in range(6, 0, -1):
            shifted = np.zeros_like(solid)
            src = solid[max(0, dy * k) : H + min(0, dy * k), max(0, dx * k) : W + min(0, dx * k)]
            shifted[max(0, -dy * k) : H + min(0, -dy * k), max(0, -dx * k) : W + min(0, -dx * k)] = src
            d[solid & ~shifted] = k
        near[name] = d
    return near


def noise(seed, sigma):
    rng = np.random.default_rng(seed)
    grain = rng.normal(0, sigma, (H, W))
    # A little blur so the grain reads as moulded texture, not pixel noise.
    k = np.array([1, 2, 1]) / 4
    grain = np.apply_along_axis(lambda r: np.convolve(r, k, "same"), 1, grain)
    return np.apply_along_axis(lambda c: np.convolve(c, k, "same"), 0, grain)


def bevel(grey, mask, light, dark):
    near = edges(mask)
    for side, amount in (("top", light), ("left", light * 0.7), ("bottom", dark), ("right", dark * 0.7)):
        d = near[side]
        grey += np.where(d <= 4, amount * (5 - d) / 4, 0)
    return grey


def recess(grey, x0, y0, x1, y1, r, depth):
    ys, xs = np.mgrid[0:H, 0:W]
    well = rounded_rect(xs, ys, x0, y0, x1, y1, r, r)
    lip_tl = rounded_rect(xs, ys, x0 - 2, y0 - 2, x1, y1, r + 2, r + 2) & ~well
    lip_br = rounded_rect(xs, ys, x0, y0, x1 + 2, y1 + 2, r + 2, r + 2) & ~well
    grey[lip_tl] -= depth
    grey[lip_br & ~lip_tl] += depth * 0.8
    return well


def seam(grey, ys, xs, depth=22):
    """A moulding line: a dark groove with its lit lip just below or right of it."""
    grey[ys, xs] -= depth
    if isinstance(ys, slice):
        grey[ys, xs + 1] += depth * 0.6
    else:
        grey[ys + 1, xs] += depth * 0.6


def screw(grey, cx, cy):
    ys, xs = np.mgrid[0:H, 0:W]
    r2 = (xs - cx) ** 2 + (ys - cy) ** 2
    grey[r2 < 7**2] -= 30
    head = r2 < 5**2
    grey[head] = 168 + 30 * np.clip((cy - ys[head] + cx - xs[head]) / 7, -1, 1)
    grey[(r2 < 2**2)] = 110


def neutral(mask):
    grey = np.full((H, W), float(PLASTIC)) + noise(1, 5)
    grey = bevel(grey, mask, 46, -34)
    # Grip ridges down both upper sides, beside the label.
    for y in range(22, 84, 10):
        for x0, x1 in ((10, 32), (W - 32, W - 10)):
            grey[y : y + 3, x0:x1] -= 26
            grey[y + 3, x0:x1] += 16
    well = recess(grey, *LABEL, LABEL_R, 34)
    grey[well] = LABEL_GREY
    # The raised centre panel under the label, between two side seams.
    top = LABEL[3] + 8
    left, right = PANEL
    seam(grey, slice(top, H - 4), left)
    seam(grey, slice(top, H - 4), right)
    seam(grey, top, slice(left, right + 1))
    grey[top + 1 : H - 4, left + 2 : right - 1] += 6
    # The finger groove: a long rounded slot split in three.
    x0, y0, x1, y1 = GROOVE
    groove = recess(grey, x0, y0, x1, y1, (y1 - y0) // 2, 30)
    grey[groove] -= 14
    for x in (x0 + (x1 - x0) // 3, x0 + 2 * (x1 - x0) // 3):
        grey[y0 + 2 : y1 - 2, x] += 22
        grey[y0 + 2 : y1 - 2, x + 1] -= 12
    screw(grey, 20, H - 20)
    screw(grey, W - 20, H - 20)
    return grey


def interior(mask):
    grey = np.full((H, W), 99.0) + noise(2, 4)
    grey = bevel(grey, mask, 30, -30)
    # The back's raised middle, where the warning sticker sits, and its two screw holes.
    seam(grey, slice(20, H - 6), 60, 18)
    seam(grey, slice(20, H - 6), W - 60, 18)
    seam(grey, 20, slice(60, W - 60 + 1), 18)
    ys, xs = np.mgrid[0:H, 0:W]
    for cx in (22, W - 22):
        r2 = (xs - cx) ** 2 + (ys - (H - 22)) ** 2
        grey[r2 < 6**2] -= 40
        grey[r2 < 3**2] = 30
    return grey


def board():
    """The Game Boy pak's board at the SNES cart's height, its sides carried out in its own
    green so it fills the wider cart."""
    gb = Image.open(OUT / "gb_board.png").convert("RGBA")
    w = round(gb.width * H / gb.height)
    gb = gb.resize((w, H), Image.LANCZOS)
    px = np.array(gb)
    green = np.median(px[H // 3 : 2 * H // 3, 4:12, :3].reshape(-1, 3), axis=0)
    out = np.zeros((H, W, 4), np.uint8)
    out[..., :3] = green.astype(np.uint8)
    out[..., 3] = 255
    x0 = (W - w) // 2
    alpha = px[..., 3:4] / 255
    out[:, x0 : x0 + w, :3] = (px[..., :3] * alpha + out[:, x0 : x0 + w, :3] * (1 - alpha)).astype(np.uint8)
    Image.fromarray(out, "RGBA").save(OUT / "snes_board.png", optimize=True)


def write(name, grey, alpha):
    g = np.clip(np.round(grey), 0, 255).astype(np.uint8)
    rgba = np.dstack([g, g, g, np.round(alpha * 255).astype(np.uint8)])
    Image.fromarray(rgba, "RGBA").save(OUT / name, optimize=True)


def main():
    mask = coverage()
    write("snes_cart.png", np.full((H, W), 255.0), mask)
    write("snes_neutral.png", neutral(mask), mask)
    write("snes_interior.png", interior(mask), mask)
    board()


if __name__ == "__main__":
    main()
