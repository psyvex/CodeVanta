"""LoRA fine-tuning of a model manifest's base model against a dataset
manifest's train split.

Model/tokenizer loading (`load_base_model`) needs network access to the base
model's hub repository and is exercised for real in production. The training
loop itself (`run_training`) takes an already-loaded model and tokenizer, so
it can be -- and is, in tests/test_train.py -- proven correct against a tiny
in-memory model with no network or GPU involved.
"""

from __future__ import annotations

import json
import random
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from .prompting import format_example

REPO_ROOT = Path(__file__).resolve().parents[2]


@dataclass(frozen=True)
class LoraTrainingConfig:
    base_model_id: str
    rank: int = 16
    alpha: int = 32
    dropout: float = 0.05
    target_modules: list[str] = field(default_factory=lambda: ["q_proj", "v_proj"])
    learning_rate: float = 2e-4
    epochs: int = 1
    batch_size: int = 4
    max_length: int = 512
    seed: int = 0


@dataclass(frozen=True)
class TrainingResult:
    loss_history: list[float]
    steps: int

    @property
    def final_loss(self) -> float:
        return self.loss_history[-1] if self.loss_history else float("nan")


def load_base_model(base_model_id: str):
    """Loads the base model and tokenizer from the Hugging Face hub.

    Requires network access to the model's hub repository; not exercised in
    this sandbox's test suite (see module docstring).
    """
    from transformers import AutoModelForCausalLM, AutoTokenizer

    tokenizer = AutoTokenizer.from_pretrained(base_model_id)
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    model = AutoModelForCausalLM.from_pretrained(base_model_id)
    return model, tokenizer


def build_lora_model(model, config: LoraTrainingConfig):
    from peft import LoraConfig, TaskType, get_peft_model

    lora_config = LoraConfig(
        task_type=TaskType.CAUSAL_LM,
        r=config.rank,
        lora_alpha=config.alpha,
        lora_dropout=config.dropout,
        target_modules=config.target_modules,
    )
    return get_peft_model(model, lora_config)


def build_supervised_batch(tokenizer, examples: list[dict[str, Any]], max_length: int):
    """Tokenizes (prompt, target) pairs for causal-LM SFT, masking prompt
    tokens out of the loss (labels = -100) so the model is only trained to
    predict the finding, not to reproduce the code it was shown.
    """
    import torch

    input_ids_batch: list[list[int]] = []
    labels_batch: list[list[int]] = []

    for example in examples:
        prompt, target = format_example(example)
        prompt_ids = tokenizer(prompt, add_special_tokens=False)["input_ids"]
        target_ids = tokenizer(
            target + tokenizer.eos_token, add_special_tokens=False
        )["input_ids"]

        input_ids = (prompt_ids + target_ids)[:max_length]
        labels = ([-100] * len(prompt_ids) + target_ids)[:max_length]

        input_ids_batch.append(input_ids)
        labels_batch.append(labels)

    max_len = max(len(ids) for ids in input_ids_batch)
    pad_id = tokenizer.pad_token_id

    input_ids = torch.full((len(examples), max_len), pad_id, dtype=torch.long)
    attention_mask = torch.zeros((len(examples), max_len), dtype=torch.long)
    labels = torch.full((len(examples), max_len), -100, dtype=torch.long)

    for row, (ids, lbl) in enumerate(zip(input_ids_batch, labels_batch)):
        input_ids[row, : len(ids)] = torch.tensor(ids, dtype=torch.long)
        attention_mask[row, : len(ids)] = 1
        labels[row, : len(lbl)] = torch.tensor(lbl, dtype=torch.long)

    return {"input_ids": input_ids, "attention_mask": attention_mask, "labels": labels}


def run_training(
    model,
    tokenizer,
    examples: list[dict[str, Any]],
    config: LoraTrainingConfig,
) -> TrainingResult:
    """Runs a plain SFT loop over `examples` for `config.epochs` epochs,
    batched by `config.batch_size`, and returns the per-step loss history.
    """
    import torch

    rng = random.Random(config.seed)
    optimizer = torch.optim.AdamW(
        (parameter for parameter in model.parameters() if parameter.requires_grad),
        lr=config.learning_rate,
    )

    model.train()
    loss_history: list[float] = []

    order = list(range(len(examples)))
    for _ in range(config.epochs):
        rng.shuffle(order)
        for start in range(0, len(order), config.batch_size):
            batch_indices = order[start : start + config.batch_size]
            batch = build_supervised_batch(
                tokenizer, [examples[i] for i in batch_indices], config.max_length
            )

            outputs = model(**batch)
            loss = outputs.loss

            optimizer.zero_grad()
            loss.backward()
            optimizer.step()

            loss_history.append(float(loss.detach()))

    return TrainingResult(loss_history=loss_history, steps=len(loss_history))


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="Fine-tune a CodeVanta model manifest's base model.")
    parser.add_argument("--model-manifest", required=True, type=Path)
    parser.add_argument("--train-examples", required=True, type=Path, help="JSONL path")
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--epochs", type=int, default=1)
    args = parser.parse_args()

    manifest = json.loads(args.model_manifest.read_text())
    adapter = manifest.get("adapter") or {}
    config = LoraTrainingConfig(
        base_model_id=manifest["baseModel"]["id"],
        rank=adapter.get("rank") or 16,
        target_modules=adapter.get("targetModules") or ["q_proj", "v_proj"],
        epochs=args.epochs,
    )

    examples = [json.loads(line) for line in args.train_examples.read_text().splitlines() if line.strip()]

    model, tokenizer = load_base_model(config.base_model_id)
    model = build_lora_model(model, config)
    result = run_training(model, tokenizer, examples, config)

    args.output_dir.mkdir(parents=True, exist_ok=True)
    model.save_pretrained(args.output_dir)
    tokenizer.save_pretrained(args.output_dir)

    print(f"trained {result.steps} steps, final loss {result.final_loss:.4f}")


if __name__ == "__main__":
    main()
