"""merge_adapter is proven end-to-end against the tiny offline model shared
with test_train.py. convert_to_gguf's subprocess plumbing is proven with a
mocked subprocess by default (no llama.cpp checkout needed); an optional
live integration test against a real llama.cpp checkout runs only when
LLAMA_CPP_CONVERT_SCRIPT is set -- see docs/model-pipeline.md.
"""

import os
from pathlib import Path
from unittest.mock import patch

import pytest

pytest.importorskip("torch")
pytest.importorskip("peft")

from codevanta_training.export import ExportError, convert_to_gguf, merge_adapter
from codevanta_training.train import LoraTrainingConfig, build_lora_model


def test_merge_adapter_produces_a_standalone_model_with_matching_outputs(
    tiny_model, tiny_tokenizer, tmp_path: Path
) -> None:
    import torch

    config = LoraTrainingConfig(base_model_id="tiny/offline-test-model", rank=4, target_modules=["c_attn"])
    lora_model = build_lora_model(tiny_model, config)

    inputs = tiny_tokenizer("hello world", return_tensors="pt")
    lora_model.eval()
    with torch.no_grad():
        expected_logits = lora_model(**inputs).logits

    output_dir = tmp_path / "merged"
    merged = merge_adapter(lora_model, output_dir, tiny_tokenizer)

    # No LoRA parameters should remain -- this is a standalone model now.
    assert all("lora" not in name for name, _ in merged.named_parameters())
    assert (output_dir / "config.json").is_file()
    assert (output_dir / "tokenizer.json").is_file()

    merged.eval()
    with torch.no_grad():
        merged_logits = merged(**inputs).logits
    assert torch.allclose(expected_logits, merged_logits, atol=1e-5)


def test_convert_to_gguf_builds_the_expected_command(tmp_path: Path) -> None:
    convert_script = tmp_path / "convert_hf_to_gguf.py"
    convert_script.write_text("# stub")
    model_dir = tmp_path / "model"
    gguf_output = tmp_path / "out.gguf"

    def fake_run(command, capture_output, text, check):
        assert command[1:3] == [str(convert_script), str(model_dir)]
        assert "--outtype" in command and "q8_0" in command
        gguf_output.write_bytes(b"fake gguf")

        class Result:
            returncode = 0
            stderr = ""

        return Result()

    with patch("subprocess.run", side_effect=fake_run):
        result = convert_to_gguf(model_dir, gguf_output, convert_script, outtype="q8_0")

    assert result == gguf_output


def test_convert_to_gguf_raises_with_stderr_on_failure(tmp_path: Path) -> None:
    convert_script = tmp_path / "convert_hf_to_gguf.py"
    convert_script.write_text("# stub")

    class Result:
        returncode = 1
        stderr = "line1\nline2\nNotImplementedError: boom"

    with patch("subprocess.run", return_value=Result()), pytest.raises(ExportError, match="boom"):
        convert_to_gguf(tmp_path / "model", tmp_path / "out.gguf", convert_script)


def test_convert_to_gguf_requires_the_script_to_exist(tmp_path: Path) -> None:
    with pytest.raises(ExportError, match="not found"):
        convert_to_gguf(tmp_path / "model", tmp_path / "out.gguf", tmp_path / "missing.py")


@pytest.mark.skipif(
    not os.environ.get("LLAMA_CPP_CONVERT_SCRIPT"),
    reason="set LLAMA_CPP_CONVERT_SCRIPT to a llama.cpp checkout's convert_hf_to_gguf.py to run this",
)
def test_convert_to_gguf_against_a_real_llama_cpp_checkout(tiny_model, tiny_tokenizer, tmp_path: Path) -> None:
    """Live integration test, skipped unless LLAMA_CPP_CONVERT_SCRIPT is set.

    Documents a real, current limitation rather than hiding it: llama.cpp
    only converts tokenizers it can recognize by hashing their vocabulary
    against a table of known real tokenizers (GPT-2, Llama, Qwen, ...), so a
    from-scratch synthetic tokenizer like this fixture's is expected to be
    rejected at the vocabulary stage -- after tensor conversion has already
    succeeded. A real base model's own tokenizer (downloaded from its hub
    repository) is recognized and converts cleanly.
    """
    convert_script = Path(os.environ["LLAMA_CPP_CONVERT_SCRIPT"])
    model_dir = tmp_path / "merged"
    tiny_model.save_pretrained(model_dir)
    tiny_tokenizer.save_pretrained(model_dir)

    # llama.cpp's GPT-2 converter still reads the legacy `n_ctx` config key,
    # which recent transformers versions stopped serializing (only
    # n_positions is written). This is an upstream compatibility gap
    # between the two projects' release cadences, not something this
    # wrapper should paper over -- pinning transformers to the version in
    # llama.cpp's own requirements-convert_hf_to_gguf.txt avoids it in
    # a real export.
    import json

    config_path = model_dir / "config.json"
    config = json.loads(config_path.read_text())
    config["n_ctx"] = config["n_positions"]
    config_path.write_text(json.dumps(config))

    with pytest.raises(ExportError, match="pre-tokenizer|tokenizer"):
        convert_to_gguf(model_dir, tmp_path / "out.gguf", convert_script)
