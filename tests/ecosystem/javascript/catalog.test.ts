import assert from 'node:assert/strict';
import test from 'node:test';
import { isKnownEcosystemValue, NODE_ECOSYSTEM } from '../../../ecosystem/javascript/catalog.js';

test('contains the initial Node.js ecosystem scope', () => {
  assert.ok(NODE_ECOSYSTEM.languages.includes('typescript'));
  assert.ok(NODE_ECOSYSTEM.frameworks.includes('nestjs'));
  assert.ok(NODE_ECOSYSTEM.orms.includes('typeorm'));
  assert.ok(NODE_ECOSYSTEM.orms.includes('prisma'));
  assert.ok(NODE_ECOSYSTEM.databases.includes('postgresql'));
});

test('recognizes catalog values', () => {
  assert.equal(isKnownEcosystemValue('nestjs'), true);
  assert.equal(isKnownEcosystemValue('unknown-framework'), false);
});
