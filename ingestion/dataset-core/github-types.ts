import type { Language, LicenseStatus } from './types.js';

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

export const PYTHON_EXTENSIONS = new Set(['.py', '.pyi']);

/**
 * The languages the miner knows how to scope a mining run to, and the file
 * extensions each one covers. Adding a language here is what's needed for
 * mine.ts's --language flag to recognize it; languages/<name> having real
 * analyzers is a separate, unrelated prerequisite for the mined data to be
 * useful, not for mining to run.
 */
export const MINEABLE_LANGUAGE_EXTENSIONS: Record<'javascript-typescript' | 'python', Set<string>> = {
  'javascript-typescript': JAVASCRIPT_TYPESCRIPT_EXTENSIONS,
  python: PYTHON_EXTENSIONS,
};

export function inferLanguageFromPath(path: string): Language | null {
  if (JAVASCRIPT_TYPESCRIPT_EXTENSIONS.has(extensionOf(path))) {
    return /\.(ts|tsx|mts|cts)$/.test(path) ? 'typescript' : 'javascript';
  }
  if (PYTHON_EXTENSIONS.has(extensionOf(path))) {
    return 'python';
  }
  return null;
}

function extensionOf(path: string): string {
  return path.slice(path.lastIndexOf('.'));
}

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
