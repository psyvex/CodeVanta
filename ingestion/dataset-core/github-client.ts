import { classifyLicense } from './license-policy.js';
import type { LicenseStatus } from './types.js';
import {
  JAVASCRIPT_TYPESCRIPT_EXTENSIONS,
  type MergedPullRequestSummary,
  type RawFileChange,
  type RawPullRequest,
  type RawReviewComment,
} from './github-types.js';

const DEFAULT_BASE_URL = 'https://api.github.com';
const MAX_FILE_BYTES = 200_000;

export interface GitHubClientOptions {
  token: string;
  baseUrl?: string;
  fetchImpl?: typeof fetch;
  maxFileBytes?: number;
  /** File extensions to fetch full before/after content for. Defaults to JS/TS. */
  fileExtensions?: Set<string>;
}

interface GitHubFileEntry {
  filename: string;
  previous_filename?: string;
  status: string;
  additions: number;
  deletions: number;
}

/**
 * A GitHub REST API client for mining merged pull requests into
 * RawPullRequest records. Every request goes through `fetchImpl`, which
 * defaults to the global `fetch` but is injectable so tests never make a
 * real network call.
 */
export class GitHubClient {
  private readonly token: string;
  private readonly baseUrl: string;
  private readonly fetchImpl: typeof fetch;
  private readonly maxFileBytes: number;
  private readonly fileExtensions: Set<string>;
  private readonly licenseCache = new Map<string, { spdxId: string | null; status: LicenseStatus }>();

  constructor(options: GitHubClientOptions) {
    this.token = options.token;
    this.baseUrl = options.baseUrl ?? DEFAULT_BASE_URL;
    this.fetchImpl = options.fetchImpl ?? fetch;
    this.maxFileBytes = options.maxFileBytes ?? MAX_FILE_BYTES;
    this.fileExtensions = options.fileExtensions ?? JAVASCRIPT_TYPESCRIPT_EXTENSIONS;
  }

  private async request(path: string, init: RequestInit = {}, attempt = 0): Promise<Response> {
    const response = await this.fetchImpl(`${this.baseUrl}${path}`, {
      ...init,
      headers: {
        Accept: 'application/vnd.github+json',
        Authorization: `Bearer ${this.token}`,
        'X-GitHub-Api-Version': '2022-11-28',
        ...init.headers,
      },
    });

    const isRateLimited =
      response.status === 403 && response.headers.get('x-ratelimit-remaining') === '0';

    if (isRateLimited && attempt < 1) {
      const resetAt = Number(response.headers.get('x-ratelimit-reset') ?? '0') * 1000;
      const delayMs = Math.max(resetAt - Date.now(), 1000);
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      return this.request(path, init, attempt + 1);
    }

    return response;
  }

  private nextPageUrl(response: Response): string | null {
    const link = response.headers.get('link');
    if (!link) return null;

    for (const segment of link.split(',')) {
      const match = segment.match(/<([^>]+)>;\s*rel="next"/);
      if (match) return match[1] ?? null;
    }
    return null;
  }

  async getRepositoryLicense(
    owner: string,
    repo: string,
  ): Promise<{ spdxId: string | null; status: LicenseStatus }> {
    const cacheKey = `${owner}/${repo}`;
    const cached = this.licenseCache.get(cacheKey);
    if (cached) return cached;

    const response = await this.request(`/repos/${owner}/${repo}/license`);
    if (response.status === 404) {
      const result = { spdxId: null, status: classifyLicense(undefined) };
      this.licenseCache.set(cacheKey, result);
      return result;
    }
    if (!response.ok) {
      throw new Error(`GitHub API request failed: GET license -> ${response.status}`);
    }

    const body = (await response.json()) as { license: { spdx_id?: string } | null };
    const spdxId = body.license?.spdx_id && body.license.spdx_id !== 'NOASSERTION' ? body.license.spdx_id : undefined;
    const result = { spdxId: spdxId ?? null, status: classifyLicense(spdxId) };
    this.licenseCache.set(cacheKey, result);
    return result;
  }

  /** Lists merged pull requests, most recently updated first, stopping once
   * a page's PRs were all updated before `since`. */
  async listMergedPullRequests(
    owner: string,
    repo: string,
    options: { since?: Date | undefined; perPage?: number; maxPages?: number } = {},
  ): Promise<MergedPullRequestSummary[]> {
    const perPage = options.perPage ?? 50;
    const maxPages = options.maxPages ?? 10;
    const results: MergedPullRequestSummary[] = [];

    let path: string | null =
      `/repos/${owner}/${repo}/pulls?state=closed&sort=updated&direction=desc&per_page=${perPage}`;

    for (let page = 0; path && page < maxPages; page += 1) {
      const response = await this.request(path);
      if (!response.ok) {
        throw new Error(`GitHub API request failed: GET pulls -> ${response.status}`);
      }

      const body = (await response.json()) as Array<{
        number: number;
        title: string;
        merged_at: string | null;
        merge_commit_sha: string | null;
        updated_at: string;
        base: { sha: string };
      }>;

      let reachedCutoff = false;
      for (const pr of body) {
        if (options.since && new Date(pr.updated_at) < options.since) {
          reachedCutoff = true;
          break;
        }
        if (!pr.merged_at || !pr.merge_commit_sha) continue;

        results.push({
          number: pr.number,
          title: pr.title,
          mergeCommitSha: pr.merge_commit_sha,
          baseSha: pr.base.sha,
          mergedAt: pr.merged_at,
        });
      }

      if (reachedCutoff) break;
      const next = this.nextPageUrl(response);
      path = next ? this.stripBaseUrl(next) : null;
    }

    return results;
  }

  private stripBaseUrl(url: string): string {
    return url.startsWith(this.baseUrl) ? url.slice(this.baseUrl.length) : url;
  }

  private async listPullRequestFiles(owner: string, repo: string, pullNumber: number): Promise<GitHubFileEntry[]> {
    const files: GitHubFileEntry[] = [];
    let path: string | null = `/repos/${owner}/${repo}/pulls/${pullNumber}/files?per_page=100`;

    while (path) {
      const response = await this.request(path);
      if (!response.ok) {
        throw new Error(`GitHub API request failed: GET pull files -> ${response.status}`);
      }
      files.push(...((await response.json()) as GitHubFileEntry[]));
      const next = this.nextPageUrl(response);
      path = next ? this.stripBaseUrl(next) : null;
    }

    return files;
  }

  async getFileContent(owner: string, repo: string, path: string, ref: string): Promise<string | null> {
    const encodedPath = path.split('/').map(encodeURIComponent).join('/');
    const response = await this.request(
      `/repos/${owner}/${repo}/contents/${encodedPath}?ref=${encodeURIComponent(ref)}`,
    );
    if (response.status === 404) return null;
    if (!response.ok) {
      throw new Error(`GitHub API request failed: GET contents -> ${response.status}`);
    }

    const body = (await response.json()) as { encoding: string; content: string; size: number };
    if (body.size > this.maxFileBytes) return null;
    if (body.encoding !== 'base64') return null;

    return Buffer.from(body.content, 'base64').toString('utf-8');
  }

  async getPullRequestReviewComments(owner: string, repo: string, pullNumber: number): Promise<RawReviewComment[]> {
    const comments: RawReviewComment[] = [];
    let path: string | null = `/repos/${owner}/${repo}/pulls/${pullNumber}/comments?per_page=100`;

    while (path) {
      const response = await this.request(path);
      if (!response.ok) {
        throw new Error(`GitHub API request failed: GET pull comments -> ${response.status}`);
      }
      const body = (await response.json()) as Array<{
        path: string | null;
        line: number | null;
        body: string;
        user: { login: string } | null;
      }>;
      comments.push(
        ...body.map((comment) => ({
          path: comment.path,
          line: comment.line,
          body: comment.body,
          author: comment.user?.login ?? null,
        })),
      );
      const next = this.nextPageUrl(response);
      path = next ? this.stripBaseUrl(next) : null;
    }

    return comments;
  }

  private isRelevantFile(path: string): boolean {
    const extension = path.slice(path.lastIndexOf('.'));
    return this.fileExtensions.has(extension);
  }

  private toChangeStatus(status: string): RawFileChange['status'] {
    switch (status) {
      case 'added':
        return 'added';
      case 'removed':
        return 'removed';
      case 'renamed':
        return 'renamed';
      default:
        return 'modified';
    }
  }

  /** Mines a single merged pull request into a RawPullRequest: before/after
   * content for files matching `fileExtensions`, review comments, and
   * repository license. */
  async minePullRequest(
    owner: string,
    repo: string,
    summary: MergedPullRequestSummary,
  ): Promise<RawPullRequest> {
    const [entries, reviewComments, license] = await Promise.all([
      this.listPullRequestFiles(owner, repo, summary.number),
      this.getPullRequestReviewComments(owner, repo, summary.number),
      this.getRepositoryLicense(owner, repo),
    ]);

    const relevantEntries = entries.filter((entry) => this.isRelevantFile(entry.filename));

    const files: RawFileChange[] = await Promise.all(
      relevantEntries.map(async (entry) => {
        const status = this.toChangeStatus(entry.status);
        const [before, after] = await Promise.all([
          status === 'added'
            ? null
            : this.getFileContent(owner, repo, entry.filename, summary.baseSha),
          status === 'removed'
            ? null
            : this.getFileContent(owner, repo, entry.filename, summary.mergeCommitSha),
        ]);

        return {
          path: entry.filename,
          status,
          additions: entry.additions,
          deletions: entry.deletions,
          before,
          after,
        };
      }),
    );

    return {
      host: 'github',
      repository: `${owner}/${repo}`,
      pullNumber: summary.number,
      title: summary.title,
      baseSha: summary.baseSha,
      mergeCommitSha: summary.mergeCommitSha,
      sourceUrl: `https://github.com/${owner}/${repo}/pull/${summary.number}`,
      mergedAt: summary.mergedAt,
      licenseSpdx: license.spdxId,
      licenseStatus: license.status,
      files,
      reviewComments,
      collectedAt: new Date().toISOString(),
    };
  }
}
