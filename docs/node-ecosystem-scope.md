# Node Ecosystem Model Scope

## Languages

Primary support:

- TypeScript
- JavaScript

Runtime:

- Node.js

## Frameworks

### Primary

- NestJS

### Secondary

- Express
- Fastify

The first training and evaluation datasets should prioritize NestJS because its decorators, dependency injection, modules, guards, pipes, interceptors, filters, providers, and controller/service boundaries create useful repository-level review signals.

## ORM and data-access coverage

Prioritize:

1. TypeORM
2. Prisma
3. Drizzle ORM
4. Sequelize
5. MikroORM
6. Knex
7. Kysely

TypeORM, Prisma, and Drizzle should receive the first deep rules because they represent distinct styles: decorator/entity based, generated typed client, and SQL-oriented typed ORM/query building.

## Database coverage

Relational:

- PostgreSQL
- MySQL
- MariaDB
- SQLite
- SQL Server

Document/key-value:

- MongoDB
- Redis

The model should learn database-specific review context instead of treating every database as interchangeable. For example, raw SQL safety, transaction semantics, indexing, pagination, isolation, connection lifecycle, and query patterns differ by database and access library.

## Tooling and testing

Initial ecosystem signals should include:

- TypeScript compiler
- ESLint
- Prettier
- npm
- pnpm
- yarn
- Bun
- Jest
- Vitest
- Supertest
- Playwright
- Cypress
- common Node build tooling such as esbuild, SWC, webpack, and tsup

## Model responsibilities

The model should reason about:

- security
- correctness
- likely bugs
- readability
- maintainability
- architecture
- ORM usage
- database access
- error handling
- API design
- testing quality
- performance
- dependency usage
- Node/TypeScript/JavaScript idioms
- NestJS conventions
- repository-specific standards

It should not replace deterministic tools. Static analyzers should provide machine-verifiable evidence that is then supplied to the model as context.

## Initial training priority

The first dataset should favor examples with a clear engineering signal:

1. security fixes
2. bug fixes
3. pull-request review comments with accepted changes
4. refactors with an explicit reason
5. regression fixes
6. ORM/database fixes
7. NestJS architecture corrections
8. test additions that close a real behavior gap
9. maintainability/readability improvements
10. style-only changes last
