import argparse
import json
import sys

import fitz


def page_layout(page):
    lines = []
    for block in page.get_text("dict")["blocks"]:
        for line in block.get("lines", []):
            first = line["spans"][0]
            lines.append({
                "baseline": round(first["origin"][1], 1),
                "x0": round(line["bbox"][0], 1),
                "x1": round(line["bbox"][2], 1),
                "size": round(first["size"], 1),
                "font": first["font"].split("+")[-1],
                "color": f"#{first['color']:06X}",
                "text": "".join(span["text"] for span in line["spans"]),
            })
    rules = [
        {"y": round(item["rect"].y0, 1), "x0": round(item["rect"].x0, 1), "x1": round(item["rect"].x1, 1), "width": round(item.get("width") or 0, 2),
         "color": "#" + "".join(f"{round(c * 255):02X}" for c in (item.get("color") or (0, 0, 0)))}
        for item in page.get_drawings()
    ]
    return {"lines": sorted(lines, key=lambda line: (line["baseline"], line["x0"])), "rules": sorted(rules, key=lambda rule: rule["y"])}


def main():
    parser = argparse.ArgumentParser(description="Print text line positions, fonts and rules of every page of a PDF, to compare a report against gen_pdf_desk.py output")
    parser.add_argument("pdf")
    parser.add_argument("--json", action="store_true", help="print JSON instead of one line per item")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    document = fitz.open(args.pdf)
    pages = [page_layout(page) for page in document]
    fonts = sorted({font[3].split("+")[-1] for page in document for font in page.get_fonts()})
    if args.json:
        print(json.dumps({"pages": pages, "fonts": fonts}, ensure_ascii=False, indent=1))
        return
    print(f"pages {len(pages)}  fonts {', '.join(fonts)}")
    for number, page in enumerate(pages, 1):
        print(f"-- page {number}")
        for rule in page["rules"]:
            print(f"rule  y {rule['y']}  x {rule['x0']}-{rule['x1']}  {rule['width']} pt  {rule['color']}")
        for line in page["lines"]:
            print(f"text  y {line['baseline']}  x {line['x0']}-{line['x1']}  {line['size']} {line['font']} {line['color']}  {line['text'][:50]}")


if __name__ == "__main__":
    main()
