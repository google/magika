# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


"""
It performs a number of checks to determine whether the package is ready for release.
"""

import re
import subprocess
import sys

import click

import magika


@click.command()
@click.argument("expected_version")
@click.option(
    "--use-python-client",
    is_flag=True,
    help="Use the Python client instead of Rust CLI.",
)
def main(expected_version: str, use_python_client: bool) -> None:
    """Checks version consistency for the `magika` package."""

    with_errors = False

    # Get the versions
    module_version = getattr(magika, "__version__", "")
    try:
        instance_version = magika.Magika().get_module_version()
    except Exception:
        instance_version = ""

    package_version = get_magika_package_version_via_pip_show()

    if use_python_client:
        cli_version = instance_version
    else:
        cli_version = get_rust_cli_version()

    if module_version == "":
        click.echo("ERROR: failed to get module_version.")
        with_errors = True
    if instance_version == "":
        click.echo("ERROR: failed to get instance_version.")
        with_errors = True
    if package_version == "":
        click.echo("ERROR: failed to get package_version.")
        with_errors = True
    if cli_version == "":
        click.echo("ERROR: failed to get cli_version.")
        with_errors = True

    click.echo(
        f"Versions: {expected_version=}, {module_version=}, {instance_version=}, {package_version=}, {cli_version=}"
    )

    if module_version != expected_version:
        click.echo(f"ERROR: {module_version=} != {expected_version=}")
        with_errors = True

    if module_version != instance_version:
        click.echo(f"ERROR: {instance_version=} != {module_version=}")
        with_errors = True

    if module_version != package_version:
        click.echo(f"ERROR: {module_version=} != {package_version=}")
        with_errors = True

    # From now on, we assume all the python-related versions are the same. If
    # they are not, we would have at least one error above.

    if not is_valid_python_version(module_version):
        click.echo(f"ERROR: {module_version=} is not a valid python version")
        with_errors = True

    if module_version.endswith("-dev") or cli_version.endswith("-dev"):
        click.echo("ERROR: One of the versions is a -dev version.")
        with_errors = True

    if cli_version.endswith("-rc") and not module_version.endswith("-rc"):
        click.echo("ERROR: The CLI has an -rc version, but the python module does not.")
        with_errors = True

    if with_errors:
        click.echo("There was at least one error")
        sys.exit(1)
    else:
        click.echo("All tests pass!")


def get_rust_cli_version() -> str:
    """Get the version of the Rust CLI `magika`.

    Returns an empty string ("") if an error is encountered.
    """
    try:
        result = subprocess.run(
            ["magika", "--version"], capture_output=True, text=True, check=True
        )
        parts = result.stdout.strip().split()
        if len(parts) < 2:
            click.echo("ERROR: Could not parse CLI version output.")
            return ""
        cli_version = parts[1]
        return cli_version
    except subprocess.CalledProcessError as e:
        click.echo(f"ERROR: Could not retrieve CLI version: {e}")
        return ""


def get_magika_package_version_via_pip_show() -> str:
    try:
        r = subprocess.run(
            ["python3", "-m", "pip", "show", "magika"], capture_output=True, text=True
        )
        lines = r.stdout.strip().split("\n")
        for line in lines:
            if line.startswith("Version: "):
                return line.split(": ", 1)[1]
        click.echo(
            f"ERROR: Could not extract the package version via pip show. Output from pip show: {r.stdout}"
        )
        return ""
    except subprocess.CalledProcessError as e:
        click.echo(f"ERROR: Could not retrieve package version via pip show: {e}")
        return ""


def is_valid_python_version(version: str) -> bool:
    # Regex from PEP440: '[N!]N(.N)*[{a|b|rc}N][.postN][.devN]'
    PEP440_CANONICAL_REGEX = re.compile(
        r"""
^
# Optional Epoch segment (e.g., 1!)
(?P<epoch>\d+!)?

# Required Release segment (e.g., 1.2.3)
(?P<release>[0-9]+(?:\.[0-9]+)*)

# Optional Pre-release segment (e.g., a1, b2, rc3)
(?P<pre>
    (?:a|b|rc)
    [0-9]+
)?

# Optional Post-release segment (e.g., .post4)
(?P<post>
    (?:\.post[0-9]+)
)?

# Optional Development release segment (e.g., .dev5)
(?P<dev>
    (?:\.dev[0-9]+)
)?
$
""",
        re.VERBOSE | re.IGNORECASE,
    )
    return PEP440_CANONICAL_REGEX.fullmatch(version) is not None


def test_is_valid_python_version() -> None:
    assert is_valid_python_version("1.2.3") is True
    assert is_valid_python_version("1.2.3.rc") is False
    assert is_valid_python_version("1.2.3.rc0") is False
    assert is_valid_python_version("1.2.3rc0") is True
    assert is_valid_python_version("1.2.3rc1") is True
    assert is_valid_python_version("1.2.3-dev") is False
    assert is_valid_python_version("1.2.3.dev0") is True
    assert is_valid_python_version("1.2.3-dev0") is False


if __name__ == "__main__":
    main()
