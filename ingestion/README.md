# Ingestion

Source acquisition and normalization pipelines for Git repositories, GitHub pull requests, review comments, commits, diffs, ecosystem metadata, and license/provenance records.

Ingestion must preserve source identity and licensing metadata and must not silently treat unknown licensing as training-safe.

## GitHub pull request mining

`dataset-core/github-client.ts` is a GitHub REST API client that mines merged pull requests from a repository: changed file content (before/after) for a configurable set of extensions, review comments, and the repository's license (classified via `license-policy.ts`; only `MIT/Apache-2.0/BSD-2-Clause/BSD-3-Clause/ISC` are `allowed`, everything else -- including unknown -- is `review-required` or `blocked`).

`mine.ts`'s `--language` flag selects which extensions to mine and defaults to `javascript-typescript`; pass `--language python` to mine `.py`/`.pyi` files instead (`github-types.ts#MINEABLE_LANGUAGE_EXTENSIONS`). `raw-to-example.ts` infers each `DatasetExample`'s `language` field from the mined files themselves, so nothing downstream needs to know which mining run produced them.

It deliberately stops at raw evidence (`RawPullRequest`, in `github-types.ts`): it does **not** infer a finding's `category`/`severity`/`confidence` from a review comment's free text. Doing so would be fabricating training labels, not extracting them.

## Labeling: mine -> label -> review -> promote

Turning mined evidence into labeled `ReviewSignal`s is a separate, human-in-the-loop pipeline of four steps:

```sh
# 1. Mine merged PRs into raw evidence (needs GITHUB_TOKEN + network access to the repo)
GITHUB_TOKEN=... node --import tsx mine.ts --repo owner/name --since 2024-01-01 --out raw.jsonl
# ... or, for a Python repository:
GITHUB_TOKEN=... node --import tsx mine.ts --repo owner/name --language python --out raw.jsonl

# 2. Propose candidate signals from PR titles and review comments (rule-based, offline)
node --import tsx label.ts --in raw.jsonl --out review.jsonl

# 3. A human opens review.jsonl and edits it in place: delete wrong proposals,
#    correct category/severity/confidence/summary on the ones kept. This step
#    happens outside any script.

# 4. Join the edited review file back with the raw evidence into DatasetExamples
node --import tsx promote.ts --raw raw.jsonl --review review.jsonl --out examples.jsonl
```

`propose-signals.ts`'s rules only match PR-title and review-comment keywords (security/bug/performance/testing families) -- they never read the code diff, so `confidence` is always capped below `definite` and every proposal carries `rule` + `evidence` so a reviewer can check it against its source in seconds. There is no automatic promotion path: `promote.ts` only accepts what survived a human's edit of `review.jsonl` (a pull request with zero remaining proposals is skipped entirely). An LLM-assisted proposer could replace or augment `propose-signals.ts`'s heuristics later, but the human-review step in the middle does not go away -- it is what keeps a proposal from becoming a fabricated label.

`examples.jsonl` (`DatasetExample` records) is what `dataset-core/project.ts` flattens into `training-example.schema.json` records, which `training/codevanta_training/dataset.py` then validates and splits.

Running steps 1 and 2 against a real repository requires network access to the GitHub API and a token with read access to it -- neither of which this development sandbox has for arbitrary external repositories.
