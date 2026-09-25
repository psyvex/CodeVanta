#!/usr/bin/env node
/**
 * Mines merged pull requests from a GitHub repository into RawPullRequest
 * JSONL. Produces raw evidence only -- no category/severity/confidence
 * labels are invented from review comments (see github-types.ts). A
 * separate, explicitly reviewed labeling step turns this output into
 * DatasetExample records via raw-to-example.ts.
 *
 * Usage:
 *   GITHUB_TOKEN=... node mine.js --repo owner/name --since 2024-01-01 --out raw.jsonl
 *   GITHUB_TOKEN=... node mine.js --repo owner/name --language python --out raw.jsonl
 */
import { writeFile } from 'node:fs/promises';
import { GitHubClient } from './github-client.js';
import { MINEABLE_LANGUAGE_EXTENSIONS } from './github-types.js';
import type { RawPullRequest } from './github-types.js';

type MineableLanguage = keyof typeof MINEABLE_LANGUAGE_EXTENSIONS;

const DEFAULT_LANGUAGE: MineableLanguage = 'javascript-typescript';

interface CliOptions {
  owner: string;
  repo: string;
  language: MineableLanguage;
  since?: Date | undefined;
  maxPages: number;
  out: string;
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

  const repoArg = args.get('repo');
  if (!repoArg || !repoArg.includes('/')) {
    throw new Error('--repo owner/name is required');
  }
  const [owner, repo] = repoArg.split('/', 2) as [string, string];

  const out = args.get('out');
  if (!out) {
    throw new Error('--out <path> is required');
  }

  const languageArg = args.get('language') ?? DEFAULT_LANGUAGE;
  if (!(languageArg in MINEABLE_LANGUAGE_EXTENSIONS)) {
    const supported = Object.keys(MINEABLE_LANGUAGE_EXTENSIONS).join(', ');
    throw new Error(`--language must be one of: ${supported} (got "${languageArg}")`);
  }

  const sinceArg = args.get('since');
  return {
    owner,
    repo,
    language: languageArg as MineableLanguage,
    since: sinceArg ? new Date(sinceArg) : undefined,
    maxPages: Number(args.get('max-pages') ?? '10'),
    out,
  };
}

export async function mine(client: GitHubClient, options: CliOptions): Promise<RawPullRequest[]> {
  const summaries = await client.listMergedPullRequests(options.owner, options.repo, {
    since: options.since,
    maxPages: options.maxPages,
  });

  const results: RawPullRequest[] = [];
  for (const summary of summaries) {
    results.push(await client.minePullRequest(options.owner, options.repo, summary));
  }
  return results;
}

async function main(): Promise<void> {
  const token = process.env.GITHUB_TOKEN;
  if (!token) {
    throw new Error('GITHUB_TOKEN environment variable is required');
  }

  const options = parseArgs(process.argv.slice(2));
  const client = new GitHubClient({
    token,
    fileExtensions: MINEABLE_LANGUAGE_EXTENSIONS[options.language],
  });
  const raw = await mine(client, options);

  const jsonl = raw.map((entry) => JSON.stringify(entry)).join('\n') + (raw.length ? '\n' : '');
  await writeFile(options.out, jsonl);

  console.log(
    `mined ${raw.length} merged pull requests (${options.language}) from ${options.owner}/${options.repo} -> ${options.out}`,
  );
}

if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
