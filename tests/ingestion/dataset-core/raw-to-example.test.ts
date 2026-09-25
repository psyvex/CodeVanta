import assert from 'node:assert/strict';
import test from 'node:test';
import { toDatasetExample } from '../../../ingestion/dataset-core/raw-to-example.js';
import type { RawPullRequest } from '../../../ingestion/dataset-core/github-types.js';

function rawPullRequest(overrides: Partial<RawPullRequest> = {}): RawPullRequest {
  return {
    host: 'github',
    repository: 'acme/widgets',
    pullNumber: 7,
    title: 'Fix interpolated query',
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
        additions: 2,
        deletions: 1,
        before: 'const q = `SELECT * FROM users WHERE id = ${id}`;',
        after: 'const q = sql`SELECT * FROM users WHERE id = ${id}`;',
      },
    ],
    reviewComments: [{ path: 'src/app.ts', line: 1, body: 'Use a parameterized query.', author: 'reviewer' }],
    collectedAt: '2025-06-02T00:00:00Z',
    ...overrides,
  };
}

test('combines a raw pull request with curated signals into a DatasetExample', () => {
  const example = toDatasetExample(rawPullRequest(), [
    {
      category: 'security',
      confidence: 'likely',
      severity: 'high',
      summary: 'Interpolated SQL query',
      reviewerComment: 'Use a parameterized query.',
    },
  ]);

  assert.equal(example.language, 'typescript');
  assert.equal(example.changes.length, 1);
  assert.equal(example.provenance.commitSha, 'merge');
  assert.equal(example.provenance.licenseSpdx, 'MIT');
  assert.equal(example.signals.length, 1);
});

test('drops files whose before or after content could not be fetched', () => {
  const raw = rawPullRequest({
    files: [
      { path: 'src/app.ts', status: 'added', additions: 3, deletions: 0, before: null, after: 'new file' },
      { path: 'src/old.ts', status: 'removed', additions: 0, deletions: 3, before: 'old file', after: null },
      {
        path: 'src/kept.ts',
        status: 'modified',
        additions: 1,
        deletions: 1,
        before: 'before',
        after: 'after',
      },
    ],
  });

  const example = toDatasetExample(raw, []);

  assert.deepEqual(
    example.changes.map((change) => change.path),
    ['src/kept.ts'],
  );
});

test('infers javascript when no TypeScript files are present', () => {
  const raw = rawPullRequest({
    files: [
      { path: 'src/app.js', status: 'modified', additions: 1, deletions: 1, before: 'a', after: 'b' },
    ],
  });

  const example = toDatasetExample(raw, []);
  assert.equal(example.language, 'javascript');
});

test('infers python from .py files', () => {
  const raw = rawPullRequest({
    files: [
      { path: 'src/app.py', status: 'modified', additions: 1, deletions: 1, before: 'a', after: 'b' },
    ],
  });

  const example = toDatasetExample(raw, []);
  assert.equal(example.language, 'python');
});

test('omits licenseSpdx from provenance when the license could not be determined', () => {
  const raw = rawPullRequest({ licenseSpdx: null, licenseStatus: 'unknown' });

  const example = toDatasetExample(raw, []);

  assert.equal('licenseSpdx' in example.provenance, false);
});
