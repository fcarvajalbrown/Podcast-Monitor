import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description='Count exact spellings of each glossary name in stt_bench records or podcast-monitor transcripts')
    parser.add_argument('glossary', help='text file, one name per line')
    parser.add_argument('records', nargs='+', help='stt_bench records to count')
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding='utf-8')
    names = [line.strip() for line in Path(args.glossary).read_text(encoding='utf-8').splitlines() if line.strip()]
    counts = []
    for record in args.records:
        data = json.loads(Path(record).read_text(encoding='utf-8'))
        text = data['text'] if 'text' in data else ' '.join(segment['text'] for segment in data['segments'])
        counts.append(Counter(re.findall(r'\w+', text)))
    labels = [f'{Path(record).parent.name}/{Path(record).stem}' for record in args.records]
    print('name'.ljust(14) + ''.join(label[:24].rjust(26) for label in labels))
    for name in names:
        print(name.ljust(14) + ''.join(str(count[name]).rjust(26) for count in counts))


if __name__ == '__main__':
    main()
