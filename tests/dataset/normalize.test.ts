import assert from 'node:assert/strict';
import test from 'node:test';
import { normalizeExample, normalizeReviewSignals } from '../../src/dataset/normalize.js';
import type { DatasetExample } from '../../src/dataset/types.js';

test('normalizes and deduplicates review signals', () => {
  const result = normalizeReviewSignals([
    { category: 'security', confidence: 'likely', summary: '  SQL injection  ' },
    { category: 'security', confidence: 'likely', summary: 'SQL injection' },
    { category: 'bug', confidence: 'possible', summary: 'Null handling' },
  ]);

  assert.equal(result.length, 2);
  assert.equal(result[0]?.summary, 'SQL injection');
});

test('normalizes ecosystem metadata and provenance', () => {
  const example: DatasetExample = {
    id: 'example',
    language: 'typescript',
    frameworks: ['nestjs', 'nestjs', ' '],
    libraries: ['typeorm'],
    databases: ['postgresql'],
    tools: ['eslint'],
    changes: [],
    signals: [{ category: 'standards', confidence: 'suggestion', summary: '  Prefer explicit return types  ' }],
    provenance: {
      host: 'github',
      repository: '  example/repo  ',
      commitSha: 'abc',
      sourceUrl: ' https://github.com/example/repo/commit/abc ',
      licenseStatus: 'allowed',
      collectedAt: '2025-01-01T00:00:00Z',
    },
  };

  const normalized = normalizeExample(example);
  assert.deepEqual(normalized.frameworks, ['nestjs']);
  assert.equal(normalized.provenance.repository, 'example/repo');
});
