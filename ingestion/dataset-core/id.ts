import { createHash } from 'node:crypto';
import type { CodeChange, RepositoryProvenance, ReviewSignal } from './types.js';

export function datasetExampleId(
  provenance: RepositoryProvenance,
  changes: CodeChange[],
  signals: ReviewSignal[],
): string {
  const canonical = JSON.stringify({
    repository: provenance.repository,
    commitSha: provenance.commitSha,
    changes,
    signals,
  });

  return createHash('sha256').update(canonical).digest('hex');
}
