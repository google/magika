#!/usr/bin/env python3

import shutil
from pathlib import Path

import click

ASSETS_DIR = Path(__file__).parent.parent.parent / "assets"
PYTHON_ROOT_DIR = Path(__file__).parent.parent

MODELS_NAMES = ['standard_v2_0']


@click.command()
def main() -> None:
    import_content_type_kb()

    for model_name in MODELS_NAMES:
        import_model(model_name)


def import_content_type_kb() -> None:
    kb_path = ASSETS_DIR / "content_types_kb.min.json"
    python_config_dir = PYTHON_ROOT_DIR / "magika" / "config"
    python_kb_path = python_config_dir / kb_path.name
    copy(kb_path, python_kb_path)


def import_model(model_name: str) -> None:
    models_dir = ASSETS_DIR / "models"
    onnx_path = models_dir / model_name / "model.onnx"
    config_path = models_dir / model_name / "config.min.json"

    python_model_dir = PYTHON_ROOT_DIR / "magika" / "models" / model_name
    python_model_dir.mkdir(parents=True, exist_ok=True)

    copy(onnx_path, python_model_dir / onnx_path.name)
    copy(config_path, python_model_dir / config_path.name)


def copy(src_path: Path, dst_path: Path) -> None:
    """Util to copy files and log what is being copied."""
    print(f"Copying {src_path} => {dst_path}")
    dst_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy(src_path, dst_path)


if __name__ == "__main__":
    main()
