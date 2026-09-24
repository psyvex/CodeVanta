import assert from 'node:assert/strict';
import test from 'node:test';
import { detectJavaScriptEcosystemFromManifest } from '../../../ecosystems/javascript/src/detector.js';

test('detects known Node.js technologies from package metadata', () => {
  const context = detectJavaScriptEcosystemFromManifest({
    dependencies: {
      '@nestjs/common': '^11.0.0',
      typeorm: '^0.3.0',
      pg: '^8.0.0',
    },
    devDependencies: {
      typescript: '^5.0.0',
    },
  });

  assert.equal(context.language, 'typescript');
  assert.equal(context.runtime, 'nodejs');
  assert.deepEqual(
    context.technologies.map((technology) => technology.id),
    ['nestjs', 'typeorm'],
  );
});

test('ignores packages outside the known ecosystem catalog', () => {
  const context = detectJavaScriptEcosystemFromManifest({
    dependencies: {
      'unknown-framework': '^1.0.0',
    },
  });

  assert.deepEqual(context.technologies, []);
});
