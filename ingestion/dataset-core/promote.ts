#!/usr/bin/env node
/**
 * Joins a human-reviewed label.ts review file back with the original mined
 * RawPullRequest evidence to produce DatasetExample JSONL, ready for
 * project.ts to flatten into training-example.schema.json records.
 *
 * A pull request with no entry in the review file (because label.ts found
 * nothing to propose, or a human deleted every proposal on it) is skipped:
 * it contributes no signals, so it has nothing to train on.
 *
 * Usage:
 *   node promote.js --raw raw.jsonl --review review.jsonl --out examples.jsonl
 */
import { readFile, writeFile } from 'node:fs/promises';
import { toReviewSignal } from './propose-signals.js';
import { toDatasetExample } from './raw-to-example.js';
import type { RawPullRequest } from './github-types.js';
import type { ReviewFileEntry } from './label.js';
import type { DatasetExample } from './types.js';

function reviewKey(repository: string, pullNumber: number): string {
  return `${repository}#${pullNumber}`;
}

export function promote(rawPullRequests: RawPullRequest[], reviewEntries: ReviewFileEntry[]): DatasetExample[] {
  const reviewByKey = new Map(
    reviewEntries.map((entry) => [reviewKey(entry.repository, entry.pullNumber), entry]),
  );

  const examples: DatasetExample[] = [];
  for (const raw of rawPullRequests) {
    const review = reviewByKey.get(reviewKey(raw.repository, raw.pullNumber));
    if (!review || review.proposals.length === 0) continue;

    const signals = review.proposals.map(toReviewSignal);
    examples.push(toDatasetExample(raw, signals));
  }

  return examples;
}

async function readJsonl<T>(path: string): Promise<T[]> {
  const lines = (await readFile(path, 'utf-8')).split(/\r?\n/).filter((line) => line.trim());
  return lines.map((line) => JSON.parse(line) as T);
}

interface CliOptions {
  rawPath: string;
  reviewPath: string;
  outPath: string;
}

export function parseArgs(argv: string[]): CliOptions {
  const args = new Map<string, string>();
  for (let i = 0; i < argv.length; i += 2) {
    const flag = argv[i];
    const value = argv[i + 1];
    if (!flag?.startsWith('--') || value === undefined) {
      throw new Error(`invalid arguments near "${flag ?? ''}"`);
    }
    args.set(flag.slice(2), value);
  }

  const rawPath = args.get('raw');
  const reviewPath = args.get('review');
  const outPath = args.get('out');
  if (!rawPath || !reviewPath || !outPath) {
    throw new Error('--raw <raw.jsonl>, --review <review.jsonl>, and --out <examples.jsonl> are required');
  }

  return { rawPath, reviewPath, outPath };
}

async function main(): Promise<void> {
  const options = parseArgs(process.argv.slice(2));
  const [rawPullRequests, reviewEntries] = await Promise.all([
    readJsonl<RawPullRequest>(options.rawPath),
    readJsonl<ReviewFileEntry>(options.reviewPath),
  ]);

  const examples = promote(rawPullRequests, reviewEntries);
  const jsonl = examples.map((example) => JSON.stringify(example)).join('\n') + (examples.length ? '\n' : '');
  await writeFile(options.outPath, jsonl);

  console.log(`promoted ${examples.length} reviewed pull requests to DatasetExamples -> ${options.outPath}`);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
