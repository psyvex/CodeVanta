import type { CodeChange, DatasetExample, ReviewSignal, Severity } from './types.js';

export type TrainingSourceType =
  | 'repository'
  | 'commit'
  | 'pull-request'
  | 'issue'
  | 'synthetic'
  | 'human-authored';

/**
 * A single supervised fine-tuning record, matching
 * configs/training-example.schema.json.
 */
export interface TrainingExample {
  language: DatasetExample['language'];
  framework: string | null;
  library: string | null;
  database: string | null;
  code: string;
  context?: string | undefined;
  finding: {
    category: string;
    confidence: ReviewSignal['confidence'];
    severity: Severity;
    summary: string;
    reason?: string | undefined;
  };
  provenance: {
    source_type: TrainingSourceType;
    source_url: string;
    repository?: string | null;
    commit?: string | null;
    license?: string | null;
  };
}

/**
 * Projects a raw ingested DatasetExample into one TrainingExample per review
 * signal it carries. Only examples whose license has been explicitly
 * classified as `allowed` are projected -- `unknown` and `review-required`
 * must never be silently treated as training-safe (see ingestion/README.md).
 */
export function projectTrainingExamples(example: DatasetExample): TrainingExample[] {
  if (example.provenance.licenseStatus !== 'allowed') {
    return [];
  }

  const code = renderChanges(example.changes);
  if (!code) {
    return [];
  }

  return example.signals.map((signal) => ({
    language: example.language,
    // A signal is not currently linked to a specific technology among the
    // example's frameworks/libraries/databases, so at most the first of
    // each is surfaced as the primary technology rather than guessing.
    framework: example.frameworks[0] ?? null,
    library: example.libraries[0] ?? null,
    database: example.databases[0] ?? null,
    code,
    context: signal.reviewerComment,
    finding: {
      category: signal.category,
      confidence: signal.confidence,
      severity: signal.severity,
      summary: signal.summary,
      reason: signal.reviewerComment,
    },
    provenance: {
      source_type: 'commit',
      source_url: example.provenance.sourceUrl,
      repository: example.provenance.repository,
      commit: example.provenance.commitSha,
      license: example.provenance.licenseSpdx ?? null,
    },
  }));
}

function renderChanges(changes: CodeChange[]): string {
  return changes
    .filter((change) => change.after.length > 0)
    .map((change) => `// ${change.path}\n${change.after}`)
    .join('\n\n');
}
