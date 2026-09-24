# Ecosystems

Ecosystem integrations identify framework, runtime, ORM, database, testing, package-management, and tooling context for a repository.

Each ecosystem has a canonical machine-readable catalog and an implementation-specific detector. Detection must produce evidence so downstream analyzers and models can distinguish observed repository facts from inferred context.

The first deep integration targets the JavaScript/TypeScript ecosystem, with Node.js, NestJS, Express, Fastify, TypeORM, Prisma, Drizzle, Sequelize, MikroORM, Knex, Kysely, common databases, testing frameworks, and build/tooling systems.

Analysis dimensions such as security, correctness, readability, maintainability, architecture, and performance belong to CodeVanta analyzers and models, not to ecosystem catalogs.
