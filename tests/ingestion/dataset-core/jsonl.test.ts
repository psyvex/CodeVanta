import assert from 'node:assert/strict';
import test from 'node:test';
import { parseDataset, serializeDataset } from '../../../ingestion/dataset-core/jsonl.js';
import type { DatasetExample } from '../../../ingestion/dataset-core/types.js';

const example: DatasetExample = {
  id: 'example',
  language: 'typescript',
  frameworks: ['nestjs'],
  libraries: ['typeorm'],
  databases: ['postgresql'],
  tools: ['eslint'],
  changes: [],
  signals: [],
  provenance: {
    host: 'github',
    repository: 'example/repo',
    commitSha: 'abc',
    sourceUrl: 'https://github.com/example/repo/commit/abc',
    licenseStatus: 'allowed',
    collectedAt: '2025-01-01T00:00:00Z',
  },
};

test('round trips dataset examples as JSONL', () => {
  const serialized = serializeDataset([example]);
  assert.equal(serialized.endsWith('\n'), true);
  assert.deepEqual(parseDataset(serialized), [example]);
});

test('reports invalid JSONL line numbers', () => {
  assert.throws(() => parseDataset('{"id":"ok"}\nnot-json\n'), /line 2/);
});
