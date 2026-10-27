#!/usr/bin/env python3
"""Local, source-attributed briefing from public RSS 2.0 podcasts.
No paid API calls. Requires local Ollama and a multilingual Whisper model.
"""
import argparse
import gc
import hashlib
import json
import re
import textwrap
from datetime import datetime, timedelta, timezone
from email.utils import parsedate_to_datetime
from pathlib import Path
from urllib.parse import urlparse
from xml.etree import ElementTree as ET

import requests
from faster_whisper import WhisperModel
from ollama import chat

USER_AGENT = 'AgendaPodcastMonitor/0.1 (personal research)'
AUDIO_TYPES = ('.mp3', '.m4a', '.mp4', '.ogg', '.opus', '.aac', '.wav')
TRANSCRIPT_TYPES = ('text/plain', 'text/vtt', 'application/x-subrip')
SESSION = requests.Session()
SESSION.headers['User-Agent'] = USER_AGENT


def fetch_bytes(url, max_bytes):
    with SESSION.get(url, stream=True, timeout=(15, 90)) as response:
        response.raise_for_status()
        data = bytearray()
        for piece in response.iter_content(chunk_size=256 * 1024):
            data.extend(piece)
            if len(data) > max_bytes:
                raise ValueError(f'Download exceeds size limit ({max_bytes // 1024**2} MiB): {url}')
    return bytes(data)


def node_text(item, tag):
    node = item.find(tag)
    return (node.text or '').strip() if node is not None else ''


def parse_feed(xml_data):
    root = ET.fromstring(xml_data)
    channel = root.find('channel')
    if channel is None:
        raise ValueError('Only standard RSS 2.0 feeds supported (root/channel/item).')
    podcast_name = node_text(channel, 'title') or 'Podcast'
    episodes = []
    for item in channel.findall('item'):
        enclosure = item.find('enclosure')
        audio = enclosure.get('url', '') if enclosure is not None else ''
        if not audio:
            continue
        transcript = None
        for child in item:
            if child.tag.rsplit('}', 1)[-1] == 'transcript':
                mime = child.get('type', '').split(';')[0].strip().lower()
                if mime in TRANSCRIPT_TYPES and child.get('url'):
                    transcript = {'url': child.get('url'), 'mime': mime}
                    break
        episodes.append({
            'podcast': podcast_name,
            'title': node_text(item, 'title') or 'Sin título',
            'link': node_text(item, 'link') or audio,
            'id': node_text(item, 'guid') or audio,
            'published': node_text(item, 'pubDate'),
            'audio': audio,
            'transcript': transcript,
        })
    return episodes


def in_window(episode, days):
    if not episode['published']:
        return True  # Unknown date: include, never invent a publication date.
    try:
        published = parsedate_to_datetime(episode['published'])
        if published.tzinfo is None:
            published = published.replace(tzinfo=timezone.utc)
        return published >= datetime.now(timezone.utc) - timedelta(days=days)
    except (ValueError, TypeError, IndexError):
        return True


def hhmmss(seconds):
    seconds = max(0, round(seconds))
    return f'{seconds // 3600:02d}:{(seconds // 60) % 60:02d}:{seconds % 60:02d}'


def get_transcript(episode, directory, model_name, gpu):
    transcript_file = directory / 'transcript.txt'
    if transcript_file.exists():
        return transcript_file.read_text(encoding='utf-8')

    # Use publisher transcripts before invoking speech recognition.
    t = episode['transcript']
    if t:
        try:
            raw = fetch_bytes(t['url'], 15 * 1024**2)
            decoded = raw.decode('utf-8-sig', errors='replace')
            if decoded.strip():
                transcript_file.write_text(decoded, encoding='utf-8')
                (directory / 'transcript_source.txt').write_text(t['url'], encoding='utf-8')
                return decoded
        except (requests.RequestException, ValueError) as err:
            print(f'  Publisher transcript unavailable ({err}); using Whisper.')

    parsed = urlparse(episode['audio'])
    extension = Path(parsed.path).suffix.lower()
    if extension not in AUDIO_TYPES:
        extension = '.mp3'  # FFmpeg/PyAV also detect media from the file header.
    audio_file = directory / f'audio{extension}'
    if not audio_file.exists():
        audio_file.write_bytes(fetch_bytes(episode['audio'], 700 * 1024**2))
    print(f'  Transcribing with Whisper {model_name} on {"GPU" if gpu else "CPU"}...')
    model = WhisperModel(model_name, device='cuda' if gpu else 'cpu', compute_type='int8')
    segments, _ = model.transcribe(str(audio_file), language='es', vad_filter=True, beam_size=5)
    lines = [f'[{hhmmss(s.start)} - {hhmmss(s.end)}] {s.text.strip()}' for s in segments]
    del model
    gc.collect()
    transcript = '\n'.join(lines)
    transcript_file.write_text(transcript, encoding='utf-8')
    (directory / 'transcript_source.txt').write_text('Whisper local', encoding='utf-8')
    return transcript


def chunks(text, limit=10500):
    pieces, current = [], ''
    for line in text.splitlines():
        # Avoid losing content if an unusually long line occurs.
        for part in textwrap.wrap(line, width=limit, break_long_words=True,
                                  replace_whitespace=False, drop_whitespace=False) or ['']:
            if len(current) + len(part) + 1 > limit and current:
                pieces.append(current)
                current = ''
            current += part + '\n'
    if current.strip():
        pieces.append(current)
    return pieces


SYSTEM_PROMPT = '''Eres un archivista de actualidad pública chilena. Extrae información
verificable de transcripciones de programas políticos para un boletín informativo
NEUTRAL. No hagas propaganda, no recomiendes estrategias electorales ni infieras
motivaciones. Distingue declaraciones, opiniones, propuestas y hechos comprobables.
Una declaración no es un hecho verificado. Nunca inventes nombres ni fechas.
Devuelve SOLO JSON con forma {"items": [{"tema":"...", "tipo":"declaración | opinión | propuesta | evento anunciado | dato por verificar", "actor":"... o no identificado", "detalle":"...", "momento":"HH:MM:SS o no disponible", "evidencia":"frase exacta presente en la transcripción"}]}.
Extrae solo hechos o afirmaciones sustantivas relevantes para políticas públicas,
legislación, instituciones, economía y agenda pública de Chile. Máximo 8 ítems por
fragmento. No atribuyas una cita a una persona que no esté identificada en el texto.
El campo evidencia debe copiar una frase CONTIGUA exacta del fragmento.
El momento se copia solo si aparece un código de tiempo relacionado en el texto.'''


def normalize(text):
    return re.sub(r'\s+', ' ', text).strip().casefold()


def analyze(transcript, model_name):
    results = []
    for i, chunk in enumerate(chunks(transcript), 1):
        print(f'  Analyzing chunk {i} locally...')
        response = chat(
            model=model_name,
            messages=[{'role': 'system', 'content': SYSTEM_PROMPT},
                      {'role': 'user', 'content': chunk + '\n/no_think'}],
            format='json', options={'temperature': 0}, stream=False,
            think=False,
        )
        try:
            items = json.loads(response.message.content).get('items', [])
        except (ValueError, AttributeError):
            print('  Invalid model JSON; skipping fragment.')
            continue
        for item in items if isinstance(items, list) else []:
            if not isinstance(item, dict):
                continue
            evidence = str(item.get('evidencia') or '').strip()
            if len(evidence) < 12 or normalize(evidence) not in normalize(chunk):
                continue  # Reject unsupported or invented quotations.
            validated = {key: str(item.get(key, '')).strip()
                         for key in ('tema', 'tipo', 'actor', 'detalle', 'momento', 'evidencia')}
            # Do not print a timestamp the original chunk did not contain.
            if normalize(validated['momento']) not in normalize(chunk):
                validated['momento'] = 'no disponible'
            results.append(validated)
    return results


def render_one(episode, items):
    title = episode['title']
    lines = [f'## {title}', f'- **Podcast:** {episode["podcast"]}',
             f'- **Publicado:** {episode["published"] or "fecha no informada"}',
             f'- **Fuente:** {episode["link"]}', '', ]
    if not items:
        lines.append('*Sin hallazgos validados en esta ejecución.*')
    for item in items:
        lines += [f'### {item["tema"] or "Asunto de agenda"}',
                  f'- **Clasificación:** {item["tipo"]}',
                  f'- **Actor:** {item["actor"]}',
                  f'- **Resumen atribuido:** {item["detalle"]}',
                  f'- **Minuto indicado:** {item["momento"] or "no disponible"}',
                  f'- **Texto original:** «{item["evidencia"]}»', '']
    return '\n'.join(lines)


def main():
    parser = argparse.ArgumentParser(description='Offline-priced, local political podcast briefings.')
    parser.add_argument('rss', nargs='+', help='One or more publicly accessible RSS 2.0 feed URLs')
    parser.add_argument('--days', type=int, default=7)
    parser.add_argument('--max-per-feed', type=int, default=2)
    parser.add_argument('--whisper', default='small', help='Multilingual Whisper: tiny, base, small, medium')
    parser.add_argument('--model', default='qwen3:8b', help='Local model in Ollama')
    parser.add_argument('--gpu', action='store_true', help='Needs compatible CUDA 12/cuDNN 9')
    parser.add_argument('--out', default='agenda_output')
    args = parser.parse_args()
    output = Path(args.out)
    output.mkdir(parents=True, exist_ok=True)
    pages = ['# Boletín de seguimiento de agenda pública',
             f'Generado: {datetime.now().astimezone().isoformat(timespec="minutes")}',
             '\n**Aviso:** El informe reproduce declaraciones detectadas automáticamente. '
             'No constituye verificación independiente de los hechos; revisar audio y transcripción.', '']
    count = 0
    for feed_url in args.rss:
        try:
            episodes = parse_feed(fetch_bytes(feed_url, 5 * 1024**2))
        except (ET.ParseError, requests.RequestException, ValueError) as error:
            print(f'Could not parse {feed_url}: {error}')
            continue
        selected = [e for e in episodes if in_window(e, args.days)][:args.max_per_feed]
        for episode in selected:
            key = hashlib.sha256(episode['id'].encode('utf-8')).hexdigest()[:16]
            directory = output / key
            directory.mkdir(exist_ok=True)
            report_path = directory / 'report.md'
            print('Processing:', episode['podcast'], '/', episode['title'])
            try:
                if not report_path.exists():
                    transcript = get_transcript(episode, directory, args.whisper, args.gpu)
                    items = analyze(transcript, args.model)
                    (directory / 'items.json').write_text(json.dumps(items, indent=2, ensure_ascii=False), encoding='utf-8')
                    report_path.write_text(render_one(episode, items), encoding='utf-8')
                pages.append(report_path.read_text(encoding='utf-8'))
                count += 1
            except Exception as error:
                print(f'  Could not process episode: {error}')
    briefing = output / 'boletin.md'
    briefing.write_text('\n\n---\n\n'.join(pages) + '\n', encoding='utf-8')
    print(f'Finished: {count} episodes. Briefing: {briefing.resolve()}')


if __name__ == '__main__':
    main()
