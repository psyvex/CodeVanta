import assert from 'node:assert/strict';
import test from 'node:test';
import { proposeSignals } from '../../../ingestion/dataset-core/propose-signals.js';
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

test('proposes a security signal from a security-flavored PR title', () => {
  const proposals = proposeSignals(rawPullRequest({ title: 'Fix SQL injection vulnerability' }));

  const security = proposals.find((p) => p.category === 'security');
  assert.ok(security);
  assert.equal(security.confidence, 'possible');
  assert.equal(security.rule.startsWith('title-keyword:'), true);
});

test('proposes a bug signal from a fix-flavored PR title', () => {
  const proposals = proposeSignals(rawPullRequest({ title: 'Fix null pointer crash on login' }));

  assert.ok(proposals.some((p) => p.category === 'bug'));
});

test('proposes nothing from a title with no matching keywords', () => {
  const proposals = proposeSignals(rawPullRequest({ title: 'Update dependencies' }));
  assert.deepEqual(proposals, []);
});

test('review comments produce higher-confidence proposals than titles', () => {
  const proposals = proposeSignals(
    rawPullRequest({
      reviewComments: [
        { path: 'src/db.ts', line: 10, body: 'This looks vulnerable to sql injection.', author: 'reviewer' },
      ],
    }),
  );

  const security = proposals.find((p) => p.category === 'security');
  assert.ok(security);
  assert.equal(security.confidence, 'likely');
  assert.equal(security.reviewerComment, 'This looks vulnerable to sql injection.');
});

test('deduplicates identical proposals from title and comments', () => {
  const proposals = proposeSignals(
    rawPullRequest({
      title: 'Fix security issue',
      reviewComments: [{ path: null, line: null, body: 'Fix security issue', author: 'reviewer' }],
    }),
  );

  // Title and comment produce distinct rules (title-keyword vs review-comment-keyword)
  // even with the same text, so both are kept -- but each rule only fires once.
  const securityProposals = proposals.filter((p) => p.category === 'security');
  assert.equal(securityProposals.length, 2);
  assert.notEqual(securityProposals[0]?.rule, securityProposals[1]?.rule);
});

test('never produces definite confidence', () => {
  const proposals = proposeSignals(
    rawPullRequest({
      title: 'Fix critical security vulnerability CVE-2024-0001',
      reviewComments: [{ path: null, line: null, body: 'confirmed security exploit', author: 'reviewer' }],
    }),
  );

  assert.ok(proposals.length > 0);
  assert.ok(proposals.every((p) => p.confidence !== 'definite'));
});
