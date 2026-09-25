import { datasetExampleId } from './id.js';
import type { RawPullRequest } from './github-types.js';
import type { CodeChange, DatasetExample, Language, ReviewSignal } from './types.js';

function inferLanguage(files: RawPullRequest['files']): Language {
  const isTypeScript = files.some((file) => /\.(ts|tsx|mts|cts)$/.test(file.path));
  return isTypeScript ? 'typescript' : 'javascript';
}

/**
 * Combines a mined RawPullRequest with signals produced by a separate,
 * explicitly reviewed labeling step (human curation, or LLM-assisted
 * classification a human signs off on) into a DatasetExample.
 *
 * Only fully-resolved modifications (both before and after content fetched)
 * become CodeChange records; added/removed files or files whose content
 * could not be fetched (binary, too large, missing) are dropped, since
 * CodeChange requires both sides as text.
 */
export function toDatasetExample(raw: RawPullRequest, signals: ReviewSignal[]): DatasetExample {
  const changes: CodeChange[] = raw.files
    .filter((file): file is typeof file & { before: string; after: string } =>
      file.before !== null && file.after !== null,
    )
    .map((file) => ({
      path: file.path,
      before: file.before,
      after: file.after,
      additions: file.additions,
      deletions: file.deletions,
    }));

  const provenance = {
    host: raw.host,
    repository: raw.repository,
    commitSha: raw.mergeCommitSha,
    sourceUrl: raw.sourceUrl,
    ...(raw.licenseSpdx ? { licenseSpdx: raw.licenseSpdx } : {}),
    licenseStatus: raw.licenseStatus,
    collectedAt: raw.collectedAt,
  };

  return {
    id: datasetExampleId(provenance, changes, signals),
    language: inferLanguage(raw.files),
    frameworks: [],
    libraries: [],
    databases: [],
    tools: [],
    changes,
    signals,
    provenance,
  };
}
