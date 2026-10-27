<p align="center"><img src="assets/brand/podcast-monitor-icon.svg" alt="Podcast-Monitor logo" width="128"></p>

# Local Political Podcast Monitor

A local RSS-to-briefing proof of concept. It does **not** call Claude or paid inference APIs.

## Install

1. Install Python 3.10+ and Ollama. In a PowerShell shell with Ollama available:

```powershell
ollama pull qwen3:8b
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

2. Run:

```powershell
python monitor.py "https://EXAMPLE.COM/podcast/rss.xml" --days 7 --max-per-feed 2
```

Or pass multiple feed URLs in one invocation. Reports go to `agenda_output/boletin.md`; transcripts, per-episode reports and extracted items are saved under hashed episode IDs. Re-running reuses completed reports.

## Options

- `--whisper base` for lower RAM and faster ASR, or `--whisper medium` for slower potentially more accurate ASR.
- `--model qwen3:4b` if 8B is too heavy locally (first run `ollama pull qwen3:4b`).
- `--gpu` for Whisper CUDA GPU inference. Verify you have compatible NVIDIA CUDA 12 and cuDNN 9; omit this flag for a reliable CPU path.
- `--days 7`, `--max-per-feed 2`, `--out agenda_output` are defaults.
- Publisher-provided plain text / SRT / VTT transcripts from `<podcast:transcript>` take priority over downloading audio and Whisper.

## Restrictions / limitations

- Supports public RSS 2.0 feeds with `item` and audio `enclosure`. Does not implement Atom or authenticated feeds.
- Some RSS transcript formats (HTML, JSON) are not supported by the transcript-first parser and will fall back to audio.
- Uses no real speaker diarization. Speaker names are only included if explicit in transcription.
- Local LLM output is **not factual verification**. Contiguous source quotes are checked against transcript chunk; other generated fields require editorial review.
- Publisher transcript time codes may differ in format. Plain text transcripts may lack time codes.
- The report aggregates episodes; it does not deduplicate/merge identical stories across series or independently fact-check against primary sources.
- Downloads and analysis may take a while on CPU. Audio can be hundreds of MB. Respect each provider's rights, terms and rate limits.
- The local Ollama daemon must be running, and models download the first time.
- Designed for a single operator and trusted feed list, not a public web service. For production, harden remote URL handling against SSRF and limit redirects and memory usage.

## Scheduling on Windows

Use Windows Task Scheduler and point it to the virtual environment Python path and the script with feed URL arguments. Do **not** launch Claude Code for routine scheduled runs.

## Model note

Use multilingual Whisper model names (`tiny`, `base`, `small`, `medium`), not `.en` models, for Chilean Spanish.
