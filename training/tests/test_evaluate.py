import pytest

from codevanta_training.evaluate import (
    macro_f1,
    passes_regression_gate,
    run_evaluation,
    score_predictions,
    validate_evaluation_report,
)


def test_run_evaluation_produces_a_schema_valid_report_end_to_end(tiny_model, tiny_tokenizer) -> None:
    examples = [
        {
            "language": "typescript",
            "framework": "nestjs",
            "code": "const q = `SELECT * FROM users WHERE id = ${id}`;",
            "finding": {
                "category": "security",
                "confidence": "likely",
                "severity": "high",
                "summary": "Interpolated SQL query",
            },
            "provenance": {
                "source_type": "commit",
                "source_url": "https://github.com/example/repo/commit/abc123",
            },
        }
    ]

    # An untrained tiny model won't produce a correct finding, but generation,
    # parsing, scoring, and schema validation must all run without error.
    report = run_evaluation(tiny_model, tiny_tokenizer, examples)

    assert report["exampleCount"] == 1
    validate_evaluation_report(report)


def test_score_predictions_counts_exact_matches_as_true_positives() -> None:
    scores = score_predictions(["security", "bug"], ["security", "bug"])

    assert scores["security"].true_positives == 1
    assert scores["security"].false_positives == 0
    assert scores["bug"].true_positives == 1


def test_score_predictions_counts_mismatches_as_fp_and_fn() -> None:
    scores = score_predictions(["bug"], ["security"])

    assert scores["security"].false_negatives == 1
    assert scores["bug"].false_positives == 1
    assert scores["security"].true_positives == 0


def test_score_predictions_none_prediction_only_counts_as_false_negative() -> None:
    scores = score_predictions([None], ["security"])

    assert scores["security"].false_negatives == 1
    assert scores["security"].true_positives == 0
    assert len(scores) == 1


def test_score_predictions_rejects_mismatched_lengths() -> None:
    with pytest.raises(ValueError):
        score_predictions(["security"], ["security", "bug"])


def test_macro_f1_of_perfect_predictions_is_one() -> None:
    scores = score_predictions(["security", "bug"], ["security", "bug"])
    assert macro_f1(scores) == 1.0


def test_macro_f1_of_empty_scores_is_zero() -> None:
    assert macro_f1({}) == 0.0


def test_regression_gate_blocks_a_drop_from_the_previous_release() -> None:
    previous = {"macroF1": 0.8}
    worse = {"macroF1": 0.5}

    assert passes_regression_gate(worse, previous) is False


def test_regression_gate_allows_an_improvement() -> None:
    previous = {"macroF1": 0.5}
    better = {"macroF1": 0.8}

    assert passes_regression_gate(better, previous) is True


def test_regression_gate_passes_first_release_above_floor() -> None:
    assert passes_regression_gate({"macroF1": 0.4}, previous=None, min_macro_f1=0.3) is True
    assert passes_regression_gate({"macroF1": 0.1}, previous=None, min_macro_f1=0.3) is False


def test_validate_evaluation_report_accepts_a_well_formed_report() -> None:
    report = {
        "exampleCount": 2,
        "macroF1": 1.0,
        "categories": {
            "security": {
                "precision": 1.0,
                "recall": 1.0,
                "f1": 1.0,
                "truePositives": 2,
                "falsePositives": 0,
                "falseNegatives": 0,
            }
        },
    }
    validate_evaluation_report(report)


def test_validate_evaluation_report_rejects_a_missing_field() -> None:
    import jsonschema

    with pytest.raises(jsonschema.ValidationError):
        validate_evaluation_report({"exampleCount": 1, "categories": {}})
