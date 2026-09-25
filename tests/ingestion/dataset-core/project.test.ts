import assert from 'node:assert/strict';
import test from 'node:test';
import { projectTrainingExamples } from '../../../ingestion/dataset-core/project.js';
import type { DatasetExample } from '../../../ingestion/dataset-core/types.js';

function example(overrides: Partial<DatasetExample> = {}): DatasetExample {
  return {
    id: 'example',
    language: 'typescript',
    frameworks: ['nestjs'],
    libraries: ['typeorm'],
    databases: ['postgresql'],
    tools: ['eslint'],
    changes: [
      {
        path: 'src/users.service.ts',
        before: 'old',
        after: 'const q = `SELECT * FROM users WHERE id = ${id}`;',
        additions: 1,
        deletions: 1,
      },
    ],
    signals: [
      {
        category: 'security',
        confidence: 'likely',
        severity: 'high',
        summary: 'Interpolated SQL query',
        reviewerComment: 'Use a parameterized query instead.',
      },
    ],
    provenance: {
      host: 'github',
      repository: 'example/repo',
      commitSha: 'abc123',
      sourceUrl: 'https://github.com/example/repo/commit/abc123',
      licenseSpdx: 'MIT',
      licenseStatus: 'allowed',
      collectedAt: '2025-01-01T00:00:00Z',
    },
    ...overrides,
  };
}

test('projects one training example per review signal', () => {
  const [projected] = projectTrainingExamples(example());

  assert.ok(projected);
  assert.equal(projected.language, 'typescript');
  assert.equal(projected.framework, 'nestjs');
  assert.equal(projected.finding.category, 'security');
  assert.equal(projected.finding.severity, 'high');
  assert.match(projected.code, /src\/users\.service\.ts/);
  assert.equal(projected.provenance.source_type, 'commit');
  assert.equal(projected.provenance.commit, 'abc123');
});

test('never projects examples with unresolved licensing', () => {
  for (const licenseStatus of ['unknown', 'review-required', 'blocked'] as const) {
    const projected = projectTrainingExamples(
      example({ provenance: { ...example().provenance, licenseStatus } }),
    );
    assert.deepEqual(projected, []);
  }
});

test('skips examples with no renderable code change', () => {
  const projected = projectTrainingExamples(example({ changes: [] }));
  assert.deepEqual(projected, []);
});
