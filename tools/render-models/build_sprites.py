"""Join the rendered turn of each model into one horizontal sprite strip.

    python build_sprites.py <render dir> <assets dir>

Expects `<render dir>/<model>/000.png ...`, writes `<assets dir>/<model>-turn.webp`.
"""
import sys
from pathlib import Path

from PIL import Image

MODELS = ("airpod_left", "airpod_right", "airpods_case", "iphone")
WEBP_QUALITY = 92


def build_strip(frames: list[Path]) -> Image.Image:
    images = [Image.open(frame).convert("RGBA") for frame in frames]
    width, height = images[0].size
    strip = Image.new("RGBA", (width * len(images), height))
    for index, image in enumerate(images):
        strip.paste(image, (index * width, 0))
    return strip


def main() -> None:
    render_dir, assets_dir = Path(sys.argv[1]), Path(sys.argv[2])
    for model in MODELS:
        frames = sorted((render_dir / model).glob("*.png"))
        out = assets_dir / f"{model.replace('_', '-')}-turn.webp"
        build_strip(frames).save(out, quality=WEBP_QUALITY, method=6)
        print(f"{out.name}: {len(frames)} frames, {out.stat().st_size // 1024} KB")


if __name__ == "__main__":
    main()
