import argparse
import json
import math
import re
import sys
import unicodedata
from collections import Counter
from pathlib import Path

SENTENCE_END = re.compile(r'[.!?…]["»)]*$')
WORD = re.compile(r'\w+')
STOPWORDS_FILE = Path(__file__).with_name('stopwords-es.txt')


def fold(word):
    return ''.join(c for c in unicodedata.normalize('NFKD', word.lower()) if not unicodedata.combining(c))


def sentences(segments):
    current, start = [], None
    for segment in segments:
        text = segment['text'].strip()
        if not text:
            continue
        if start is None:
            start = segment['start_s']
        current.append(text)
        if SENTENCE_END.search(text):
            yield start, ' '.join(current)
            current, start = [], None
    if current:
        yield start, ' '.join(current)


def tokens(text, stopwords):
    return [w for w in (fold(raw) for raw in WORD.findall(text)) if len(w) > 2 and w not in stopwords]


def weigh(documents):
    frequency = Counter(term for document in documents for term in set(document))
    total = len(documents)
    return [{term: count * math.log(total / frequency[term]) for term, count in Counter(document).items()} for document in documents]


def cosine(a, b):
    if len(a) > len(b):
        a, b = b, a
    dot = sum(weight * b.get(term, 0.0) for term, weight in a.items())
    norm = math.sqrt(sum(w * w for w in a.values())) * math.sqrt(sum(w * w for w in b.values()))
    return dot / norm if norm else 0.0


def select(vectors, eligible, count, trade_off):
    centroid = Counter()
    for vector in vectors:
        centroid.update(vector)
    relevance = {i: cosine(vectors[i], centroid) for i in eligible}
    chosen = []
    while len(chosen) < count and len(chosen) < len(eligible):
        def marginal(i):
            redundancy = max((cosine(vectors[i], vectors[j]) for j in chosen), default=0.0)
            return trade_off * relevance[i] - (1 - trade_off) * redundancy
        chosen.append(max((i for i in eligible if i not in chosen), key=marginal))
    return chosen


def clock(seconds):
    return f'{int(seconds) // 60:02d}:{int(seconds) % 60:02d}'


def main():
    parser = argparse.ArgumentParser(description='Pick the most central, least repetitive sentences of a podcast-monitor transcript (centroid scoring with maximal marginal relevance)')
    parser.add_argument('transcript', help='transcript.json or transcript.fixed.json')
    parser.add_argument('--count', type=int, default=15)
    parser.add_argument('--trade-off', type=float, default=0.7, help='1 is pure centrality, 0 is pure diversity')
    parser.add_argument('--min-words', type=int, default=12)
    parser.add_argument('--max-words', type=int, default=60)
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding='utf-8')
    stopwords = set(STOPWORDS_FILE.read_text(encoding='utf-8').split())
    data = json.loads(Path(args.transcript).read_text(encoding='utf-8'))
    found = list(sentences(data['segments']))
    vectors = weigh([tokens(text, stopwords) for _, text in found])
    eligible = [i for i, (_, text) in enumerate(found) if args.min_words <= len(WORD.findall(text)) <= args.max_words]
    print(f'{len(found)} sentences, {len(eligible)} eligible', file=sys.stderr)
    for i in sorted(select(vectors, eligible, args.count, args.trade_off), key=lambda i: found[i][0]):
        print(f'{clock(found[i][0])}  {found[i][1]}')


if __name__ == '__main__':
    main()
