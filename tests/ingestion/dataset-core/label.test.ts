import assert from 'node:assert/strict';
import test from 'node:test';
import { buildReviewFile, parseArgs } from '../../../ingestion/dataset-core/label.js';
import type { RawPullRequest } from '../../../ingestion/dataset-core/github-types.js';

function rawPullRequest(overrides: Partial<RawPullRequest> = {}): RawPullRequest {
  return {
    host: 'github',
    repository: 'acme/widgets',
    pullNumber: 7,
    title: 'Update dependencies',
    baseSha: 'base',
    mergeCommitSha: 'merge',
    sourceUrl: 'https://github.com/acme/widgets/pull/7',
    mergedAt: '2025-06-01T00:00:00Z',
    licenseSpdx: 'MIT',
    licenseStatus: 'allowed',
    files: [],
    reviewComments: [],
    collectedAt: '2025-06-02T00:00:00Z',
    ...overrides,
  };
}

test('parseArgs requires --in and --out', () => {
  assert.throws(() => parseArgs(['--in', 'raw.jsonl']), /--in/);
  assert.throws(() => parseArgs([]), /--in/);
});

test('parseArgs parses both paths', () => {
  const options = parseArgs(['--in', 'raw.jsonl', '--out', 'review.jsonl']);
  assert.deepEqual(options, { inPath: 'raw.jsonl', outPath: 'review.jsonl' });
});

test('buildReviewFile only includes pull requests with at least one proposal', () => {
  const withSignal = rawPullRequest({ pullNumber: 1, title: 'Fix security vulnerability' });
  const withoutSignal = rawPullRequest({ pullNumber: 2, title: 'Update dependencies' });

  const reviewFile = buildReviewFile([withSignal, withoutSignal]);

  assert.equal(reviewFile.length, 1);
  assert.equal(reviewFile[0]?.pullNumber, 1);
  assert.ok(reviewFile[0]?.proposals.length ?? 0 > 0);
});

test('buildReviewFile carries enough context to find the PR without the raw evidence', () => {
  const [entry] = buildReviewFile([rawPullRequest({ title: 'Fix security bug' })]);

  assert.equal(entry?.repository, 'acme/widgets');
  assert.equal(entry?.sourceUrl, 'https://github.com/acme/widgets/pull/7');
});
