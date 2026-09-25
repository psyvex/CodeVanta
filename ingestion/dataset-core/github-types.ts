import type { LicenseStatus } from './types.js';

/** File extensions the miner will fetch full before/after content for. */
export const JAVASCRIPT_TYPESCRIPT_EXTENSIONS = new Set([
  '.js',
  '.jsx',
  '.mjs',
  '.cjs',
  '.ts',
  '.tsx',
  '.mts',
  '.cts',
]);

export interface RawFileChange {
  path: string;
  status: 'added' | 'removed' | 'modified' | 'renamed';
  additions: number;
  deletions: number;
  /** null when the file was added, or its content could not be fetched (binary, too large, missing). */
  before: string | null;
  /** null when the file was removed, or its content could not be fetched (binary, too large, missing). */
  after: string | null;
}

export interface RawReviewComment {
  path: string | null;
  line: number | null;
  body: string;
  author: string | null;
}

/**
 * Raw mined evidence for one merged pull request. This is deliberately not
 * a DatasetExample: it carries no category/severity/confidence labels,
 * because those cannot be reliably inferred from free-text review comments
 * without fabricating training signal. A separate, explicitly reviewed
 * labeling step (human curation or LLM-assisted classification that a human
 * signs off on) turns a RawPullRequest into ReviewSignals.
 */
export interface RawPullRequest {
  host: 'github';
  repository: string;
  pullNumber: number;
  title: string;
  baseSha: string;
  mergeCommitSha: string;
  sourceUrl: string;
  mergedAt: string;
  licenseSpdx: string | null;
  licenseStatus: LicenseStatus;
  files: RawFileChange[];
  reviewComments: RawReviewComment[];
  collectedAt: string;
}

export interface MergedPullRequestSummary {
  number: number;
  title: string;
  mergeCommitSha: string;
  baseSha: string;
  mergedAt: string;
}
