# Dataset pipeline

This package defines the normalized training-example boundary for CodeVanta.

## Design goals

- Preserve repository and commit provenance.
- Preserve license classification instead of silently accepting unknown sources.
- Separate code changes from review signals.
- Capture Node.js ecosystem metadata so examples can be filtered by framework, ORM, database, test tool, or build tool.
- Produce deterministic example IDs to prevent accidental duplicate ingestion.
- Keep the normalized format independent from any particular training framework.

## Future ingestion stages

1. Discover explicitly selected GitHub repositories.
2. Read repository license and metadata.
3. Select commits, pull requests, review comments, and before/after changes.
4. Detect JavaScript/TypeScript/framework/library/database/tool signals.
5. Normalize review evidence.
6. Apply license/provenance policy.
7. Deduplicate examples.
8. Write JSONL shards for training and evaluation.

Raw GitHub data should remain separate from normalized training data so provenance can be audited.
