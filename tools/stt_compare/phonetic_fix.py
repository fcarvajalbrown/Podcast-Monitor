import argparse
import json
import re
import sys
import unicodedata
from collections import Counter
from difflib import SequenceMatcher
from pathlib import Path

WORD = re.compile(r"\w+", re.UNICODE)
SPANISH_RULES = [
    (r"ll", "y"),
    (r"qu", "k"),
    (r"c(?=[ei])", "s"),
    (r"ch", "X"),
    (r"c", "k"),
    (r"z", "s"),
    (r"x", "ks"),
    (r"v", "b"),
    (r"w", "b"),
    (r"h", ""),
    (r"ñ", "N"),
    (r"(.)\1", r"\1"),
]


def strip_accents(text):
    return "".join(c for c in unicodedata.normalize("NFKD", text) if not unicodedata.combining(c) or c == "̃")


def phonetic_key(word):
    key = word.lower().replace("ñ", "\0")
    key = strip_accents(key).replace("\0", "ñ")
    for pattern, replacement in SPANISH_RULES:
        key = re.sub(pattern, replacement, key)
    return key


def similarity(a, b):
    return SequenceMatcher(None, a, b).ratio()


def best_match(word, glossary, threshold):
    key = phonetic_key(word)
    scored = [(similarity(key, phonetic_key(name)), name) for name in glossary]
    score, name = max(scored)
    if score >= threshold and word != name and abs(len(word) - len(name)) <= 2:
        return name, score
    return None, score


def common_words(text, min_count):
    lowercase = Counter(word for word in WORD.findall(text) if word.islower())
    return {word for word, count in lowercase.items() if count >= min_count}


def correct(text, glossary, threshold, capitalized_only, common_min):
    replacements = Counter()
    common = common_words(text, common_min) if common_min else set()

    def replace(match):
        word = match.group(0)
        if capitalized_only and not word[:1].isupper():
            return word
        if len(word) < 4 or word.lower() in common:
            return word
        name, _ = best_match(word, glossary, threshold)
        if name is None:
            return word
        replacements[(word, name)] += 1
        return name

    return WORD.sub(replace, text), replacements


def main():
    parser = argparse.ArgumentParser(description="Replace transcript words that sound like a glossary name with that name")
    parser.add_argument("record", help="stt_bench record to correct")
    parser.add_argument("glossary", help="text file, one name per line")
    parser.add_argument("--threshold", type=float, default=0.75)
    parser.add_argument("--all-words", action="store_true", help="also consider lowercase words")
    parser.add_argument("--common-min", type=int, default=2, help="skip words seen this many times in lowercase in the same text; 0 disables")
    parser.add_argument("--out", help="write the corrected record here")
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding="utf-8")
    record = json.loads(Path(args.record).read_text(encoding="utf-8"))
    glossary = [line.strip() for line in Path(args.glossary).read_text(encoding="utf-8").splitlines() if line.strip()]
    text, replacements = correct(record["text"], glossary, args.threshold, not args.all_words, args.common_min)
    for (word, name), count in replacements.most_common():
        print(f"{word:20} -> {name:15} x{count}")
    print(f"{sum(replacements.values())} replacements")
    if args.out:
        Path(args.out).parent.mkdir(parents=True, exist_ok=True)
        Path(args.out).write_text(json.dumps({**record, "text": text}, ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
