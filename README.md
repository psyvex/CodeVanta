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

The Node.js ecosystem is the first deep specialization, not the repository architecture. CodeVanta is designed to add Python, Java, Go, Rust, PHP, C#, C++, and other ecosystems without reorganizing the platform.

## Design principle

CodeVanta is not intended to replace deterministic software-analysis tools. Linters, formatters, type checkers, AST analysis, dependency scanners, security scanners, and test frameworks should provide deterministic evidence. The model should focus on contextual reasoning: maintainability, architecture, suspicious error handling, likely bugs, security context, repository conventions, missing edge cases, and actionable improvements.

## Planned pipeline

```text
Repository
  -> language/framework/tool detection
  -> AST + static-analysis evidence
  -> repository context
  -> specialized language/ecosystem model
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

```text
models/       Model definitions, adapters, manifests, and artifact metadata
datasets/     Versioned dataset outputs and manifests
training/     Fine-tuning and reproducible training pipelines
evaluation/   Benchmarks and model-quality measurements
inference/    Local CPU/GPU/WebGPU/WASM inference runtimes
analyzers/    Deterministic AST, lint, type, security, and dependency analysis
ecosystem/    Language/framework/ORM/database/tool catalogs
ingestion/    Git/GitHub/review/diff/license/provenance ingestion
docker/       Reproducible development, training, evaluation, and inference images
scripts/      Small deterministic project utilities
docs/         Architecture and project documentation
tests/        Automated tests
```

Large model binaries and raw third-party repositories should not be committed to the source repository.

## Status

The repository now contains the initial multi-language platform structure and a language-neutral dataset core. The first deep implementation remains the Node.js/TypeScript/JavaScript ingestion and ecosystem pipeline.
