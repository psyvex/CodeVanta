import assert from 'node:assert/strict';
import test from 'node:test';

import { toEngineEcosystemContext } from './bridge.js';

test('converts JavaScript ecosystem detection into engine context', () => {
  const context = toEngineEcosystemContext({
    language: 'typescript',
    runtime: 'nodejs',
    technologies: [
      {
        id: 'nestjs',
        confidence: 'definite',
        evidence: [{ kind: 'manifest', source: 'package.json', detail: 'dependency: @nestjs/core' }],
      },
      {
        id: 'typeorm',
        confidence: 'definite',
        evidence: [{ kind: 'manifest', source: 'package.json', detail: 'dependency: typeorm' }],
      },
    ],
  });

  assert.deepEqual(context, {
    runtime: 'nodejs',
    technologies: [
      {
        id: 'nestjs',
        confidence: 'definite',
        evidence: [{ kind: 'manifest', source: 'package.json', detail: 'dependency: @nestjs/core' }],
      },
      {
        id: 'typeorm',
        confidence: 'definite',
        evidence: [{ kind: 'manifest', source: 'package.json', detail: 'dependency: typeorm' }],
      },
    ],
  });
});

test('does not invent a runtime when detection did not provide one', () => {
  const context = toEngineEcosystemContext({
    language: 'javascript',
    technologies: [],
  });

  assert.deepEqual(context, { technologies: [] });
});
