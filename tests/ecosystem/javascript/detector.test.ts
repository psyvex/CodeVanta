import assert from 'node:assert/strict';
import test from 'node:test';
import { detectJavaScriptEcosystemFromManifest } from '../../../ecosystems/javascript/src/detector.js';

test('detects TypeScript, Node.js, NestJS, and TypeORM from package metadata', () => {
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
  assert.equal(context.technologies[0]?.confidence, 'definite');
  assert.equal(context.technologies[0]?.evidence[0]?.source, 'package.json');
});
