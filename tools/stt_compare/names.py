import argparse
import json
import re
import sys
import unicodedata
from collections import Counter
from pathlib import Path

CAPITALIZED = re.compile(r"(?<![.!?¿¡]\s)(?<!^)\b([A-ZÁÉÍÓÚÑ][a-záéíóúñü]{2,})\b")


def fold(word):
    return ''.join(c for c in unicodedata.normalize('NFKD', word.lower()) if not unicodedata.combining(c))


def capitalized_words(text):
    return Counter(match.group(1) for match in CAPITALIZED.finditer(text))


def main():
    parser = argparse.ArgumentParser(description='Compare how often each mid-sentence capitalized word of a reference transcript appears in another transcript')
    parser.add_argument('reference', help='stt_bench record used as reference')
    parser.add_argument('candidate', help='stt_bench record to check')
    parser.add_argument('--min-count', type=int, default=3)
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding='utf-8')
    reference = json.loads(Path(args.reference).read_text(encoding='utf-8'))['text']
    candidate = json.loads(Path(args.candidate).read_text(encoding='utf-8'))['text']
    candidate_words = Counter(fold(w) for w in re.findall(r'\w+', candidate))
    found = total = 0
    for word, count in capitalized_words(reference).most_common():
        if count < args.min_count:
            break
        hits = candidate_words[fold(word)]
        total += 1
        found += hits > 0
        print(f'{word:20} reference={count:3} candidate={hits:3}')
    print(f'{found} of {total} reference names appear in the candidate at least once')


if __name__ == '__main__':
    main()
