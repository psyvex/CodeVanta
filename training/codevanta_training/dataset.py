"""Loading, validation, deduplication, and deterministic splitting of
training-example.schema.json records into a versioned dataset manifest.

The examples themselves are never committed to Git (datasets/generated and
datasets/raw are gitignored); only the manifest this module writes is.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import jsonschema

REPO_ROOT = Path(__file__).resolve().parents[2]
TRAINING_EXAMPLE_SCHEMA_PATH = REPO_ROOT / "configs" / "training-example.schema.json"
DATASET_MANIFEST_SCHEMA_PATH = REPO_ROOT / "schemas" / "dataset-manifest.schema.json"

# Deterministic split: an example is assigned to a bucket by hashing its
# content together with the seed, so re-running the split on the same input
# and seed always reproduces the same assignment without persisting it.
DEFAULT_VAL_RATIO = 0.1
DEFAULT_TEST_RATIO = 0.1


class ValidationError(Exception):
    def __init__(self, index: int, message: str) -> None:
        self.index = index
        self.message = message
        super().__init__(f"example {index}: {message}")


def load_training_example_schema() -> dict[str, Any]:
    return json.loads(TRAINING_EXAMPLE_SCHEMA_PATH.read_text())


def load_examples(path: Path) -> list[dict[str, Any]]:
    """Loads training examples from a JSONL file."""
    examples: list[dict[str, Any]] = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        line = line.strip()
        if not line:
            continue
        try:
            examples.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise ValidationError(line_number, f"invalid JSON: {error}") from error
    return examples


def validate_examples(examples: list[dict[str, Any]]) -> list[ValidationError]:
    schema = load_training_example_schema()
    errors: list[ValidationError] = []
    for index, example in enumerate(examples):
        try:
            jsonschema.validate(example, schema)
        except jsonschema.ValidationError as error:
            errors.append(ValidationError(index, error.message))
    return errors


def example_checksum(example: dict[str, Any]) -> str:
    canonical = json.dumps(example, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def deduplicate_examples(examples: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Drops exact-duplicate examples, keeping the first occurrence."""
    seen: set[str] = set()
    unique: list[dict[str, Any]] = []
    for example in examples:
        checksum = example_checksum(example)
        if checksum in seen:
            continue
        seen.add(checksum)
        unique.append(example)
    return unique


@dataclass(frozen=True)
class DatasetSplit:
    train: list[dict[str, Any]]
    val: list[dict[str, Any]]
    test: list[dict[str, Any]]


def split_examples(
    examples: list[dict[str, Any]],
    seed: int,
    val_ratio: float = DEFAULT_VAL_RATIO,
    test_ratio: float = DEFAULT_TEST_RATIO,
) -> DatasetSplit:
    """Deterministically splits examples into train/val/test buckets.

    Each example is assigned by hashing its own content together with the
    seed, so the split is reproducible from (examples, seed) alone and does
    not need to be stored -- re-running it later on the same input reproduces
    the same buckets.
    """
    if not 0 <= val_ratio < 1 or not 0 <= test_ratio < 1 or val_ratio + test_ratio >= 1:
        raise ValueError("val_ratio and test_ratio must be in [0, 1) and sum to less than 1")

    train: list[dict[str, Any]] = []
    val: list[dict[str, Any]] = []
    test: list[dict[str, Any]] = []

    val_threshold = int(val_ratio * 2**32)
    test_threshold = int((val_ratio + test_ratio) * 2**32)

    for example in examples:
        digest = hashlib.sha256(f"{seed}:{example_checksum(example)}".encode()).digest()
        bucket = int.from_bytes(digest[:4], "big")

        if bucket < val_threshold:
            val.append(example)
        elif bucket < test_threshold:
            test.append(example)
        else:
            train.append(example)

    return DatasetSplit(train=train, val=val, test=test)


def split_checksum(examples: list[dict[str, Any]]) -> str:
    """A checksum over a split's contents, order-independent."""
    digests = sorted(example_checksum(example) for example in examples)
    return hashlib.sha256("".join(digests).encode("utf-8")).hexdigest()


def build_dataset_manifest(
    dataset_id: str,
    version: str,
    seed: int,
    split: DatasetSplit,
    source_examples_checksum: str | None = None,
) -> dict[str, Any]:
    return {
        "$schema": "../../schemas/dataset-manifest.schema.json",
        "datasetId": dataset_id,
        "version": version,
        "splitSeed": seed,
        "counts": {
            "train": len(split.train),
            "val": len(split.val),
            "test": len(split.test),
        },
        "checksums": {
            "train": split_checksum(split.train),
            "val": split_checksum(split.val),
            "test": split_checksum(split.test),
        },
        "sourceExamplesChecksum": source_examples_checksum,
    }


def validate_dataset_manifest(manifest: dict[str, Any]) -> None:
    schema = json.loads(DATASET_MANIFEST_SCHEMA_PATH.read_text())
    jsonschema.validate(manifest, schema)


def build_and_write_manifest(
    examples_path: Path,
    manifest_path: Path,
    dataset_id: str,
    version: str,
    seed: int,
) -> dict[str, Any]:
    """End-to-end: load -> validate -> dedup -> split -> write manifest."""
    examples = load_examples(examples_path)

    errors = validate_examples(examples)
    if errors:
        raise ValidationError(errors[0].index, errors[0].message)

    unique_examples = deduplicate_examples(examples)
    split = split_examples(unique_examples, seed=seed)
    source_checksum = split_checksum(unique_examples)

    manifest = build_dataset_manifest(dataset_id, version, seed, split, source_checksum)
    validate_dataset_manifest(manifest)

    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    return manifest
