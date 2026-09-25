#!/usr/bin/env node
/**
 * Proposes candidate ReviewSignals for every mined pull request in a
 * RawPullRequest JSONL file (mine.ts's output), writing a review file a
 * human edits in place: delete wrong proposals, correct category/severity/
 * confidence/summary on the ones kept. promote.ts then reads that edited
 * file back and joins it with the raw evidence to produce DatasetExamples.
 *
 * Usage:
 *   node label.js --in raw.jsonl --out review.jsonl
 */
import { readFile, writeFile } from 'node:fs/promises';
import { proposeSignals, type ProposedSignal } from './propose-signals.js';
import type { RawPullRequest } from './github-types.js';

export interface ReviewFileEntry {
  repository: string;
  pullNumber: number;
  sourceUrl: string;
  title: string;
  proposals: ProposedSignal[];
}

export function buildReviewFile(rawPullRequests: RawPullRequest[]): ReviewFileEntry[] {
  return rawPullRequests
    .map((raw) => ({
      repository: raw.repository,
      pullNumber: raw.pullNumber,
      sourceUrl: raw.sourceUrl,
      title: raw.title,
      proposals: proposeSignals(raw),
    }))
    .filter((entry) => entry.proposals.length > 0);
}

interface CliOptions {
  inPath: string;
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

  const inPath = args.get('in');
  const outPath = args.get('out');
  if (!inPath || !outPath) {
    throw new Error('--in <raw.jsonl> and --out <review.jsonl> are required');
  }

  return { inPath, outPath };
}

async function main(): Promise<void> {
  const options = parseArgs(process.argv.slice(2));
  const lines = (await readFile(options.inPath, 'utf-8')).split(/\r?\n/).filter((line) => line.trim());
  const rawPullRequests = lines.map((line) => JSON.parse(line) as RawPullRequest);

  const reviewFile = buildReviewFile(rawPullRequests);
  const jsonl = reviewFile.map((entry) => JSON.stringify(entry)).join('\n') + (reviewFile.length ? '\n' : '');
  await writeFile(options.outPath, jsonl);

  console.log(
    `proposed signals for ${reviewFile.length} of ${rawPullRequests.length} pull requests -> ${options.outPath}`,
  );
  console.log('Review this file before running promote.ts: delete wrong proposals, correct the rest.');
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
