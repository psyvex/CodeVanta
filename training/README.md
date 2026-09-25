# Training

Reproducible training and fine-tuning pipelines for language and ecosystem-specific CodeVanta models. See `docs/model-pipeline.md` for the full pipeline this package implements.

Training jobs consume versioned dataset manifests and record base model, adapter, hyperparameters, hardware, and resulting artifact metadata.

## Modules

- `codevanta_training/manifest.py` -- loads `models/registry.json` and validates every model manifest against `schemas/model-manifest.schema.json`.
- `codevanta_training/dataset.py` -- loads training-example JSONL, validates against `configs/training-example.schema.json`, deduplicates, deterministically splits train/val/test by content hash, and writes a `schemas/dataset-manifest.schema.json` manifest. The examples themselves are never committed; only the manifest is.
- `codevanta_training/prompting.py` -- the single prompt/target format shared by training and evaluation, so they cannot drift apart.
- `codevanta_training/train.py` -- LoRA fine-tuning of a model manifest's base model against a dataset manifest's train split (`python -m codevanta_training.train --model-manifest ... --train-examples ... --output-dir ...`).
- `codevanta_training/evaluate.py` -- scores a trained adapter's generated findings against a held-out test split by review-taxonomy category (precision/recall/F1), and gates release on no regression versus the previous evaluation report.

## Running

```sh
pip install -e .[train]   # torch, transformers, peft, accelerate, tokenizers
pytest -q tests
```

The base `pip install -e .` (no extra) and default `pytest -q tests` only need `jsonschema` and run in seconds; tests that need a model self-skip. The full suite proves the LoRA training loop actually reduces loss, using a tiny model and tokenizer built from scratch locally -- no network or GPU required to verify the mechanics. Training against a real base model additionally needs network access to the model's hub repository and, for anything beyond a toy dataset, a GPU.
