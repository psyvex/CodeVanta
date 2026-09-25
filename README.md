# CodeVanta

Local, language-specific AI models and deterministic analysis for software quality.

## Production architecture

CodeVanta is a polyglot monorepo designed for long-term local and production use:

- **Rust** is the production core: repository processing, analysis orchestration, findings, patch representation, local runtime foundations, CLI, and future LSP/server/WASM targets.
- **Python** is the ML layer: dataset preparation, fine-tuning, evaluation, distillation, and quantization.
- **TypeScript** is used where the JavaScript/TypeScript ecosystem is the domain, and for future editor integrations such as VS Code.
- **Language-native tooling** is preferred when it provides authoritative parsers, compilers, analyzers, or ecosystem metadata.

## Initial specialization

The first deep model family targets the Node.js / TypeScript / JavaScript ecosystem:

- Node.js runtime and platform APIs
- TypeScript and JavaScript
- NestJS, Express, and Fastify
- TypeORM, Prisma, Drizzle ORM, Sequelize, MikroORM, Knex, and Kysely
- PostgreSQL, MySQL/MariaDB, SQLite, SQL Server, MongoDB, and Redis
- Jest, Vitest, Supertest, Playwright, and Cypress
- ESLint, Prettier, TypeScript compiler, package managers, bundlers, and common Node.js tooling

This is the first specialization, not the platform boundary. Python, Java, Go, Rust, PHP, C#, C++, and other ecosystems can be added without redesigning the core.

## Analysis philosophy

CodeVanta does not attempt to replace deterministic software-analysis tools. Linters, formatters, compilers/type checkers, AST analyzers, dependency scanners, security scanners, and tests provide evidence. Models focus on contextual reasoning: maintainability, architecture, suspicious error handling, likely bugs, security context, repository conventions, missing edge cases, prioritization, explanations, and actionable patches.

## Pipeline

```text
Repository
  -> language/framework/tool detection
  -> Rust core: repository + AST + deterministic analysis
  -> ecosystem context
  -> specialized local model
  -> finding classification + confidence
  -> suggested patch
  -> deterministic/test validation
```

## Finding confidence

Every model finding must be classified as:

- Definite
- Likely
- Possible
- Suggestion

Uncertain findings must not be presented as facts.

## Model strategy

Start from a suitable open code model and adapt it rather than training a foundation model from scratch. Prefer a shared base model with language/ecosystem-specific adapters when evaluation shows that this improves quality.

Training data should emphasize real review signals: before/after fixes, pull-request discussions, security fixes, bug fixes, refactors, and accepted/rejected review feedback. Dataset records retain provenance and applicable license metadata.

Large model binaries and raw third-party repositories do not belong in Git.

## Repository layout

```text
core/         Rust production engine
languages/    Language-native integrations
ecosystems/   Framework, ORM, database, and tool integrations
models/       Model definitions, adapters, manifests, artifact metadata
training/     Python ML workflows
datasets/     Versioned dataset outputs and manifests
ingestion/    Git/GitHub/review/diff/license/provenance ingestion
analyzers/    Deterministic AST, lint, type, security, dependency analysis
evaluation/   Benchmarks and regression measurement
interfaces/   CLI, LSP, API, and future editor clients
runtimes/     Native, server, and future WASM/WebGPU targets
schemas/      Versioned cross-language contracts
docker/       Reproducible development/training/evaluation environments
docs/         Architecture and development documentation
tests/        Automated tests
```

## Using the CLI

```sh
cargo run -p codevanta-cli -- analyze <path>   # deterministic findings only
cargo run -p codevanta-cli -- review <path>    # findings + a local model's explanation
```

`review` runs the same deterministic analyzers as `analyze`, then asks a locally running OpenAI-compatible chat endpoint (Ollama, llama.cpp's `llama-server`, or a released CodeVanta adapter served the same way) to explain and prioritize each file's findings. The local model is optional: if the endpoint can't be reached, the deterministic findings still print and a warning goes to stderr instead of failing the command. Configure the endpoint and model with `--endpoint`/`--model` flags or the `CODEVANTA_LOCAL_MODEL_ENDPOINT`/`CODEVANTA_LOCAL_MODEL` environment variables (defaults: `http://localhost:11434/v1/chat/completions`, Ollama's default port).

## Status

The repository has the production-oriented polyglot foundation in place: a Rust analysis engine and CLI (deterministic AST analyzers plus local-model review), a GitHub pull-request mining pipeline, a Python dataset/training/evaluation pipeline (proven end-to-end against a tiny offline model), model manifests for the first javascript-typescript specializations, shared JSON schemas, and CI across all three stacks. See `docs/model-pipeline.md` for the full pipeline and what still needs a GPU and licensed data versus what runs today.
