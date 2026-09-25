import assert from 'node:assert/strict';
import test from 'node:test';
import { mine, parseArgs } from '../../../ingestion/dataset-core/mine.js';
import { GitHubClient } from '../../../ingestion/dataset-core/github-client.js';

test('parseArgs requires --repo in owner/name form', () => {
  assert.throws(() => parseArgs(['--out', 'x.jsonl']), /--repo/);
  assert.throws(() => parseArgs(['--repo', 'not-a-repo', '--out', 'x.jsonl']), /--repo/);
});

test('parseArgs requires --out', () => {
  assert.throws(() => parseArgs(['--repo', 'acme/widgets']), /--out/);
});

test('parseArgs parses owner/repo, since, and max-pages', () => {
  const options = parseArgs([
    '--repo',
    'acme/widgets',
    '--since',
    '2024-01-01',
    '--max-pages',
    '3',
    '--out',
    'raw.jsonl',
  ]);

  assert.equal(options.owner, 'acme');
  assert.equal(options.repo, 'widgets');
  assert.equal(options.maxPages, 3);
  assert.equal(options.out, 'raw.jsonl');
  assert.equal(options.since?.toISOString().startsWith('2024-01-01'), true);
});

test('mine lists merged pull requests then mines each one', async () => {
  const requestedPaths: string[] = [];
  const fetchImpl = (async (input: RequestInfo | URL) => {
    const url = new URL(typeof input === 'string' ? input : input.toString());
    requestedPaths.push(url.pathname);

    if (url.pathname === '/repos/acme/widgets/pulls') {
      return new Response(
        JSON.stringify([
          {
            number: 1,
            title: 'Fix bug',
            merged_at: '2025-01-01T00:00:00Z',
            merge_commit_sha: 'sha1',
            updated_at: '2025-01-01T00:00:00Z',
            base: { sha: 'base1' },
          },
        ]),
        { status: 200, headers: { 'content-type': 'application/json' } },
      );
    }
    if (url.pathname === '/repos/acme/widgets/pulls/1/files') {
      return new Response(JSON.stringify([]), { status: 200 });
    }
    if (url.pathname === '/repos/acme/widgets/pulls/1/comments') {
      return new Response(JSON.stringify([]), { status: 200 });
    }
    if (url.pathname === '/repos/acme/widgets/license') {
      return new Response(JSON.stringify({ license: { spdx_id: 'MIT' } }), { status: 200 });
    }
    throw new Error(`unexpected request: ${url.pathname}`);
  }) as typeof fetch;

  const client = new GitHubClient({ token: 'x', fetchImpl });
  const results = await mine(client, {
    owner: 'acme',
    repo: 'widgets',
    language: 'javascript-typescript',
    maxPages: 1,
    out: 'x.jsonl',
  });

  assert.equal(results.length, 1);
  assert.equal(results[0]?.pullNumber, 1);
  assert.equal(results[0]?.licenseStatus, 'allowed');
  assert.ok(requestedPaths.includes('/repos/acme/widgets/pulls/1/files'));
});

test('parseArgs defaults to javascript-typescript and accepts python', () => {
  const defaulted = parseArgs(['--repo', 'acme/widgets', '--out', 'raw.jsonl']);
  assert.equal(defaulted.language, 'javascript-typescript');

  const python = parseArgs(['--repo', 'acme/widgets', '--language', 'python', '--out', 'raw.jsonl']);
  assert.equal(python.language, 'python');
});

test('parseArgs rejects an unsupported language', () => {
  assert.throws(
    () => parseArgs(['--repo', 'acme/widgets', '--language', 'ruby', '--out', 'raw.jsonl']),
    /--language/,
  );
});
