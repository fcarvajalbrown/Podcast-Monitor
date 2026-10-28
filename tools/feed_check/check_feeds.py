import argparse
import json
import sys
from datetime import date
import urllib.parse
import urllib.request
from email.utils import parsedate_to_datetime
from xml.etree import ElementTree as ET

ITUNES_NS = '{http://www.itunes.com/dtds/podcast-1.0.dtd}'
USER_AGENT = 'Mozilla/5.0 PodcastMonitorFeedCheck/0.1'


def get(url):
    request = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def apple_search(term):
    query = urllib.parse.urlencode({'term': term, 'media': 'podcast', 'country': 'cl', 'limit': 5})
    return json.loads(get(f'https://itunes.apple.com/search?{query}'))['results']


def duration_seconds(text):
    if not text:
        return 0
    try:
        total = 0
        for part in text.strip().split(':'):
            total = total * 60 + int(float(part))
        return total
    except ValueError:
        return 0


def summarize_feed(feed_url, since=None):
    channel = ET.fromstring(get(feed_url)).find('channel')
    items = channel.findall('item')
    with_audio = [item for item in items if item.find('enclosure') is not None]
    transcripts = sum(1 for item in items for child in item if child.tag.endswith('}transcript'))
    hours = sum(duration_seconds(item.findtext(f'{ITUNES_NS}duration')) for item in items) / 3600
    dates = []
    recent_seconds = []
    for item in items:
        try:
            published = parsedate_to_datetime(item.findtext('pubDate'))
        except (TypeError, ValueError):
            continue
        dates.append(published)
        if since and published.date() >= since:
            recent_seconds.append(duration_seconds(item.findtext(f'{ITUNES_NS}duration')))
    return {
        'since_episodes': len(recent_seconds),
        'since_hours': round(sum(recent_seconds) / 3600, 1),
        'episodes': len(items),
        'with_audio': len(with_audio),
        'hours': round(hours, 1),
        'transcript_tags': transcripts,
        'oldest': min(dates).date().isoformat() if dates else '',
        'newest': max(dates).date().isoformat() if dates else '',
    }


def check(term, publisher, since=None):
    matches = apple_search(term)
    if publisher:
        matches = [m for m in matches if publisher.lower() in m.get('artistName', '').lower()] or matches
    if not matches:
        return {'show': term, 'status': 'not in Apple directory'}
    match = matches[0]
    feed_url = match.get('feedUrl')
    row = {'show': term, 'apple_name': match.get('collectionName'), 'publisher': match.get('artistName'), 'feed': feed_url}
    if not feed_url:
        row['status'] = 'no feed URL in Apple directory'
        return row
    try:
        row.update(summarize_feed(feed_url, since))
        row['status'] = 'ok'
    except Exception as error:
        row['status'] = f'feed failed: {error}'
    return row


def main():
    parser = argparse.ArgumentParser(description='Find each show in the Apple directory and check its RSS feed')
    parser.add_argument('shows', help='JSON file: list of {"name": ..., "publisher": ...}')
    parser.add_argument('--out', help='write results as JSON here')
    parser.add_argument('--since', type=date.fromisoformat, help='also count episodes and hours published on or after this date (YYYY-MM-DD)')
    args = parser.parse_args()
    sys.stdout.reconfigure(encoding='utf-8')
    with open(args.shows, encoding='utf-8') as handle:
        shows = json.load(handle)
    results = [check(show['name'], show.get('publisher', ''), args.since) for show in shows]
    for row in results:
        print(' | '.join(str(row.get(key, '')) for key in ('show', 'publisher', 'status', 'episodes', 'with_audio', 'hours', 'transcript_tags', 'oldest', 'newest', 'since_episodes', 'since_hours', 'feed')))
    if args.out:
        with open(args.out, 'w', encoding='utf-8') as handle:
            json.dump(results, handle, ensure_ascii=False, indent=2)


if __name__ == '__main__':
    main()
