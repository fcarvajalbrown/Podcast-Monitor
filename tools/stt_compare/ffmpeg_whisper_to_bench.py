import argparse
import json
from pathlib import Path


def segments(jsonl_path):
    content = Path(jsonl_path).read_text(encoding='utf-8')
    if content.lstrip().startswith('{') and '"transcription"' in content[:200000]:
        try:
            return json.loads(content)['transcription']
        except json.JSONDecodeError:
            pass
    rows = []
    for line in content.splitlines():
        line = line.strip()
        if line:
            rows.append(json.loads(line))
    return rows


def main():
    parser = argparse.ArgumentParser(description="Turn an FFmpeg whisper JSON-lines transcript or a whisper-cli -oj JSON file into a Cuña-Cutter stt_bench record")
    parser.add_argument('transcript', help='JSON-lines file from the FFmpeg whisper filter, or JSON from whisper-cli -oj')
    parser.add_argument('--id', required=True, help='episode id, used as the record file name')
    parser.add_argument('--engine', required=True, help='engine folder name, such as whisper-turbo-ffmpeg')
    parser.add_argument('--compute-s', type=float, required=True, help='wall-clock seconds the transcription took')
    parser.add_argument('--audio-s', type=float, required=True, help='episode duration in seconds')
    parser.add_argument('--out', default='work/bench')
    args = parser.parse_args()
    rows = segments(args.transcript)
    record = {
        'id': args.id,
        'engine': args.engine,
        'start': 0.0,
        'end': args.audio_s,
        'audio_s': args.audio_s,
        'compute_s': args.compute_s,
        'text': ' '.join(row['text'].strip() for row in rows),
    }
    folder = Path(args.out) / args.engine
    folder.mkdir(parents=True, exist_ok=True)
    (folder / f'{args.id}.json').write_text(json.dumps(record, ensure_ascii=False, indent=2), encoding='utf-8')
    print(folder / f'{args.id}.json')


if __name__ == '__main__':
    main()
