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
@click.option(
    "--expected-version",
    default="",
    help="Expected version string (e.g., '1.2.3'). If provided, checks will be validated against this value.",
)
@click.option(
    "--report-only",
    is_flag=True,
    help="Print errors without failing. (Default: Fails on errors)",
)
@click.option(
    "--run-magika-via-uv",
    is_flag=True,
    help="When running `magika`, prefix it with `uv run`.",
)
@click.option(
    "--check-pip-show-package-version/--no-check-pip-show-package-version",
    is_flag=True,
    default=True,
    help="Enable/disable version check via 'pip show'. (Default: Enabled)",
)
@click.option(
    "--use-python-client",
    is_flag=True,
    help="Use the Python client instead of Rust client. (Default: False)",
)
def main(
    expected_version: str,
    report_only: bool,
    run_magika_via_uv: bool,
    check_pip_show_package_version: bool,
    use_python_client: bool,
) -> None:
    """Checks versions consistency for the `magika` package."""

    if report_only:
        click.echo('Running in "report only" mode.')
    if run_magika_via_uv:
        click.echo("Running magika client via uv.")
    if not check_pip_show_package_version:
        click.echo("Skipping checking package version via pip show.")
    if use_python_client:
        click.echo("Using python client instead of Rust client.")

    strict_mode = not report_only
    if strict_mode:
        if expected_version == "":
            click.secho(
                "ERROR: when not using --report-only, --expected-version is required."
            )
            sys.exit(1)

    with_errors = False

    # Get the versions
    module_version = getattr(magika, "__version__", "")
    try:
        instance_version = magika.Magika().get_module_version()
    except Exception:
        instance_version = ""

    if check_pip_show_package_version:
        pip_show_package_version = get_magika_package_version_via_pip_show()
    else:
        pip_show_package_version = ""

    if use_python_client:
        cli_version = instance_version
    else:
        cli_version = get_rust_cli_version(run_magika_via_uv=run_magika_via_uv)

    if module_version == "":
        click.echo("ERROR: failed to get module_version.")
        with_errors = True
    if instance_version == "":
        click.echo("ERROR: failed to get instance_version.")
        with_errors = True
    if check_pip_show_package_version and pip_show_package_version == "":
        click.echo("ERROR: failed to get pip_show_package_version.")
        with_errors = True
    if cli_version == "":
        click.echo("ERROR: failed to get cli_version.")
        with_errors = True

    click.echo(
        f"Extracted versions: {expected_version=}, {module_version=}, {instance_version=}, {pip_show_package_version=}, {cli_version=}."
    )

    if expected_version != "" and module_version != expected_version:
        click.echo(f"ERROR: {module_version=} != {expected_version=}")
        with_errors = True

    if module_version != instance_version:
        click.echo(f"ERROR: {instance_version=} != {module_version=}")
        with_errors = True

    if check_pip_show_package_version:
        if module_version != pip_show_package_version:
            click.echo(f"ERROR: {module_version=} != {pip_show_package_version=}")
            with_errors = True

    # From now on, we assume all the python-related versions are the same. If
    # they are not, we would have at least one error above.

    if not is_valid_python_version(module_version):
        click.echo(f"ERROR: {module_version=} is not a valid python version.")
        with_errors = True

    if module_version.endswith("-dev") or cli_version.endswith("-dev"):
        click.echo("ERROR: One of the versions is a -dev version.")
        with_errors = True

    if cli_version.endswith("-rc") and not module_version.endswith("-rc"):
        click.echo("ERROR: The CLI has an -rc version, but the python module does not.")
        with_errors = True

    if with_errors:
        click.secho("There was at least one error.", fg="red")
        if strict_mode:
            sys.exit(1)
    else:
        click.secho("All tests pass!", fg="green")


def get_rust_cli_version(run_magika_via_uv: bool) -> str:
    """Get the version of the Rust CLI `magika`.

    If run_magika_via_uv is True, then attempts to find the magika client via
    `uv run`. Otherwise, assume magika is available in the PATH.

    Returns an empty string ("") if an error is encountered.
    """

    cmd_parts = ["magika", "--version"]
    if run_magika_via_uv:
        cmd_parts = ["uv", "run"] + cmd_parts

    try:
        result = subprocess.run(
            cmd_parts,
            capture_output=True,
            text=True,
            check=True,
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
            f"ERROR: Could not extract the package version via pip show. Output from pip show:\nstdout={r.stdout}\n\nstderr={r.stderr}"
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
