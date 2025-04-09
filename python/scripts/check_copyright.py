#!/usr/bin/env python3

import re
import subprocess
import sys
from pathlib import Path

import click

EXTENSIONS = (".py", ".sh", ".ts")
EXCLUDED_PATH = "tests_data"
DIRECTORIES = ("python/", "rust/", "js/")
COPYRIGHT_PATTERN = re.compile(r"Copyright", re.IGNORECASE)


def get_git_files():
    """
    Retrieve a list of files tracked by git that meet the following conditions:
      - File extension is one of the allowed extensions (.py, .sh, .ts).
      - The file path does not contain the excluded path ('tests_data').
      - The file is located in one of the designated directories (python/, rust/, js/).

    Returns:
        List[Path]: A list of Path objects for files meeting the criteria.

    Raises:
        SystemExit: Exits with code 2 if the git command fails.
    """

    try:
        result = subprocess.run(
            ["git", "ls-files"], capture_output=True, text=True, check=True
        )
        return [
            Path(f)
            for f in result.stdout.strip().splitlines()
            if f.endswith(EXTENSIONS)
            and EXCLUDED_PATH not in f
            and f.startswith(DIRECTORIES)
        ]
    except subprocess.CalledProcessError as e:
        click.secho(f"Git command failed: {e}", fg="red", bold=True)
        sys.exit(2)


def has_copyright(path: Path) -> bool:
    """
    Checks if the file at the given path contains a copyright notice.
    It reads the first five lines and searches for a match to the COPYRIGHT_PATTERN.

    Args:
        path (Path): The file path to check.

    Returns:
        bool: True if the copyright notice is found within the first five lines; False otherwise.
    """

    try:
        with path.open("r", encoding="utf-8") as f:
            for _ in range(5):
                line = f.readline()
                if COPYRIGHT_PATTERN.search(line):
                    return True
        return False
    except Exception:
        return False


@click.group()
def cli():
    """
    A Click command group to define copyright-related commands.

    This function serves as an entry point for the command-line interface.
    """
    pass


@cli.command("check")
def check_command():
    """
    Check for missing copyright headers in the target files.

    The command performs the following steps:
        - Retrieves the list of git-tracked files meeting the criteria.
        - Checks each file for a valid copyright header.
        - Prints any files that are missing the header.
        - Exits with status code 1 if any file is missing the header; otherwise exits successfully.
    """
    error_files = []
    for file_path in get_git_files():
        if not has_copyright(file_path):
            error_files.append(file_path)

    if error_files:
        click.secho("Missing copyright in:", fg="red", bold=True)
        for file in error_files:
            click.echo(f"  - {file}")
        sys.exit(1)
    else:
        click.secho("All files have valid copyright.", fg="green")


def copyright_found(file_path: Path) -> bool:
    """
    A wrapper function to determine if a given file has a valid copyright banner.

    Args:
        file_path (Path): The Path object representing the file to check.

    Returns:
        bool: True if the file contains the copyright header in its first five lines; otherwise False.
    """
    return has_copyright(file_path)


if __name__ == "__main__":
    cli()
