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


"""Performs a number of sanity checks on Magika's documentation."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path

import click
import requests

REPO_ROOT_DIR = Path(__file__).parent.parent.parent
assert REPO_ROOT_DIR.is_dir() and (REPO_ROOT_DIR / ".git").is_dir()


IGNORE_PREFIX_PATTERNS = [
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "python/.venv",
    "python/dist",
    "website/node_modules",
    "website/dist",
    "js/node_modules",
    "js/dist",
    "website-ng/node_modules",
    "website-ng/dist",
    # Checking relative links in the new website is not supported yet.
    "website-ng/src",
]


@click.command()
@click.option("-v", "--verbose", is_flag=True)
def main(verbose: bool) -> None:
    with_errors = False

    success = check_versions_are_up_to_date()
    if not success:
        with_errors = True

    success = check_markdown_links(verbose)
    if not success:
        with_errors = True

    if with_errors:
        print("There was at least one error.")
        sys.exit(1)

    print("Everything looks good.")


def check_versions_are_up_to_date() -> bool:
    """Checks that the mentioned latest versions and models are up to date.
    Returns True if everything is good, False otherwise."""

    # Actual last versions and models
    rust_cli_latest_stable_version = get_max_stable_version_for_crate("magika-cli")
    rust_lib_latest_stable_version = get_max_stable_version_for_crate("magika")
    rust_default_model_name = get_rust_default_model_name()
    python_latest_stable_version = get_python_latest_stable_version()
    python_default_model_name = get_python_default_model_name()
    javascript_latest_stable_version = get_latest_version_for_npm_package("magika")
    javascript_default_model_name = get_javascript_default_model_name()
    demo_model_name = get_demo_model_name()

    expected_table = [
        (rust_cli_latest_stable_version, rust_default_model_name),
        (python_latest_stable_version, python_default_model_name),
        (javascript_latest_stable_version, javascript_default_model_name),
        (rust_lib_latest_stable_version, rust_default_model_name),
        ("-", demo_model_name),
        ("-", "-"),
    ]

    # Extract documented last versions and models
    bindings_overview_path = (
        REPO_ROOT_DIR
        / "website-ng"
        / "src"
        / "content"
        / "docs"
        / "cli-and-bindings"
        / "overview.md"
    )
    assert bindings_overview_path.is_file()
    lines = bindings_overview_path.read_text().splitlines()
    parsed_table = []
    for line in lines:
        # This is a hack to parse the table in the binding's overview, but it is
        # simple and self-contained enough to not cause problems. And we'll
        # notice immediately if things break.
        if line.startswith("| ["):
            cols = line.split("|")
            latest_version = cols[3].strip(" `")
            default_model = cols[4].strip()
            if default_model != "-":
                default_model = default_model.split("]")[0].split("[")[1].strip(" `")
            parsed_table.append((latest_version, default_model))

    if expected_table == parsed_table:
        return True
    else:
        print(
            f"ERROR: Found stale information in binding's overview table:\n{expected_table=}\n{parsed_table=}"
        )
        return False


def get_python_latest_stable_version() -> str:
    res = requests.get("https://pypi.org/pypi/magika/json")
    assert res.status_code == 200
    latest_stable_version = res.json().get("info", {}).get("version", None)
    assert latest_stable_version is not None
    return latest_stable_version


def get_python_default_model_name() -> str:
    magika_path = REPO_ROOT_DIR / "python" / "src" / "magika" / "magika.py"
    return extract_one_match_with_regex_from_file(
        magika_path, '_DEFAULT_MODEL_NAME = "([a-zA-Z0-9_]+)"'
    )


def get_javascript_default_model_name() -> str:
    magika_path = REPO_ROOT_DIR / "js" / "magika.ts"
    return extract_one_match_with_regex_from_file(
        magika_path, 'static MODEL_VERSION = "([a-zA-Z0-9_]+)";'
    )


def get_demo_model_name() -> str:
    """Get the model name used by the demo."""

    demo_path = (
        REPO_ROOT_DIR / "website-ng" / "src" / "components" / "MagikaDemo.svelte"
    )
    return extract_one_match_with_regex_from_file(
        demo_path, 'const MAGIKA_MODEL_VERSION = "([a-zA-Z0-9_]+)";'
    )


def get_rust_default_model_name() -> str:
    model_symlink_path = REPO_ROOT_DIR / "rust" / "gen" / "model"
    assert model_symlink_path.is_symlink()
    return model_symlink_path.readlink().name


def check_markdown_links(verbose: bool) -> bool:
    """Checks that links in Markdown files are OK. Returns True if everything is
    good, False otherwise."""

    with_errors = False
    for path in enumerate_markdown_files_in_dir(Path(".")):
        if verbose:
            print(f"Analyzing {path}")
        for ui in extract_uris_infos_from_file(
            path,
            verbose=verbose,
        ):
            if not ui.is_valid:
                with_errors = True
                print(
                    f"ERROR: {path.relative_to(REPO_ROOT_DIR)} has non-valid uri: {ui.uri}"
                )

            # For python/README.md (which is used on pypi), we also check that
            # the URIs are either pointing to an external resource or are pure
            # anchors.
            if str(path.relative_to(REPO_ROOT_DIR)) == "python/README.md":
                if not ui.is_external and not ui.is_pure_anchor:
                    with_errors = True
                    print(
                        f"ERROR: {path.relative_to(REPO_ROOT_DIR)}, in python/, has a non-external uri: {ui.uri}"
                    )
            # Same for js/README.md, which ends up on npm.
            if str(path.relative_to(REPO_ROOT_DIR)) == "js/README.md":
                if not ui.is_external and not ui.is_pure_anchor:
                    with_errors = True
                    print(
                        f"ERROR: {path.relative_to(REPO_ROOT_DIR)}, in python/, has a non-external uri: {ui.uri}"
                    )

    success = with_errors is False
    return success


def enumerate_markdown_files_in_dir(rel_dir: Path) -> list[Path]:
    if rel_dir.is_absolute():
        print(f"{rel_dir} is not relative")
        sys.exit(1)
    a_dir = REPO_ROOT_DIR / rel_dir
    assert a_dir.is_dir()
    paths: list[Path] = []
    for path in sorted(a_dir.rglob("*.md")):
        should_ignore = False
        for exclude_prefix_pattern in IGNORE_PREFIX_PATTERNS:
            if str(path.relative_to(REPO_ROOT_DIR)).startswith(exclude_prefix_pattern):
                should_ignore = True
                break
        if not should_ignore:
            paths.append(path)
    return paths


def extract_uris_infos_from_file(path: Path, verbose: bool) -> list[UriInfo]:
    uri_regex = r"\[.*?\]\((.*?)\)"
    uris: list[str] = re.findall(uri_regex, path.read_text())

    uris_infos: list[UriInfo] = []
    for uri in uris:
        if verbose:
            print(f"Analyzing uri: {uri}")

        is_external = uri.startswith("http://") or uri.startswith("https://")
        is_valid = None
        is_pure_anchor = None
        is_insecure = None

        if is_external:
            # We treat links pointing to our own repo in a special way. For
            # simplicity, we only deal with links pointing to the main branch.
            repo_main_prefix_url = "https://github.com/google/magika/blob/main/"
            if uri.startswith(repo_main_prefix_url):
                rel_path = uri.removeprefix(repo_main_prefix_url)
                assert (
                    rel_path.find("#") == -1
                ), "Local links with anchors not supported yet"
                abs_path = REPO_ROOT_DIR / rel_path
                is_valid = abs_path.is_file()
            else:
                # We mark any other external link as valid, as actually checking
                # it's too much of a pain.
                is_valid = True
            is_pure_anchor = False
            if uri.startswith("http://"):
                is_insecure = True
                print(f"WARNING: {uri} is not using https")
            else:
                is_insecure = False
        else:
            is_insecure = False
            if uri.startswith("#"):
                is_valid = True
                is_pure_anchor = True
            else:
                is_pure_anchor = False
                if Path(uri).is_absolute():
                    is_valid = False
                else:
                    if uri.find("#") >= 0:
                        # This URI is not a pure anchor, but it does have an
                        # anchor. We remove it so that we can check whether the
                        # file exists or not.
                        rel_file_path = uri.split("#")[0]
                    else:
                        rel_file_path = uri
                    abs_path = path.parent / rel_file_path
                    is_valid = abs_path.is_file() or abs_path.is_dir()

        assert is_valid is not None
        assert is_pure_anchor is not None
        assert is_insecure is not None

        uris_infos.append(
            UriInfo(
                uri=uri,
                is_external=is_external,
                is_valid=is_valid,
                is_pure_anchor=is_pure_anchor,
                is_insecure=is_insecure,
            )
        )

    return uris_infos


def get_max_stable_version_for_crate(crate_name: str) -> str:
    url = f"https://crates.io/api/v1/crates/{crate_name}"
    r = requests.get(url)
    crate_info = r.json()
    return crate_info["crate"]["max_stable_version"]


def get_latest_version_for_npm_package(package_name: str) -> str:
    url = f"https://registry.npmjs.org/{package_name}/latest"
    r = requests.get(url)
    crate_info = r.json()
    return crate_info["version"]


def extract_one_match_with_regex_from_file(path: Path, regex: str) -> str:
    """Extract one (and only one!) match with a regex from a file.

    Raises an error in case of zero or more than one hits.
    """

    assert path.is_file()
    matching_str = None
    for line in path.read_text().splitlines():
        m = re.fullmatch(regex, line.strip())
        if m:
            # If we already found something, there is a bug somewhere
            assert matching_str is None
            matching_str = m.group(1)

    assert matching_str is not None
    return matching_str


@dataclass(kw_only=True)
class UriInfo:
    uri: str
    is_external: bool
    is_valid: bool
    is_pure_anchor: bool
    is_insecure: bool


if __name__ == "__main__":
    main()
