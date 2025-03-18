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
]

# List of (prefix) of urls that return non-200 even if they are valid; for
# simplicity, we just skip checking them.
URLS_ALLOWLIST_PREFIXES = [
    "https://api.securityscorecards.dev/projects/github.com/google/magika/badge",
    "https://crates.io/crates/magika",
    "https://crates.io/crates/magika-cli",
    "https://www.unrealengine.com/en-US",
    "https://www.unrealengine.com/marketplace/en-US/store",
    "https://www.virustotal.com/",
]


@click.command()
@click.option("--skip-external-validity-check", is_flag=True)
@click.option("-v", "--verbose", is_flag=True)
def main(skip_external_validity_check: bool, verbose: bool) -> None:
    with_errors = False

    success = check_versions_are_up_to_date()
    if not success:
        with_errors = True

    success = check_markdown_links(skip_external_validity_check, verbose)
    if not success:
        with_errors = True

    if with_errors:
        print("There was at least one error.")
        sys.exit(1)

    print("Everything looks good.")


def check_versions_are_up_to_date() -> bool:
    """Checks that the mentioned latest versions and models are up to date.
    Returns True if everything is good, False otherwise."""

    # Check that the versions mentioned in the READMEs are up to date
    python_latest_stable_version = get_python_latest_stable_version()
    python_default_model_name = get_python_default_model_name()
    rust_default_model_name = get_rust_default_model_name()

    print(
        f"INFO: {python_latest_stable_version=} {python_default_model_name=} {rust_default_model_name=}"
    )

    expected_lines = [
        f"> - The documentation on GitHub refers to the latest, potentially unreleased and unstable version of Magika. The latest stable release of the `magika` Python package is `{python_latest_stable_version}`, and you can consult the associated documenation [here](https://github.com/google/magika/tree/python-v{python_latest_stable_version}). You can install the latest stable version with: `pip install magika`.",
        f"- Trained and evaluated on a dataset of ~100M files across [200+ content types](./assets/models/{python_default_model_name}/README.md).",
        f"- [List of supported content types by the latest model, `{python_default_model_name}`](./assets/models/{python_default_model_name}/README.md)",
        f"| [Python `Magika` module](./python/README.md) | Stable enough for prod use cases | [`{python_default_model_name}`](./assets/models/{python_default_model_name}/README.md) |",
        f"| [Rust `magika` CLI](https://crates.io/crates/magika-cli) | Stable enough for prod use cases | [`{rust_default_model_name}`](./assets/models/{rust_default_model_name}/README.md) |",
        f"| [Rust `magika` library](https://docs.rs/magika) | Stable enough for prod use cases | [`{rust_default_model_name}`](./assets/models/{rust_default_model_name}/README.md) |",
    ]

    readme_content_lines_set = set(
        (REPO_ROOT_DIR / "README.md").read_text().split("\n")
    )

    with_errors = False
    for expected_line in expected_lines:
        if expected_line not in readme_content_lines_set:
            print(f'ERROR: could not find the following line: "{expected_line}"')
            with_errors = True

    success = with_errors is False
    return success


def get_python_latest_stable_version() -> str:
    res = requests.get("https://pypi.org/pypi/magika/json")
    assert res.status_code == 200
    latest_stable_version = res.json().get("info", {}).get("version", None)
    assert latest_stable_version is not None
    return latest_stable_version


def get_python_default_model_name() -> str:
    default_model_name = None
    magika_path = REPO_ROOT_DIR / "python" / "src" / "magika" / "magika.py"
    assert magika_path.is_file()
    for line in magika_path.read_text().split("\n"):
        m = re.fullmatch('_DEFAULT_MODEL_NAME = "([a-zA-Z0-9_]+)"', line)
        if m:
            # If we already found something, there is a bug somewhere
            assert default_model_name is None
            default_model_name = m.group(1)

    return default_model_name


def get_rust_default_model_name() -> str:
    model_symlink_path = REPO_ROOT_DIR / "rust" / "gen" / "model"
    assert model_symlink_path.is_symlink()
    return model_symlink_path.readlink().name


def check_markdown_links(skip_external_validity_check: bool, verbose: bool) -> bool:
    """Checks that links in Markdown files are OK. Returns True if everything is
    good, False otherwise."""

    with_errors = False
    for path in enumerate_markdown_files_in_dir(Path(".")):
        if verbose:
            print(f"Analyzing {path}")
        for ui in extract_uris_infos_from_file(
            path,
            skip_external_validity_check=skip_external_validity_check,
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


def extract_uris_infos_from_file(
    path: Path, skip_external_validity_check: bool, verbose: bool
) -> list[UriInfo]:
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
            is_pure_anchor = False
            if uri.startswith("http://"):
                is_insecure = True
                print(f"WARNING: {uri} is not using https")
            else:
                is_insecure = False

            if skip_external_validity_check:
                print(f"WARNING: skipping check for {uri}")
                is_valid = True
            else:
                for url_allowlist_prefix in URLS_ALLOWLIST_PREFIXES:
                    if uri.startswith(url_allowlist_prefix):
                        is_valid = True
                        break
                if is_valid is not True:
                    r = requests.head(uri, allow_redirects=True, timeout=5)
                    if r.status_code == 200:
                        is_valid = True
                    else:
                        print(
                            f"ERROR: Got non-200 response status code for {uri}: {r.status_code}"
                        )
                        is_valid = False
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


@dataclass(kw_only=True)
class UriInfo:
    uri: str
    is_external: bool
    is_valid: bool
    is_pure_anchor: bool
    is_insecure: bool


if __name__ == "__main__":
    main()
