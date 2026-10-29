import argparse
import json


def main():
    parser = argparse.ArgumentParser(description='Print the speed of one podcast-monitor transcript')
    parser.add_argument('variant')
    parser.add_argument('transcript', help='transcript.json')
    args = parser.parse_args()
    with open(args.transcript, encoding='utf-8') as file:
        transcript = json.load(file)
    compute = transcript['compute_s']
    audio = transcript['audio_s']
    print(f'{args.variant}: compute_s {compute:.0f}, audio_s {audio:.0f}, {audio / compute:.1f}x, {len(transcript["segments"])} segments')


if __name__ == '__main__':
    main()
