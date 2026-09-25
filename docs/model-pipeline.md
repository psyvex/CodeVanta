# Model pipeline

This is the production pipeline for turning ingested repository history into a
released, locally-runnable CodeVanta model adapter, and for serving that
model to the CLI and editor extensions. It implements the direction set by
`README.md` and `docs/architecture.md`: adapt a small open base model per
ecosystem rather than train from scratch, keep weights out of Git, and let
deterministic analyzers supply evidence that the model reasons over.

## Stages

```text
1. ingest        mine: fetch merged GitHub PRs (files, review comments,
                  license) into RawPullRequest evidence
                  (ingestion/dataset-core/github-client.ts, mine.ts)
                  label: rule-based proposal of candidate ReviewSignals from
                  PR title/comment keywords, capped below "definite"
                  confidence (ingestion/dataset-core/propose-signals.ts,
                  label.ts)
                  review: a human edits the proposal file in place --
                  required, not optional; nothing is promoted un-reviewed
                  promote: joins the reviewed signals back with the raw
                  evidence into DatasetExample records (promote.ts,
                  raw-to-example.ts)
2. project        Flatten each DatasetExample's review signals into individual
                  training-example.schema.json records
                  (ingestion/dataset-core/project.ts)
3. curate         Validate against schema, drop unlicensed/duplicate records,
                  deterministically split train/val/test
                  (training/codevanta_training/dataset.py)
4. dataset manifest  Record split sizes, checksums, source manifest version
                  under datasets/ (data itself stays out of Git; datasets/generated
                  and datasets/raw are gitignored, only the manifest is committed)
5. train          LoRA/QLoRA fine-tune of the model manifest's base model
                  against the dataset manifest
                  (training/codevanta_training/train.py)
6. evaluate       Score the trained adapter against the held-out test split,
                  by review-taxonomy category, against the previous released
                  adapter (regression gate)
                  (training/codevanta_training/evaluate.py)
7. quantize/export  Export the merged adapter to GGUF for local CPU/GPU
                  inference via llama.cpp-compatible runtimes
8. release        Update the model manifest: status -> released, artifact
                  location + checksum, dataset/evaluation references
9. serve          Local inference server (Ollama or llama.cpp server) loads
                  the exported GGUF; the CLI and editor extensions talk to it
                  over the OpenAI-compatible HTTP API
```

Stages 1-4 and 9 are implemented as real, tested code in this repository
today. Stage 1's GitHub client is tested against mocked HTTP responses --
running it against a real external repository needs a GitHub token and
network access this sandboxed development session does not have for
arbitrary repositories. Stages 5-8 additionally require a GPU and
multi-gigabyte base-model downloads; they are implemented as real, tested
code (the training loop is proven mechanically against a tiny in-memory
model), meant to be run on infrastructure that has those resources.

## Model lifecycle

A model manifest's `status` field (`schemas/model-manifest.schema.json`)
tracks promotion through the pipeline:

```text
planned -> in-progress -> trained -> evaluated -> released
```

- `planned`: manifest exists, nothing has run yet.
- `in-progress`: a training run has started against a specific dataset manifest.
- `trained`: an adapter checkpoint exists but has not been evaluated.
- `evaluated`: the evaluation report exists and is attached via `evaluation`.
- `released`: the evaluation passed the regression gate against the previous
  released adapter (or there is none yet) and `artifact` is populated.

A manifest only advances a stage when the artifact for that stage exists and
is referenced (dataset manifest path, evaluation report path, or artifact
location + checksum). No stage is skipped.

## Reproducibility requirements

- Every training run records: base model id + revision, dataset manifest
  version, adapter hyperparameters, and a random seed, so a run can be
  reproduced from the manifest alone.
- Dataset splits are deterministic (seeded hash-based split on example id),
  so re-running `curate` on the same input produces the same split without
  storing the split itself.
- Evaluation always runs against the same held-out test split referenced by
  the dataset manifest, never a resplit.

## Serving to the CLI and editor extensions

`interfaces/cli`'s `review` command (`local_model.rs`) calls a local
OpenAI-compatible chat completions endpoint (the default local port for
Ollama or `llama-server`) for the reasoning layer, after the deterministic
analyzers have produced findings. This is decoupled from whether a
CodeVanta-trained adapter exists yet: any local model already running behind
that API (a released CodeVanta adapter, or a stock small code model) works,
and if none is reachable the deterministic findings still print with a
warning rather than the command failing. Editor extensions are future
`interfaces/` clients that reuse the same engine + local-model contract; they
are not implemented yet.
