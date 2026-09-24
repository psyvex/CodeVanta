import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

export type EvidenceKind = 'manifest' | 'import' | 'config' | 'file';
export type DetectionConfidence = 'definite' | 'likely' | 'possible';

export interface EcosystemEvidence {
  kind: EvidenceKind;
  source: string;
  detail: string;
}

export interface DetectedTechnology {
  id: string;
  confidence: DetectionConfidence;
  evidence: EcosystemEvidence[];
}

export interface JavaScriptEcosystemContext {
  language: 'javascript' | 'typescript';
  runtime?: 'nodejs';
  technologies: DetectedTechnology[];
}

export interface PackageManifest {
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
}

const TECHNOLOGY_BY_PACKAGE: Record<string, string> = {
  '@nestjs/common': 'nestjs',
  '@nestjs/core': 'nestjs',
  express: 'express',
  fastify: 'fastify',
  typeorm: 'typeorm',
  prisma: 'prisma',
  '@prisma/client': 'prisma',
  'drizzle-orm': 'drizzle-orm',
  sequelize: 'sequelize',
  '@mikro-orm/core': 'mikro-orm',
  knex: 'knex',
  kysely: 'kysely',
};

export function detectJavaScriptEcosystemFromManifest(
  packageJson: PackageManifest,
): JavaScriptEcosystemContext {
  const dependencies = {
    ...packageJson.dependencies,
    ...packageJson.devDependencies,
  };

  const evidenceByTechnology = new Map<string, EcosystemEvidence[]>();
  for (const packageName of Object.keys(dependencies)) {
    const technology = TECHNOLOGY_BY_PACKAGE[packageName];
    if (!technology) continue;

    const evidence = evidenceByTechnology.get(technology) ?? [];
    evidence.push({
      kind: 'manifest',
      source: 'package.json',
      detail: `dependency: ${packageName}`,
    });
    evidenceByTechnology.set(technology, evidence);
  }

  const technologies = [...evidenceByTechnology.entries()].map(([id, evidence]) => ({
    id,
    confidence: 'definite' as const,
    evidence,
  }));

  const language = Object.keys(dependencies).some((name) => name === 'typescript')
    ? 'typescript'
    : 'javascript';

  return { language, runtime: 'nodejs', technologies };
}

export async function detectJavaScriptEcosystem(
  root: string,
): Promise<JavaScriptEcosystemContext> {
  const packageJsonPath = resolve(root, 'package.json');
  const packageJson = JSON.parse(await readFile(packageJsonPath, 'utf8')) as PackageManifest;
  return detectJavaScriptEcosystemFromManifest(packageJson);
}
