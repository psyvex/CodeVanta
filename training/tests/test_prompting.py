from codevanta_training.prompting import format_example, parse_generated_finding


def make_example() -> dict:
    return {
        "language": "typescript",
        "framework": "nestjs",
        "code": "const q = `SELECT * FROM users WHERE id = ${id}`;",
        "finding": {
            "category": "security",
            "confidence": "likely",
            "severity": "high",
            "summary": "Interpolated SQL query may allow SQL injection.",
        },
        "provenance": {
            "source_type": "commit",
            "source_url": "https://github.com/example/repo/commit/abc123",
        },
    }


def test_format_example_includes_code_and_ecosystem_hint() -> None:
    prompt, target = format_example(make_example())

    assert "typescript" in prompt
    assert "nestjs" in prompt
    assert "SELECT * FROM users" in prompt
    assert "category: security" in target
    assert "severity: high" in target


def test_format_example_omits_ecosystem_hint_when_absent() -> None:
    example = make_example()
    example["framework"] = None

    prompt, _ = format_example(example)

    assert "using" not in prompt


def test_parse_generated_finding_round_trips_target() -> None:
    _, target = format_example(make_example())

    parsed = parse_generated_finding(target)

    assert parsed == {
        "category": "security",
        "severity": "high",
        "confidence": "likely",
        "summary": "Interpolated SQL query may allow SQL injection.",
    }


def test_parse_generated_finding_ignores_unrelated_lines() -> None:
    parsed = parse_generated_finding("not a field\ncategory: bug\n")
    assert parsed == {"category": "bug"}
