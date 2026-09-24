# CodeVanta architecture

CodeVanta is a polyglot monorepo with explicit boundaries:

- **Rust** owns the production analysis engine, repository processing, deterministic orchestration, local runtime interfaces, and the CLI/LSP foundations.
- **Python** owns model research and reproducible ML workflows: dataset preparation, fine-tuning, evaluation, distillation, and quantization.
- **TypeScript** owns JavaScript/TypeScript ecosystem integrations and future VS Code/editor clients where the native ecosystem is an advantage.
- **Language-native tooling** is preferred when a language already provides authoritative parsers, compilers, analyzers, or ecosystem metadata.

## Runtime direction

The Rust core should remain usable as a native library and process. Future targets can expose the same core through CLI, LSP, server, and WASM/WebGPU-compatible layers without coupling the model-training stack to production analysis.

## Model direction

Models are specialists, not the entire analyzer. Deterministic tools provide evidence; models provide contextual reasoning, prioritization, explanations, and patch suggestions. Large model binaries stay outside Git and are referenced by versioned manifests and checksums.

## Compatibility

Cross-language data exchanged between components should use versioned schemas under `schemas/`. Changes to persisted findings or dataset records require deliberate schema-version handling.
