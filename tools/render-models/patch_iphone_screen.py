"""Put our own wallpaper on the iPhone 14 Pro model's display.

    python patch_iphone_screen.py <texture dir> <output dir>

Copies all textures and replaces the display area of the albedo and emissive
maps with a generated wallpaper. The display box is pinned to that one atlas;
the Dynamic Island is found in it and redrawn as a black pill.
"""
import shutil
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

# Display area in the 4096 px atlas, measured on the emissive map: outermost lit
# columns, top edge at row 1578, running to the atlas's bottom
SCREEN_BOX = (2636, 1578, 3803, 4096)
SCREEN_CORNER_RATIO = 0.14
# The island sits in the top strip of the display; dark there means island
ISLAND_STRIP_RATIO = 0.07
ISLAND_MAX_BRIGHTNESS = 45
ISLAND_TOP_SKIP_PX = 12
ISLAND_COLOR = (0, 0, 0)
PATCHED_MAPS = ("Material_albedo.jpg", "Material_emissive.jpg")
JPEG_QUALITY = 95

BASE_TOP = (22, 30, 78)
BASE_BOTTOM = (6, 8, 22)
# (centre x, centre y, radius) as fractions of the screen, then colour
GLOWS = (
    ((0.15, 0.18, 0.55), (124, 92, 255)),
    ((0.92, 0.42, 0.5), (46, 200, 232)),
    ((0.25, 0.78, 0.5), (190, 70, 220)),
    ((0.8, 0.95, 0.4), (60, 110, 255)),
)
GLOW_BLUR_RATIO = 0.18


def wallpaper(size):
    width, height = size
    image = Image.new("RGB", size)
    draw = ImageDraw.Draw(image)
    for y in range(height):
        share = y / (height - 1)
        draw.line([(0, y), (width, y)], fill=tuple(round(t + (b - t) * share) for t, b in zip(BASE_TOP, BASE_BOTTOM)))
    glow = Image.new("RGB", size, (0, 0, 0))
    glow_draw = ImageDraw.Draw(glow)
    for (cx, cy, radius), color in GLOWS:
        r = radius * width
        glow_draw.ellipse([cx * width - r, cy * height - r, cx * width + r, cy * height + r], fill=color)
    glow = glow.filter(ImageFilter.GaussianBlur(GLOW_BLUR_RATIO * width))
    return Image.blend(image, Image.composite(glow, image, glow.convert("L")), 0.85)


def display_shape(size):
    width, height = size
    mask = Image.new("L", size, 0)
    ImageDraw.Draw(mask).rounded_rectangle([0, 0, width - 1, height - 1], radius=SCREEN_CORNER_RATIO * width, fill=255)
    return mask.filter(ImageFilter.GaussianBlur(1.5))


def island_box(original):
    """The Dynamic Island: the dark pill in the middle of the display's top strip."""
    width, height = original.size
    offset_x, offset_y = width // 4, ISLAND_TOP_SKIP_PX
    strip = original.convert("L").crop((offset_x, offset_y, 3 * width // 4, int(ISLAND_STRIP_RATIO * height)))
    left, top, right, bottom = strip.point(lambda v: 255 if v < ISLAND_MAX_BRIGHTNESS else 0).getbbox()
    return offset_x + left, offset_y + top, offset_x + right, offset_y + bottom


def patched_display(paper, original):
    display = Image.composite(paper, original, display_shape(original.size))
    left, top, right, bottom = island_box(original)
    ImageDraw.Draw(display).rounded_rectangle([left, top, right, bottom], radius=(bottom - top) / 2, fill=ISLAND_COLOR)
    return display


def main():
    source, target = Path(sys.argv[1]), Path(sys.argv[2])
    target.mkdir(parents=True, exist_ok=True)
    for file in source.iterdir():
        shutil.copy2(file, target / file.name)
    size = (SCREEN_BOX[2] - SCREEN_BOX[0], SCREEN_BOX[3] - SCREEN_BOX[1])
    paper = wallpaper(size)
    for name in PATCHED_MAPS:
        atlas = Image.open(source / name).convert("RGB")
        atlas.paste(patched_display(paper, atlas.crop(SCREEN_BOX)), SCREEN_BOX[:2])
        atlas.save(target / name, quality=JPEG_QUALITY)
    paper.save(target / "wallpaper-preview.png")
    print(f"patched {', '.join(PATCHED_MAPS)} into {target}")


if __name__ == "__main__":
    main()
