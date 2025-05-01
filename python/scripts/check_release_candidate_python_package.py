import re
import subprocess
import sys

import click

import magika


@click.command()
@click.argument("expected_version")
@click.option(
    "--python-client",
    is_flag=True,
    help="Use the Python client instead of CLI for validation.",
)
def main(expected_version, python_client):
    """Checks version consistency for the `magika` package."""
    module_version, instance_version, py_errors = check_python_module_version()
    for error in py_errors:
        click.echo(error, err=True)
        sys.exit(1)

    cli_version = instance_version if python_client else check_rust_cli_version()

    errors = validate_release_tag(module_version, expected_version)
    errors += validate_release_tag(cli_version, expected_version)

    if errors:
        for error in errors:
            click.echo(error, err=True)
        sys.exit(1)

    click.echo("✅ All versions match!")


def validate_release_tag(extracted_version, release_tag):
    """
    Ensures the extracted version matches the expected release tag.

    Args:
        extracted_version (str or None): The version extracted from the package or CLI.
        release_tag (str): The expected release tag version.

    Returns:
        list: A list of error messages if the extracted version does not match the release tag.
    """
    errors = []
    normalized_release_tag = re.sub(r"^python-v", "", release_tag)
    if extracted_version and extracted_version != normalized_release_tag:
        errors.append(
            f"ERROR: Extracted version {extracted_version} does not match release tag {release_tag}."
        )
    return errors


def check_python_module_version():
    """
    Checks the version of the Python `magika` package and its instance version.

    Returns:
        tuple: A tuple containing:
            - module_version (str or None): The `__version__` attribute of the `magika` package.
            - instance_version (str or None): The version retrieved from `magika.Magika().get_version()`.
            - errors (list): A list of error messages if any issues are found.
    """
    errors = []
    module_version = getattr(magika, "__version__", None)

    try:
        instance_version = magika.Magika().get_version()
    except Exception as e:
        click.echo(f"ERROR: Failed to get instance version: {e}", err=True)
        sys.exit(1)

    if module_version is None:
        click.echo("ERROR: Could not retrieve Python package version.", err=True)
        sys.exit(1)

    if "-dev" in module_version:
        errors.append(
            f"ERROR: Python package version {module_version} contains '-dev'."
        )

    return module_version, instance_version, errors


def check_rust_cli_version():
    """
    Checks the version of the Rust CLI `magika` and ensures it is non-dev.

    Returns:
        tuple: A tuple containing:
            - cli_version (str or None): The version string extracted from `magika --version`.
            - errors (list): A list of error messages if any issues are found.
    """
    try:
        result = subprocess.run(
            ["magika", "--version"], capture_output=True, text=True, check=True
        )
        parts = result.stdout.strip().split()
        if len(parts) < 2:
            click.echo("ERROR: Could not parse CLI version output.", err=True)
            sys.exit(1)

        cli_version = parts[1]
        if "-dev" in cli_version:
            click.echo(
                f"ERROR: Rust CLI version {cli_version} contains '-dev'.", err=True
            )
            sys.exit(1)
        return cli_version
    except subprocess.CalledProcessError as e:
        click.echo(f"ERROR: Could not retrieve CLI version: {e}", err=True)
        sys.exit(1)


if __name__ == "__main__":
    main()
