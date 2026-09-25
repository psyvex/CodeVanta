from codevanta_training.manifest import load_registry, validate_registry


def test_registry_is_non_empty() -> None:
    assert load_registry()


def test_all_registered_manifests_are_valid() -> None:
    errors = validate_registry()
    assert errors == []
