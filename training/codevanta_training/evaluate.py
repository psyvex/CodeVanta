"""Evaluates a trained adapter's generated findings against a held-out test
split, scored by review-taxonomy category, with a regression gate against a
previous evaluation report.

`score_predictions` is a pure function over parsed findings and is fully
exercised without any model. `run_evaluation` drives real generation against
a loaded model and is exercised in tests/test_evaluate.py against the same
tiny offline model used to prove training in tests/test_train.py.
"""

from __future__ import annotations

import json
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .prompting import format_prompt, parse_generated_finding

REPO_ROOT = Path(__file__).resolve().parents[2]
EVALUATION_REPORT_SCHEMA_PATH = REPO_ROOT / "schemas" / "evaluation-report.schema.json"


@dataclass(frozen=True)
class CategoryScore:
    category: str
    true_positives: int
    false_positives: int
    false_negatives: int

    @property
    def precision(self) -> float:
        denominator = self.true_positives + self.false_positives
        return self.true_positives / denominator if denominator else 0.0

    @property
    def recall(self) -> float:
        denominator = self.true_positives + self.false_negatives
        return self.true_positives / denominator if denominator else 0.0

    @property
    def f1(self) -> float:
        p, r = self.precision, self.recall
        return 2 * p * r / (p + r) if (p + r) else 0.0


def score_predictions(
    predicted_categories: list[str | None], expected_categories: list[str]
) -> dict[str, CategoryScore]:
    """Scores category-classification agreement per category.

    A prediction counts as a true positive for its own category when it
    exactly matches the expected category for that example; a mismatch is a
    false positive for the predicted category and a false negative for the
    expected one. A missing/unparseable prediction (None) only produces a
    false negative -- it is never credited as a match.
    """
    if len(predicted_categories) != len(expected_categories):
        raise ValueError("predicted_categories and expected_categories must be the same length")

    tp: dict[str, int] = defaultdict(int)
    fp: dict[str, int] = defaultdict(int)
    fn: dict[str, int] = defaultdict(int)

    for predicted, expected in zip(predicted_categories, expected_categories):
        if predicted == expected:
            tp[expected] += 1
        else:
            fn[expected] += 1
            if predicted is not None:
                fp[predicted] += 1

    categories = set(tp) | set(fp) | set(fn)
    return {
        category: CategoryScore(
            category=category,
            true_positives=tp[category],
            false_positives=fp[category],
            false_negatives=fn[category],
        )
        for category in categories
    }


def macro_f1(scores: dict[str, CategoryScore]) -> float:
    if not scores:
        return 0.0
    return sum(score.f1 for score in scores.values()) / len(scores)


def generate_finding(model, tokenizer, example: dict[str, Any], max_new_tokens: int = 64) -> str:
    import torch

    prompt = format_prompt(example)
    inputs = tokenizer(prompt, return_tensors="pt")

    model.eval()
    with torch.no_grad():
        output_ids = model.generate(
            **inputs,
            max_new_tokens=max_new_tokens,
            do_sample=False,
            pad_token_id=tokenizer.pad_token_id,
        )

    generated_ids = output_ids[0, inputs["input_ids"].shape[1] :]
    return tokenizer.decode(generated_ids, skip_special_tokens=True)


def validate_evaluation_report(report: dict[str, Any]) -> None:
    import jsonschema

    schema = json.loads(EVALUATION_REPORT_SCHEMA_PATH.read_text())
    jsonschema.validate(report, schema)


def run_evaluation(model, tokenizer, examples: list[dict[str, Any]]) -> dict[str, Any]:
    predicted_categories: list[str | None] = []
    expected_categories: list[str] = []

    for example in examples:
        generated = generate_finding(model, tokenizer, example)
        parsed = parse_generated_finding(generated)
        predicted_categories.append(parsed.get("category"))
        expected_categories.append(example["finding"]["category"])

    scores = score_predictions(predicted_categories, expected_categories)
    report = {
        "exampleCount": len(examples),
        "macroF1": macro_f1(scores),
        "categories": {
            category: {
                "precision": score.precision,
                "recall": score.recall,
                "f1": score.f1,
                "truePositives": score.true_positives,
                "falsePositives": score.false_positives,
                "falseNegatives": score.false_negatives,
            }
            for category, score in scores.items()
        },
    }
    validate_evaluation_report(report)
    return report


def passes_regression_gate(
    current: dict[str, Any], previous: dict[str, Any] | None, min_macro_f1: float = 0.0
) -> bool:
    """A candidate adapter is release-worthy only if it does not regress
    macro F1 relative to the previously released adapter (when one exists)
    and clears an absolute floor.
    """
    if current["macroF1"] < min_macro_f1:
        return False
    if previous is not None and current["macroF1"] < previous["macroF1"]:
        return False
    return True


def main() -> None:
    import argparse

    from .train import load_base_model

    parser = argparse.ArgumentParser(description="Evaluate a CodeVanta adapter against a test split.")
    parser.add_argument("--adapter-dir", required=True, type=Path)
    parser.add_argument("--test-examples", required=True, type=Path, help="JSONL path")
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--previous-report", type=Path, default=None)
    args = parser.parse_args()

    from peft import PeftModel

    from transformers import AutoTokenizer

    adapter_config = json.loads((args.adapter_dir / "adapter_config.json").read_text())
    base_model, tokenizer = load_base_model(adapter_config["base_model_name_or_path"])
    model = PeftModel.from_pretrained(base_model, args.adapter_dir)
    tokenizer = AutoTokenizer.from_pretrained(args.adapter_dir)

    examples = [json.loads(line) for line in args.test_examples.read_text().splitlines() if line.strip()]
    report = run_evaluation(model, tokenizer, examples)

    previous = json.loads(args.previous_report.read_text()) if args.previous_report else None
    report["passesRegressionGate"] = passes_regression_gate(report, previous)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(f"macro F1 {report['macroF1']:.4f}, gate passed: {report['passesRegressionGate']}")


if __name__ == "__main__":
    main()
