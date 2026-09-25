"""Loading and validation for models/ model manifests and the manifest registry."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import jsonschema

REPO_ROOT = Path(__file__).resolve().parents[2]
MODELS_DIR = REPO_ROOT / "models"
SCHEMA_PATH = REPO_ROOT / "schemas" / "model-manifest.schema.json"


@dataclass(frozen=True)
class ManifestError:
    manifest_path: Path
    message: str


def load_schema() -> dict[str, Any]:
    return json.loads(SCHEMA_PATH.read_text())


def load_registry() -> list[Path]:
    registry = json.loads((MODELS_DIR / "registry.json").read_text())
    return [MODELS_DIR / relative for relative in registry["manifests"]]


def load_manifest(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text())


def validate_registry() -> list[ManifestError]:
    """Validates every manifest referenced by the registry against the model-manifest schema.

    Returns one ManifestError per invalid manifest; an empty list means the
    registry and every manifest it references are valid.
    """
    schema = load_schema()
    errors: list[ManifestError] = []

    for manifest_path in load_registry():
        if not manifest_path.is_file():
            errors.append(ManifestError(manifest_path, "referenced by registry.json but missing"))
            continue

        manifest = load_manifest(manifest_path)
        try:
            jsonschema.validate(manifest, schema)
        except jsonschema.ValidationError as error:
            errors.append(ManifestError(manifest_path, error.message))

    return errors
