# Models

Model definitions, adapters, checkpoints metadata, and release manifests.

Models are organized by language and ecosystem. Large model binaries should not be committed to this repository; store artifacts externally and keep reproducible metadata here.

Planned families include TypeScript/JavaScript, Python, Java, Go, Rust, PHP, C#, and C++.

## Manifests

Each specialization is a manifest at `<language>/<ecosystem>/manifest.json`, validated against `schemas/model-manifest.schema.json`. A manifest records the base model, adapter configuration, dataset/evaluation references, license, and artifact location — never the weights themselves. `status` tracks lifecycle (`planned` -> `in-progress` -> `trained` -> `evaluated` -> `released`); every manifest currently in this repository is `planned`.

`models/registry.json` lists every manifest path. `training/codevanta_training/manifest.py` loads the registry and validates each manifest against the schema; run `pytest` in `training/` to check it.

The first javascript-typescript manifests follow the priority order in `docs/node-ecosystem-scope.md`: a shared `base` adapter target, then the `nestjs` framework adapter, `express` and `fastify`, and the `typeorm`, `prisma`, and `drizzle` ORM adapters.

The python manifests follow the same pattern per `docs/python-ecosystem-scope.md`: a shared `base` adapter target, then the `django` framework adapter, `fastapi` and `flask`, and the `sqlalchemy` ORM adapter. `languages/python` (Rust, tree-sitter-python) already implements five deterministic analyzers this specialization's evidence would build on.
