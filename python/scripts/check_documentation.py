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

# List of urls that return non-200 even if they are valid; for simplicity, we
# just skip checking them.
URLS_ALLOWLIST = [
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

    if with_errors:
        print("There was at least one error.")
        sys.exit(1)

    print("Everything looks good.")


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
                if uri in URLS_ALLOWLIST:
                    is_valid = True
                else:
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
