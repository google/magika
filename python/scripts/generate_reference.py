import subprocess
from pathlib import Path

import click

python_root_dir = Path(__file__).parent.parent


@click.command()
def main():
    test_scripts_paths = [
        python_root_dir / "tests" / "test_features_extraction_vs_reference.py",
        python_root_dir / "tests" / "test_inference_vs_reference.py",
    ]

    for test_script_path in test_scripts_paths:
        assert test_script_path.is_file()
        cmd = [
            "uv",
            "run",
            str(test_script_path),
            "generate-tests",
        ]

        print(f'Running CMD: {" ".join(cmd)}')
        subprocess.run(
            cmd,
            cwd=python_root_dir,
            check=True,
        )

    print("Everything went good.")


if __name__ == "__main__":
    main()
