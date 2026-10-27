import argparse
import json
import re
import sys
import unicodedata

import fitz

QUOTE = re.compile(r"\[(\d{2,}:\d{2})\] «(.*?)»", re.S)
FOOTER_Y = 790
HEADER_Y = 95


def fold(text):
    text = unicodedata.normalize("NFKD", text.lower())
    text = "".join(char for char in text if not unicodedata.combining(char))
    return " ".join(re.sub(r"[^\w\s]", " ", text).split())


def clock(seconds):
    whole = int(max(seconds, 0))
    return f"{whole // 60:02}:{whole % 60:02}"


def pdf_text(path):
    lines = []
    for number, page in enumerate(fitz.open(path)):
        for block in page.get_text("dict")["blocks"]:
            for line in block.get("lines", []):
                y = line["bbox"][1]
                if y > FOOTER_Y or (number == 0 and y < HEADER_Y):
                    continue
                lines.append("".join(span["text"] for span in line["spans"]))
    text = ""
    for line in lines:
        if text.endswith("-") and line[:1].islower():
            text = text[:-1] + line
        else:
            text += ("\n" if text else "") + line
    return text


def found_at(quote, minute, segments):
    target = fold(quote)
    for index, segment in enumerate(segments):
        if clock(segment["start_s"]) != minute:
            continue
        first = fold(segment["text"])
        following = fold(" ".join(item["text"] for item in segments[index:index + 12]))
        if following.find(target) != -1 and following.find(target) < max(len(first), 1):
            return True
    return False


def main():
    parser = argparse.ArgumentParser(description="Check that every [mm:ss] «quote» printed in a report PDF starts in a transcript segment that begins at that minute")
    parser.add_argument("pdf")
    parser.add_argument("transcripts", nargs="+", help="transcript JSON files with a segments list")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    transcripts = {path: json.load(open(path, encoding="utf-8"))["segments"] for path in args.transcripts}
    quotes = QUOTE.findall(pdf_text(args.pdf))
    failures = 0
    for minute, text in quotes:
        text = " ".join(text.split())
        hits = [path for path, segments in transcripts.items() if found_at(text, minute, segments)]
        failures += not hits
        print(f"{'ok  ' if hits else 'FAIL'}  [{minute}]  {text[:70]}  {' '.join(hits)}")
    print(f"{len(quotes)} quotes, {failures} not found at their minute")
    sys.exit(1 if failures or not quotes else 0)


if __name__ == "__main__":
    main()
