"""Shared fixtures for tests that need a model: a tiny, fully offline
GPT-2-shaped model and a tiny BPE tokenizer trained from scratch on a small
local corpus. No network access or GPU is used or required.

The `pytest.importorskip` calls live inside the fixtures (not at module
level) so that importing this file never fails collection for tests that
don't need a model -- it only skips the tests that request these fixtures
when torch/transformers/tokenizers aren't installed (the default `python`
CI job; see the `python-train` job for where they are).
"""

import pytest


@pytest.fixture()
def tiny_tokenizer():
    transformers = pytest.importorskip("transformers")
    pytest.importorskip("tokenizers")

    from tokenizers import Tokenizer
    from tokenizers.models import BPE
    from tokenizers.pre_tokenizers import Whitespace
    from tokenizers.trainers import BpeTrainer

    corpus = [
        "const q = SELECT * FROM users WHERE id = id ;",
        "category security severity high confidence likely summary",
        "Interpolated SQL query may allow SQL injection typescript nestjs",
        "### Task Review the following code Identify the single most important finding",
    ]

    raw_tokenizer = Tokenizer(BPE(unk_token="<unk>"))
    raw_tokenizer.pre_tokenizer = Whitespace()
    trainer = BpeTrainer(vocab_size=512, special_tokens=["<pad>", "<unk>", "<eos>"])
    raw_tokenizer.train_from_iterator(corpus, trainer)

    tokenizer = transformers.PreTrainedTokenizerFast(
        tokenizer_object=raw_tokenizer,
        pad_token="<pad>",
        unk_token="<unk>",
        eos_token="<eos>",
    )
    return tokenizer


@pytest.fixture()
def tiny_model(tiny_tokenizer):
    torch = pytest.importorskip("torch")
    from transformers import GPT2Config, GPT2LMHeadModel

    config = GPT2Config(
        vocab_size=tiny_tokenizer.vocab_size + 8,
        n_positions=256,
        n_embd=32,
        n_layer=2,
        n_head=2,
        bos_token_id=tiny_tokenizer.eos_token_id,
        eos_token_id=tiny_tokenizer.eos_token_id,
    )
    torch.manual_seed(0)
    return GPT2LMHeadModel(config)
