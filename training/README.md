# Training

Reproducible training and fine-tuning pipelines for language and ecosystem-specific CodeVanta models. See `docs/model-pipeline.md` for the full pipeline this package implements.

Training jobs consume versioned dataset manifests and record base model, adapter, hyperparameters, hardware, and resulting artifact metadata.

## Modules

- `codevanta_training/manifest.py` -- loads `models/registry.json` and validates every model manifest against `schemas/model-manifest.schema.json`.
- `codevanta_training/dataset.py` -- loads training-example JSONL, validates against `configs/training-example.schema.json`, deduplicates, deterministically splits train/val/test by content hash, and writes a `schemas/dataset-manifest.schema.json` manifest. The examples themselves are never committed; only the manifest is.
- `codevanta_training/prompting.py` -- the single prompt/target format shared by training and evaluation, so they cannot drift apart.
- `codevanta_training/train.py` -- LoRA fine-tuning of a model manifest's base model against a dataset manifest's train split (`python -m codevanta_training.train --model-manifest ... --train-examples ... --output-dir ...`).
- `codevanta_training/evaluate.py` -- scores a trained adapter's generated findings against a held-out test split by review-taxonomy category (precision/recall/F1), and gates release on no regression versus the previous evaluation report.
- `codevanta_training/export.py` -- merges a trained LoRA adapter into its base model (`merge_adapter`), then converts the merged model to GGUF (`convert_to_gguf`) by shelling out to llama.cpp's own `convert_hf_to_gguf.py` rather than reimplementing a GGUF writer.

## Running

```sh
pip install -e .[train]   # torch, transformers, peft, accelerate, tokenizers
pytest -q tests
```

The base `pip install -e .` (no extra) and default `pytest -q tests` only need `jsonschema` and run in seconds; tests that need a model self-skip. The full suite proves the LoRA training loop actually reduces loss, using a tiny model and tokenizer built from scratch locally -- no network or GPU required to verify the mechanics. Training against a real base model additionally needs network access to the model's hub repository and, for anything beyond a toy dataset, a GPU.

### GGUF export

`convert_to_gguf`'s subprocess plumbing is unit-tested with a mocked subprocess by default. To exercise it against the real tool:

```sh
pip install -e .[train,export]   # + gguf, sentencepiece, protobuf
git clone https://github.com/ggml-org/llama.cpp /path/to/llama.cpp
LLAMA_CPP_CONVERT_SCRIPT=/path/to/llama.cpp/convert_hf_to_gguf.py pytest -q tests/test_export.py
```

That live test documents a real, current limitation rather than hiding it: llama.cpp only converts tokenizers it recognizes by hashing their vocabulary against a table of known real tokenizers, so it's expected to reject a from-scratch synthetic tokenizer (like the test fixtures') at the vocabulary stage, *after* tensor conversion has already succeeded -- a real base model's own downloaded tokenizer converts cleanly. Pin `transformers` to the version in llama.cpp's own `requirements/requirements-convert_hf_to_gguf.txt` for a real export; a newer `transformers` can drop legacy config fields (e.g. GPT-2's `n_ctx`) some of llama.cpp's per-architecture converters still read.
