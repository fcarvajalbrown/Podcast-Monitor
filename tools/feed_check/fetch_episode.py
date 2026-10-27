import argparse
import json
import sys
import urllib.request
from email.utils import parsedate_to_datetime
from pathlib import Path
from xml.etree import ElementTree as ET

from check_feeds import ITUNES_NS, USER_AGENT, duration_seconds, get


def newest_items(feed_url):
    items = [item for item in ET.fromstring(get(feed_url)).find('channel').findall('item') if item.find('enclosure') is not None]
    return sorted(items, key=lambda item: parsedate_to_datetime(item.findtext('pubDate')), reverse=True)


def download(url, target):
    partial = target.with_suffix(target.suffix + '.part')
    request = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(request, timeout=60) as response, open(partial, 'wb') as out:
        while chunk := response.read(1 << 20):
            out.write(chunk)
    partial.replace(target)


def main():
    parser = argparse.ArgumentParser(description="Download one episode's audio and metadata from a podcast RSS feed")
    parser.add_argument('feed', help='RSS feed URL')
    parser.add_argument('--out', required=True, help='folder for audio.mp3 and episode.json')
    parser.add_argument('--index', type=int, default=0, help='0 is the newest episode')
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding='utf-8')
    item = newest_items(args.feed)[args.index]
    folder = Path(args.out)
    folder.mkdir(parents=True, exist_ok=True)
    meta = {
        'feed': args.feed,
        'title': item.findtext('title'),
        'published': parsedate_to_datetime(item.findtext('pubDate')).isoformat(),
        'duration_s': duration_seconds(item.findtext(f'{ITUNES_NS}duration')),
        'audio_url': item.find('enclosure').get('url'),
        'description': item.findtext('description') or item.findtext(f'{ITUNES_NS}summary') or '',
    }
    (folder / 'episode.json').write_text(json.dumps(meta, ensure_ascii=False, indent=2), encoding='utf-8')
    audio = folder / 'audio.mp3'
    if not audio.exists():
        download(meta['audio_url'], audio)
    print(json.dumps({k: meta[k] for k in ('title', 'published', 'duration_s')}, ensure_ascii=False))
    print(meta['description'])


if __name__ == '__main__':
    main()
