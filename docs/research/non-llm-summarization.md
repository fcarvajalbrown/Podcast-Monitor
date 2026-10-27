# Summarizing without an LLM

Question: can the report be built with methods from before LLMs. Twenty-four web searches. No source tests any of these on Spanish podcast transcripts, so nothing here ranks them for this project.

## Extractive methods (pick existing sentences)

All of these return sentences copied from the transcript, which fits the PRD rule that quotes keep the transcript's own words.

- Luhn, 1958: score words by frequency, score sentences by their densest cluster of significant words ([Luhn paper](https://courses.ischool.berkeley.edu/i256/f06/papers/luhn58.pdf), [history note](https://historyofinformation.com/detail.php?id=764)).
- Centroid (MEAD): TF-IDF vectors, centroid is the sum, each sentence scored by cosine to it ([Radev et al.](https://arxiv.org/abs/cs/0005020)). Later work treats it as a baseline that can match more complex methods on DUC 2004 ([revisited](https://ar5iv.labs.arxiv.org/html/1708.07690)).
- TextRank and LexRank: sentences as graph nodes, edges from lexical similarity, PageRank-style centrality ([TextRank](https://web.eecs.umich.edu/%7Emihalcea/papers/mihalcea.emnlp04.pdf), [LexRank](https://jair.org/index.php/jair/article/view/10396)). TextRank's 2005 follow-up evaluated English and Portuguese ([paper](https://preview.aclanthology.org/fix_video/I05-2004.pdf)); no Spanish evaluation found.
- LSA: Gong and Liu pick one sentence per latent topic; Steinberger and Jezek allow several ([Steinberger](https://www.kiv.zcu.cz/~jstein/publikace/isim2004.pdf)).
- MMR and submodular selection: choose sentences by relevance minus similarity to what is already chosen, which controls redundancy ([MMR](https://people.eng.unimelb.edu.au/ammoffat/sigir98/abstracts/carbonell.html), [Lin and Bilmes](https://people.ece.uw.edu/bilmes/p/mypubs/lin2009-submodsum.pdf)). Greedy selection has a (1 - 1/e) guarantee for monotone submodular objectives; the sources disagree on one variant's ratio.
- BERTSUM (2019): a pretrained encoder plus a sentence classifier; needs training data ([paper](https://arxiv.org/pdf/1908.08345)).
- Sentence embeddings: paraphrase-multilingual-MiniLM covers 50+ languages ([guide](https://www.promptlayer.com/models/paraphrase-multilingual-minilm-l12-v2)); centroid summarization with multilingual embeddings exists ([NewSum 2023](https://aclanthology.org/2023.newsum-1.9.pdf)).

## Structure around the summary

- Topic segmentation: TextTiling splits text where lexical similarity drops ([Hearst 1994](https://arxiv.org/abs/cmp-lg/9406037)); embedding-based versions exist for meeting transcripts ([paper](https://arxiv.org/pdf/2106.12978)) and podcasts, where sentence duration and pauses did about as well as audio embeddings ([paper](https://arxiv.org/pdf/2307.13394)). One 2023 paper finds embedding baselines very sensitive to hyperparameters ([paper](https://arxiv.org/pdf/2308.10464)).
- Keywords: YAKE needs no dictionary or training and was compared with RAKE, TextRank and KeyBERT on Spanish among five languages ([Giarelis et al.](https://link.springer.com/chapter/10.1007/978-3-030-79150-6_50)); Spanish scores not retrieved.
- Names: spaCy Spanish small pipeline reports about 88.9 F-score over all entity types, not persons alone ([model](https://pypi.org/project/es-core-news-sm)).

## Neural but not LLM

- mT5 and mBART fine-tuned on Spanish MLSUM: about 29 ROUGE-1 ([Spanish seq2seq paper](https://arxiv.org/pdf/2309.11259)); mT5 on XL-Sum Spanish 30.93 ROUGE-1 ([XL-Sum](https://arxiv.org/pdf/2106.13822)). No ready Spanish MLSUM checkpoint found. Abstractive models of this kind produce unsupported content ([Maynez et al.](https://arxiv.org/pdf/2005.00661)).

## What bears on the risks

- Spoken text: extracted utterances keep disfluencies and ASR errors and read badly; abstraction was proposed to fix that, and users still found extractive summaries workable for browsing ([2016](https://arxiv.org/pdf/1609.07033), [2009](https://preview.aclanthology.org/fix_video/P09-2066.pdf), [Edinburgh study](https://www.cstr.ed.ac.uk/publications/users/s0345701_bib.html)).
- Evaluation: ROUGE is a weak proxy and Lead-3 is hard to beat on news because news puts the point first ([pointer-generator](https://arxiv.org/pdf/1704.04368), [Schluter](https://preview.aclanthology.org/naacl24-info/E17-2007)). A podcast has no lead, so a news benchmark says little here.
- Faithfulness: the search found nothing testing extractive output directly. Copied sentences cannot invent words, but what is chosen and left out can still mislead; that last point is my reasoning, not a source.
- Rust: no crate found that does TextRank sentence summarization. `keyword_extraction` has TextRank, TF-IDF, RAKE and YAKE for keywords (LGPL-3.0-or-later) ([docs](https://docs.rs/keyword_extraction)); `simple-summarize` is extractive with English and Russian only ([docs](https://docs.rs/simple-summarize)); `fastembed` wraps ONNX Runtime for embeddings ([docs](https://docs.rs/crate/fastembed/latest)).
- sumy offers Luhn, LSA, LexRank, TextRank, SumBasic, KL-Sum; Spanish support not confirmed in the results ([PyPI](https://pypi.org/project/sumy/)).
- Claim and argument extraction from podcast transcripts has been done with GPT-4 prompts, not with classic methods ([paper](https://ceur-ws.org/Vol-3438/paper_10.pdf)).

## Open

Not measured: how readable extracted sentences are from Whisper output of Ep. 34, and whether any method picks the passages a reader wants. Needs a run on Ep. 34 and a read by Felipe.
