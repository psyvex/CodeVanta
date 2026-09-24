# CodeVanta

Local, language-specific AI models for software quality.

## Initial target: Node.js / TypeScript / JavaScript ecosystem

CodeVanta will build specialized models and deterministic analysis components for reviewing and improving Node.js ecosystem code. The first model family targets:

- Node.js runtime and platform APIs
- TypeScript
- JavaScript
- NestJS
- Express and Fastify
- ORM/data-access patterns: TypeORM, Prisma, Drizzle ORM, Sequelize, MikroORM, Knex, Kysely
- SQL and database usage: PostgreSQL, MySQL/MariaDB, SQLite, SQL Server, MongoDB, Redis
- Testing: Jest, Vitest, Supertest, Playwright, Cypress
- Tooling: ESLint, Prettier, TypeScript compiler, package managers, bundlers, and common Node.js tooling

## Design principle

CodeVanta is not intended to replace deterministic software-analysis tools. Linters, formatters, type checkers, AST analysis, dependency scanners, security scanners, and test frameworks should provide deterministic evidence. The model should focus on contextual reasoning: maintainability, architecture, suspicious error handling, likely bugs, security context, repository conventions, missing edge cases, and actionable improvements.

## Planned pipeline

```text
Repository
  -> language/framework/tool detection
  -> AST + static-analysis evidence
  -> repository context
  -> specialized Node/TS/JS model
  -> finding classification + confidence
  -> suggested patch
  -> validation against tests/static tools
```

## Finding confidence

Every model finding should be classified as one of:

- Definite
- Likely
- Possible
- Suggestion

The system must not present uncertain findings as facts.

## Model strategy

Start from a suitable open code model and adapt it rather than training a foundation model from scratch. Prefer a shared base model with language/ecosystem-specific adapters when evaluation shows that this is effective.

Training data should emphasize real review signals such as before/after fixes, pull-request discussions, security fixes, bug fixes, refactors, and accepted/rejected review feedback. Dataset records must retain provenance and applicable license metadata.

## Repository layout

The repository will separate ecosystem knowledge, review taxonomy, dataset schemas, training code, evaluation, and model artifacts as the project grows.

## Status

The repository currently contains the initial project specification. The next implementation stage is the Node/TypeScript/JavaScript ecosystem dataset and analysis pipeline.
