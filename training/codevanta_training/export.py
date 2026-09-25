"""Merges a trained LoRA adapter into its base model, then converts the
merged model to GGUF for local CPU/GPU inference via llama.cpp/Ollama.

`merge_adapter` is real, provable peft/transformers code with no external
dependency beyond what training already needs; tests/test_export.py proves
it end-to-end against the same tiny offline model used to prove the
training loop.

`convert_to_gguf` shells out to llama.cpp's own `convert_hf_to_gguf.py`
(https://github.com/ggml-org/llama.cpp) rather than reimplementing a GGUF
writer from scratch: GGUF's tensor layout and, especially, its
tokenizer-family detection (a hash-matching check against known real
tokenizers' vocabularies) are maintained upstream and change with new model
architectures. A hand-rolled writer with no way to validate it against a
real llama.cpp runtime in this environment would be a worse foundation than
calling the tool that ships with llama.cpp itself. See
docs/model-pipeline.md for how to point this at a llama.cpp checkout.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path


class ExportError(Exception):
    pass


def merge_adapter(model, output_dir: Path, tokenizer=None):
    """Merges a PeftModel's LoRA weights into its base model and saves the
    result as a standalone (non-adapter) model, ready for GGUF conversion.

    Returns the merged base model (not the PeftModel wrapper).
    """
    merged = model.merge_and_unload()

    output_dir.mkdir(parents=True, exist_ok=True)
    merged.save_pretrained(output_dir)
    if tokenizer is not None:
        tokenizer.save_pretrained(output_dir)

    return merged


def convert_to_gguf(
    model_dir: Path,
    gguf_output: Path,
    convert_script: Path,
    outtype: str = "f16",
) -> Path:
    """Runs llama.cpp's convert_hf_to_gguf.py against a merged model
    directory. `convert_script` is the path to that script in a llama.cpp
    checkout -- this repository does not vendor it.
    """
    if not convert_script.is_file():
        raise ExportError(
            f"convert script not found at {convert_script}. "
            "Clone https://github.com/ggml-org/llama.cpp and pass its "
            "convert_hf_to_gguf.py path."
        )

    gguf_output.parent.mkdir(parents=True, exist_ok=True)

    result = subprocess.run(
        [
            sys.executable,
            str(convert_script),
            str(model_dir),
            "--outfile",
            str(gguf_output),
            "--outtype",
            outtype,
        ],
        capture_output=True,
        text=True,
        check=False,
    )

    if result.returncode != 0:
        stderr_tail = "\n".join(result.stderr.strip().splitlines()[-20:])
        raise ExportError(f"convert_hf_to_gguf.py failed (exit {result.returncode}):\n{stderr_tail}")

    if not gguf_output.is_file():
        raise ExportError(
            f"convert_hf_to_gguf.py exited 0 but {gguf_output} was not created"
        )

    return gguf_output


def main() -> None:
    import argparse

    from .train import load_base_model

    parser = argparse.ArgumentParser(description="Merge a LoRA adapter and export it to GGUF.")
    parser.add_argument("--adapter-dir", required=True, type=Path)
    parser.add_argument("--merged-output-dir", required=True, type=Path)
    parser.add_argument("--gguf-output", type=Path, default=None, help="Skip GGUF conversion if omitted")
    parser.add_argument("--convert-script", type=Path, default=None, help="Path to llama.cpp's convert_hf_to_gguf.py")
    parser.add_argument("--outtype", default="f16", choices=["f32", "f16", "bf16", "q8_0", "auto"])
    args = parser.parse_args()

    import json

    from peft import PeftModel

    adapter_config = json.loads((args.adapter_dir / "adapter_config.json").read_text())
    base_model, tokenizer = load_base_model(adapter_config["base_model_name_or_path"])
    peft_model = PeftModel.from_pretrained(base_model, args.adapter_dir)

    merge_adapter(peft_model, args.merged_output_dir, tokenizer)
    print(f"merged adapter -> {args.merged_output_dir}")

    if args.gguf_output:
        if not args.convert_script:
            raise ExportError("--convert-script is required when --gguf-output is set")
        convert_to_gguf(args.merged_output_dir, args.gguf_output, args.convert_script, args.outtype)
        print(f"exported GGUF -> {args.gguf_output}")


if __name__ == "__main__":
    main()
