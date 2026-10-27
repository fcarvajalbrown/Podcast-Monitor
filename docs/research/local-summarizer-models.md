# Lightweight local summarizer models

Question: is there a small model, as Whisper is for transcription, that could summarize episodes locally instead of through the reader's assistant CLI. Twelve web searches. No source tests Spanish podcast summarization on any small model, so nothing here ranks them.

## Candidates named by the sources

- Qwen3 4B and 8B: [model guide for 4-8 GB cards](https://www.mayhemcode.com/2026/06/best-local-llms-for-4gb-6gb-and-8gb.html). Qwen3 8B at Q4 fits in about 5 GB; Qwen3 is listed under Apache 2.0 ([Yandex AI Studio list](https://aistudio.yandex.ru/en/docs/ai-studio/concepts/generation/batch-processing)). No Spanish-specific score for Qwen3-4B-Instruct-2507 was found; the pages relaying its model card give English numbers only (MMLU-Pro 69.6).
- Gemma 3 4B and 12B: 128K context, 140+ languages, summarization listed as an intended use ([Hugging Face](https://huggingface.co/blog/gemma3)). No Spanish summarization evidence.
- Gemma 3n E4B: runs in llama.cpp ([Unsloth guide](https://unsloth.ai/docs/models/tutorials/gemma-3-how-to-run-and-fine-tune/gemma-3n-how-to-run-and-fine-tune)). No Spanish evidence.
- Phi-4-mini: Spanish is on its language list; its multilingual ability is limited by size ([Towards AI](https://pub.towardsai.net/small-and-cool-inside-microsofts-new-phi-4-mini-and-phi-4-multimodal-6a15ac7a86f0)).
- Llama 3.2 3B: Spanish officially supported.
- Salamandra 2B and 7B (BSC, Apache 2.0, 35 European languages): the only summarization scores found are Catalan ([Salamandra report](https://arxiv.org/pdf/2502.08489)). Spanish clickbait summarization dataset with 2B to 70B models: [NoticIA](https://arxiv.org/pdf/2404.07611); per-model scores not retrieved.

## Evidence that bears on the risk

- Faithfulness: on Vectara's short-news hallucination test, a scraped mirror lists Qwen3-4B at 5.7 % hallucinated and Qwen3 8B at 95.2 % consistent ([benchmarklist mirror](https://benchmarklist.com/benchmarks/vectara_hhem/); official board: [GitHub](https://github.com/vectara/hallucination-leaderboard/)). English, short documents, unverified mirror.
- Long context: small models lose accuracy past about 25k tokens and miss the middle of the context; Llama3.2-3B averaged 57.81 % on needle-in-a-haystack ([survey](https://arxiv.org/pdf/2409.15790)). Nothing measured for Qwen3 4B. A 78-minute episode is long, so chunking is needed ([map-reduce and refine trade-offs](https://towardsdatascience.com/summarize-podcast-transcripts-and-long-texts-better-with-nlp-and-ai-e04c89d3b2cb)).
- Output shape: llama.cpp grammars force valid JSON but not correct content; the schema must also be described in the prompt and validated in code ([Liquid docs](https://docs.liquid.ai/deployment/on-device/llama-cpp/structured-output)).
- Speed on the reader's M1: no sourced number. A bandwidth estimate (68 GB/s, 4B at Q4) gives roughly 19-23 tokens/s generation; an estimate, not a measurement.

## Open

Which model, if any, is good enough in Spanish is untested. Needs a run on a Spanish transcript we already hold (Ep. 34) with a quote and minute check against the transcript.
