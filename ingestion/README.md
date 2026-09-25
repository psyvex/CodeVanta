# Ingestion

Source acquisition and normalization pipelines for Git repositories, GitHub pull requests, review comments, commits, diffs, ecosystem metadata, and license/provenance records.

Ingestion must preserve source identity and licensing metadata and must not silently treat unknown licensing as training-safe.

## GitHub pull request mining

`dataset-core/github-client.ts` is a GitHub REST API client that mines merged pull requests from a repository: changed JS/TS file content (before/after), review comments, and the repository's license (classified via `license-policy.ts`; only `MIT/Apache-2.0/BSD-2-Clause/BSD-3-Clause/ISC` are `allowed`, everything else -- including unknown -- is `review-required` or `blocked`).

It deliberately stops at raw evidence (`RawPullRequest`, in `github-types.ts`): it does **not** infer a finding's `category`/`severity`/`confidence` from a review comment's free text. Doing so would be fabricating training labels, not extracting them. Turning mined evidence into labeled `ReviewSignal`s is a separate step that a human curates, or that an LLM proposes and a human explicitly reviews before it's accepted -- `raw-to-example.ts#toDatasetExample` is where curated signals and mined evidence are combined into a `DatasetExample`, which `dataset-core/project.ts` then flattens into `training-example.schema.json` records.

Run the miner directly against a repository:

```sh
GITHUB_TOKEN=... node --import tsx mine.ts --repo owner/name --since 2024-01-01 --out raw.jsonl
```

This requires network access to the GitHub API and a token with read access to the target repository -- neither of which this development sandbox has for arbitrary external repositories.
