#!/usr/bin/env python3

import json
from pathlib import Path
import click

ASSETS_DIR = Path(__file__).parent.parent.parent / "assets"
MODELS_NAMES = ["standard_v2_0", "standard_v2_1"]


@click.command()
def main():
    for model_name in MODELS_NAMES:
        generate_model_readme(model_name)


def generate_model_readme(model_name: str) -> None:
    kb_path = ASSETS_DIR / "content_types_kb.min.json"
    model_dir = ASSETS_DIR / "models" / model_name
    config_path = model_dir / "config.min.json"
    readme_path = model_dir / "README.md"

    kb = json.loads(kb_path.read_text())

    target_labels_space = json.loads(config_path.read_text())["target_labels_space"]

    lines = []
    for idx, target_label in enumerate(target_labels_space):
        line = f'| {idx+1} | {target_label} | {kb[target_label]["description"]} |'
        lines.append(line)

    readme_content_header = f"""
# Content types supported by model "{model_name}"

| Index   |      Content Type Label      | Description |
|----------|:-------------:|------|
"""

    readme_content = readme_content_header.strip() + "\n" + "\n".join(lines) + "\n"

    readme_path.write_text(readme_content)
    print(f"Generated readme for model {model_name} at {readme_path}")


if __name__ == "__main__":
    main()
