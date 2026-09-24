import type { DatasetExample, ReviewSignal } from './types.js';

export function normalizeReviewSignals(signals: ReviewSignal[]): ReviewSignal[] {
  return signals
    .map((signal) => ({
      ...signal,
      summary: signal.summary.trim(),
      reviewerComment: signal.reviewerComment?.trim() || undefined,
    }))
    .filter((signal) => signal.summary.length > 0)
    .filter((signal, index, all) => all.findIndex((candidate) =>
      candidate.category === signal.category &&
      candidate.summary === signal.summary &&
      candidate.confidence === signal.confidence,
    ) === index);
}

export function normalizeExample(example: DatasetExample): DatasetExample {
  return {
    ...example,
    frameworks: unique(example.frameworks),
    libraries: unique(example.libraries),
    databases: unique(example.databases),
    tools: unique(example.tools),
    signals: normalizeReviewSignals(example.signals),
    provenance: {
      ...example.provenance,
      repository: example.provenance.repository.trim(),
      sourceUrl: example.provenance.sourceUrl.trim(),
    },
  };
}

function unique(values: string[]): string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}
