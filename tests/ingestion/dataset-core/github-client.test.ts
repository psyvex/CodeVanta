import assert from 'node:assert/strict';
import test from 'node:test';
import { GitHubClient } from '../../../ingestion/dataset-core/github-client.js';
import { PYTHON_EXTENSIONS } from '../../../ingestion/dataset-core/github-types.js';

type Route = (url: URL) => Response;

function fakeFetch(routes: Record<string, Route>): typeof fetch {
  return (async (input: RequestInfo | URL) => {
    const url = new URL(typeof input === 'string' ? input : input.toString());
    const route = routes[url.pathname];
    if (!route) {
      throw new Error(`unexpected request: ${url.pathname}`);
    }
    return route(url);
  }) as typeof fetch;
}

function json(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { 'content-type': 'application/json' },
    ...init,
  });
}

test('getRepositoryLicense classifies an allowed license', async () => {
  const client = new GitHubClient({
    token: 'x',
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/license': () => json({ license: { spdx_id: 'MIT' } }),
    }),
  });

  const license = await client.getRepositoryLicense('acme', 'widgets');
  assert.deepEqual(license, { spdxId: 'MIT', status: 'allowed' });
});

test('getRepositoryLicense treats a missing license file as unknown', async () => {
  const client = new GitHubClient({
    token: 'x',
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/license': () => new Response(null, { status: 404 }),
    }),
  });

  const license = await client.getRepositoryLicense('acme', 'widgets');
  assert.deepEqual(license, { spdxId: null, status: 'unknown' });
});

test('listMergedPullRequests only returns merged PRs and stops at the since cutoff', async () => {
  const client = new GitHubClient({
    token: 'x',
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/pulls': () =>
        json([
          {
            number: 3,
            title: 'Recent merged PR',
            merged_at: '2025-06-01T00:00:00Z',
            merge_commit_sha: 'sha3',
            updated_at: '2025-06-01T00:00:00Z',
            base: { sha: 'base3' },
          },
          {
            number: 2,
            title: 'Still open, not merged',
            merged_at: null,
            merge_commit_sha: null,
            updated_at: '2025-05-15T00:00:00Z',
            base: { sha: 'base2' },
          },
          {
            number: 1,
            title: 'Too old',
            merged_at: '2025-01-01T00:00:00Z',
            merge_commit_sha: 'sha1',
            updated_at: '2025-01-01T00:00:00Z',
            base: { sha: 'base1' },
          },
        ]),
    }),
  });

  const pulls = await client.listMergedPullRequests('acme', 'widgets', {
    since: new Date('2025-03-01T00:00:00Z'),
  });

  assert.deepEqual(
    pulls.map((pull) => pull.number),
    [3],
  );
});

test('getFileContent decodes base64 content and returns null for a missing file', async () => {
  const client = new GitHubClient({
    token: 'x',
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/contents/src/index.ts': (url) => {
        if (url.searchParams.get('ref') === 'missing') {
          return new Response(null, { status: 404 });
        }
        return json({
          encoding: 'base64',
          size: 11,
          content: Buffer.from('const a=1;').toString('base64'),
        });
      },
    }),
  });

  const content = await client.getFileContent('acme', 'widgets', 'src/index.ts', 'sha1');
  assert.equal(content, 'const a=1;');

  const missing = await client.getFileContent('acme', 'widgets', 'src/index.ts', 'missing');
  assert.equal(missing, null);
});

test('getFileContent drops content larger than the configured limit', async () => {
  const client = new GitHubClient({
    token: 'x',
    maxFileBytes: 10,
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/contents/src/big.ts': () =>
        json({ encoding: 'base64', size: 1_000_000, content: '' }),
    }),
  });

  const content = await client.getFileContent('acme', 'widgets', 'src/big.ts', 'sha1');
  assert.equal(content, null);
});

test('minePullRequest only fetches content for JS/TS files and attaches review comments and license', async () => {
  const client = new GitHubClient({
    token: 'x',
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/pulls/7/files': () =>
        json([
          { filename: 'src/app.ts', status: 'modified', additions: 2, deletions: 1 },
          { filename: 'README.md', status: 'modified', additions: 5, deletions: 0 },
        ]),
      '/repos/acme/widgets/pulls/7/comments': () =>
        json([{ path: 'src/app.ts', line: 4, body: 'Consider parameterizing this query.', user: { login: 'reviewer' } }]),
      '/repos/acme/widgets/license': () => json({ license: { spdx_id: 'Apache-2.0' } }),
      '/repos/acme/widgets/contents/src/app.ts': (url) =>
        json({
          encoding: 'base64',
          size: 8,
          content: Buffer.from(url.searchParams.get('ref') === 'base' ? 'before' : 'after1').toString('base64'),
        }),
    }),
  });

  const raw = await client.minePullRequest('acme', 'widgets', {
    number: 7,
    title: 'Fix query',
    mergeCommitSha: 'merge',
    baseSha: 'base',
    mergedAt: '2025-06-01T00:00:00Z',
  });

  assert.equal(raw.files.length, 1);
  assert.equal(raw.files[0]?.path, 'src/app.ts');
  assert.equal(raw.files[0]?.before, 'before');
  assert.equal(raw.files[0]?.after, 'after1');
  assert.equal(raw.reviewComments.length, 1);
  assert.equal(raw.licenseSpdx, 'Apache-2.0');
  assert.equal(raw.licenseStatus, 'allowed');
});

test('minePullRequest respects a configured fileExtensions set', async () => {
  const client = new GitHubClient({
    token: 'x',
    fileExtensions: PYTHON_EXTENSIONS,
    fetchImpl: fakeFetch({
      '/repos/acme/widgets/pulls/7/files': () =>
        json([
          { filename: 'src/app.py', status: 'modified', additions: 2, deletions: 1 },
          { filename: 'src/app.ts', status: 'modified', additions: 3, deletions: 0 },
        ]),
      '/repos/acme/widgets/pulls/7/comments': () => json([]),
      '/repos/acme/widgets/license': () => json({ license: { spdx_id: 'MIT' } }),
      '/repos/acme/widgets/contents/src/app.py': (url) =>
        json({
          encoding: 'base64',
          size: 8,
          content: Buffer.from(url.searchParams.get('ref') === 'base' ? 'before' : 'after').toString('base64'),
        }),
    }),
  });

  const raw = await client.minePullRequest('acme', 'widgets', {
    number: 7,
    title: 'Fix query',
    mergeCommitSha: 'merge',
    baseSha: 'base',
    mergedAt: '2025-06-01T00:00:00Z',
  });

  assert.equal(raw.files.length, 1);
  assert.equal(raw.files[0]?.path, 'src/app.py');
});
