export const NODE_ECOSYSTEM = {
  runtimes: ['node'],
  languages: ['javascript', 'typescript'],
  frameworks: ['nestjs', 'express', 'fastify'],
  orms: ['typeorm', 'prisma', 'drizzle', 'sequelize', 'mikro-orm', 'knex', 'kysely'],
  databases: ['postgresql', 'mysql', 'mariadb', 'sqlite', 'sql-server', 'mongodb', 'redis'],
  testing: ['jest', 'vitest', 'supertest', 'playwright', 'cypress'],
  tooling: ['eslint', 'prettier', 'typescript', 'npm', 'pnpm', 'yarn', 'bun', 'esbuild', 'swc', 'webpack', 'tsup'],
} as const;

export type EcosystemArea = keyof typeof NODE_ECOSYSTEM;

export function isKnownEcosystemValue(value: string): boolean {
  return Object.values(NODE_ECOSYSTEM).some((values) => values.includes(value as never));
}
