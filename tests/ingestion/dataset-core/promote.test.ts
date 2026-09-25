import assert from 'node:assert/strict';
import test from 'node:test';
import { promote, parseArgs } from '../../../ingestion/dataset-core/promote.js';
import { buildReviewFile } from '../../../ingestion/dataset-core/label.js';
import type { RawPullRequest } from '../../../ingestion/dataset-core/github-types.js';

function rawPullRequest(overrides: Partial<RawPullRequest> = {}): RawPullRequest {
  return {
    host: 'github',
    repository: 'acme/widgets',
    pullNumber: 7,
    title: 'Fix security vulnerability',
    baseSha: 'base',
    mergeCommitSha: 'merge',
    sourceUrl: 'https://github.com/acme/widgets/pull/7',
    mergedAt: '2025-06-01T00:00:00Z',
    licenseSpdx: 'MIT',
    licenseStatus: 'allowed',
    files: [
      {
        path: 'src/app.ts',
        status: 'modified',
        additions: 1,
        deletions: 1,
        before: 'before',
        after: 'after',
      },
    ],
    reviewComments: [],
    collectedAt: '2025-06-02T00:00:00Z',
    ...overrides,
  };
}

test('parseArgs requires all three paths', () => {
  assert.throws(() => parseArgs(['--raw', 'raw.jsonl', '--review', 'review.jsonl']), /--raw/);
});

test('promote joins reviewed signals back with raw evidence', () => {
  const raw = rawPullRequest();
  const reviewFile = buildReviewFile([raw]);

  const examples = promote([raw], reviewFile);

  assert.equal(examples.length, 1);
  assert.equal(examples[0]?.signals[0]?.category, 'security');
  assert.equal(examples[0]?.changes[0]?.path, 'src/app.ts');
  // The promoted signal must not carry the heuristic's own bookkeeping.
  assert.equal('rule' in (examples[0]?.signals[0] ?? {}), false);
});

test('promote skips a pull request with no reviewed proposals', () => {
  const raw = rawPullRequest({ title: 'Update dependencies' });
  const reviewFile = buildReviewFile([raw]);

  const examples = promote([raw], reviewFile);

  assert.deepEqual(examples, []);
});

test('promote skips a pull request whose reviewer deleted every proposal', () => {
  const raw = rawPullRequest();
  const reviewFile = buildReviewFile([raw]).map((entry) => ({ ...entry, proposals: [] }));

  const examples = promote([raw], reviewFile);

  assert.deepEqual(examples, []);
});

test('promote ignores raw pull requests that have no matching review entry', () => {
  const reviewed = rawPullRequest({ pullNumber: 1 });
  const unreviewed = rawPullRequest({ pullNumber: 2 });
  const reviewFile = buildReviewFile([reviewed]);

  const examples = promote([reviewed, unreviewed], reviewFile);

  assert.equal(examples.length, 1);
});
