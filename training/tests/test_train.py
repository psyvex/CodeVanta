"""Proves the SFT training loop (train.py's run_training) actually trains,
using the tiny fully-offline model/tokenizer fixtures from conftest.py --
separate from load_base_model, which needs the real base model's hub
repository and is not exercised here.
"""

import pytest

pytest.importorskip("torch")
pytest.importorskip("peft")

from codevanta_training.train import LoraTrainingConfig, build_lora_model, run_training  # noqa: E402


def make_example(summary: str) -> dict:
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


def test_lora_training_loop_reduces_loss(tiny_model, tiny_tokenizer) -> None:
    config = LoraTrainingConfig(
        base_model_id="tiny/offline-test-model",
        rank=4,
        alpha=8,
        target_modules=["c_attn"],
        learning_rate=5e-3,
        epochs=8,
        batch_size=2,
        max_length=128,
        seed=0,
    )

    lora_model = build_lora_model(tiny_model, config)
    examples = [make_example(f"finding number {i}") for i in range(6)]

    result = run_training(lora_model, tiny_tokenizer, examples, config)

    assert result.steps > 0
    # A handful of epochs over a handful of repeated-pattern examples should
    # reliably drive the loss down; this is what proves the optimizer step,
    # label masking, and batching are wired correctly, not a training
    # quality claim about any real dataset.
    assert result.loss_history[-1] < result.loss_history[0]


def test_only_lora_parameters_receive_gradients(tiny_model, tiny_tokenizer) -> None:
    config = LoraTrainingConfig(
        base_model_id="tiny/offline-test-model",
        rank=4,
        target_modules=["c_attn"],
        epochs=1,
        batch_size=2,
    )

    lora_model = build_lora_model(tiny_model, config)
    trainable = [name for name, p in lora_model.named_parameters() if p.requires_grad]

    assert trainable
    assert all("lora" in name for name in trainable)
