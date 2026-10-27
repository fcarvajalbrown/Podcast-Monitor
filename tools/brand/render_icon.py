import argparse
import io
import sys
from pathlib import Path

import resvg_py
from PIL import Image

ICO_SIZES = (16, 24, 32, 48, 64, 128, 256)


def render(svg_path, size):
    png = resvg_py.svg_to_bytes(svg_path=str(svg_path), width=size, height=size)
    return Image.open(io.BytesIO(bytes(png))).convert("RGBA")


def main():
    parser = argparse.ArgumentParser(description="Render a square SVG icon to a PNG, an optional multi-size .ico and an optional preview strip at small sizes")
    parser.add_argument("svg")
    parser.add_argument("png")
    parser.add_argument("--size", type=int, default=512)
    parser.add_argument("--ico")
    parser.add_argument("--preview", help="PNG strip showing the icon at 16, 32, 64 and 128 px on light and dark grounds")
    args = parser.parse_args()
    render(args.svg, args.size).save(args.png)
    print(f"{args.png} {args.size}px")
    if args.ico:
        render(args.svg, 256).save(args.ico, sizes=[(s, s) for s in ICO_SIZES])
        print(f"{args.ico} sizes {list(ICO_SIZES)}")
    if args.preview:
        sizes = (16, 32, 64, 128)
        pad = 16
        width = sum(sizes) + pad * (len(sizes) + 1)
        strip = Image.new("RGBA", (width, (128 + 2 * pad) * 2), "white")
        strip.paste((40, 40, 44, 255), (0, 128 + 2 * pad, width, strip.height))
        for row in range(2):
            x = pad
            for s in sizes:
                icon = render(args.svg, s)
                strip.alpha_composite(icon, (x, row * (128 + 2 * pad) + pad + (128 - s) // 2))
                x += s + pad
        strip.save(args.preview)
        print(f"{args.preview} preview")
    return 0


if __name__ == "__main__":
    sys.exit(main())
