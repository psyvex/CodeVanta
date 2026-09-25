import json
from pathlib import Path

import pytest

from codevanta_training.dataset import (
    ValidationError,
    build_and_write_manifest,
    deduplicate_examples,
    split_examples,
    validate_examples,
)


def make_example(summary: str = "Interpolated SQL query") -> dict:
    return {
        "language": "typescript",
        "framework": "nestjs",
        "code": "const q = `SELECT * FROM users WHERE id = ${id}`;",
        "finding": {
            "category": "security",
            "confidence": "likely",
            "severity": "high",
            "summary": summary,
        },
        "provenance": {
            "source_type": "commit",
            "source_url": "https://github.com/example/repo/commit/abc123",
        },
    }


def test_valid_examples_produce_no_errors() -> None:
    assert validate_examples([make_example()]) == []


def test_invalid_example_is_reported() -> None:
    broken = make_example()
    del broken["finding"]["severity"]

    errors = validate_examples([broken])

    assert len(errors) == 1
    assert errors[0].index == 0


def test_deduplicate_examples_drops_exact_duplicates() -> None:
    examples = [make_example(), make_example(), make_example("A different finding")]

    unique = deduplicate_examples(examples)

    assert len(unique) == 2


def test_split_is_deterministic_and_covers_every_example() -> None:
    examples = [make_example(f"finding {i}") for i in range(200)]

    first = split_examples(examples, seed=42)
    second = split_examples(examples, seed=42)

    assert first.train == second.train
    assert first.val == second.val
    assert first.test == second.test
    assert len(first.train) + len(first.val) + len(first.test) == len(examples)
    assert len(first.val) > 0
    assert len(first.test) > 0


def test_different_seeds_can_change_the_split() -> None:
    examples = [make_example(f"finding {i}") for i in range(200)]

    a = split_examples(examples, seed=1)
    b = split_examples(examples, seed=2)

    assert a.train != b.train or a.val != b.val


def test_build_and_write_manifest_end_to_end(tmp_path: Path) -> None:
    examples_path = tmp_path / "examples.jsonl"
    examples_path.write_text(
        "\n".join(json.dumps(make_example(f"finding {i}")) for i in range(100)) + "\n"
    )
    manifest_path = tmp_path / "manifest.json"

    manifest = build_and_write_manifest(
        examples_path, manifest_path, dataset_id="javascript-typescript/nestjs", version="0.1.0", seed=7
    )

    assert manifest_path.is_file()
    assert manifest["counts"]["train"] + manifest["counts"]["val"] + manifest["counts"]["test"] == 100
    assert json.loads(manifest_path.read_text()) == manifest


def test_build_and_write_manifest_rejects_invalid_examples(tmp_path: Path) -> None:
    broken = make_example()
    del broken["finding"]["severity"]
    examples_path = tmp_path / "examples.jsonl"
    examples_path.write_text(json.dumps(broken) + "\n")

    with pytest.raises(ValidationError):
        build_and_write_manifest(
            examples_path, tmp_path / "manifest.json", dataset_id="x", version="0.1.0", seed=1
        )
