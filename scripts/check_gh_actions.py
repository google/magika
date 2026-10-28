#!/usr/bin/env python3
# Copyright 2026 Google LLC
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
# /// script
# dependencies = ["click"]
# ///
"""Verify that all GitHub Actions in `.github/` are pinned to real, consistent upstream tag hashes.

Why this check exists:
- To defend against mutable tag hijacking (where a compromised upstream action moves a tag like
  `@v4` to malicious code), our workflows pin external actions to full 40-character commit SHAs
  accompanied by a human-readable version tag comment:
      uses: owner/repo@<40-char-commit-sha> # vX.Y.Z
- However, GitHub Actions completely ignores the `# vX.Y.Z` YAML comment at runtime and only
  fetches `<40-char-commit-sha>`. Moreover, because GitHub stores all forks of a public repository
  in a shared Git object network, `owner/repo@<sha>` can resolve commits that were pushed to an
  attacker-controlled fork and never merged or tagged in `owner/repo` ("impostor commit" attack).
- A human reviewer looking at a PR diff cannot easily tell whether a 40-character hex hash actually
  belongs to `owner/repo` or matches the `# vX.Y.Z` comment next to it.

What this script checks:
1. Every YAML file under `.github/workflows/` and `.github/actions/` is scanned for `uses:` lines.
2. Local references (`uses: ./...`) must point to an existing file or directory in this repository.
3. Every external action must be pinned to a full 40-character commit SHA and annotated with a
   version tag comment (`uses: owner/repo[/subpath]@<40-char-sha> # vX.Y.Z`). Unpinned tags or
   branches (e.g., `actions/checkout@v4`) and missing tag comments are rejected.
4. For each unique `(owner/repo, tag, sha)`, we shallow-clone `https://github.com/owner/repo.git`
   at `<tag>`, resolve `refs/tags/<tag>^{commit}` in that upstream repository, and verify that the
   official upstream tag points to the exact commit SHA pinned in our workflows.

Usage:
    uv run scripts/check_gh_actions.py
"""

from __future__ import annotations

import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

import click

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
GITHUB_DIR = REPO_ROOT_DIR / ".github"

# Matches lines with `uses: <value>` in YAML files (with or without a leading `- ` list marker).
USES_LINE_RE = re.compile(r"^\s*(?:-\s*)?uses:\s*(.+?)\s*$")

# Matches `owner/repo[/subpath]@<40-char-sha> # <tag>`.
# Note: in GitHub Actions syntax `owner/repo/subpath@ref`, the git repository is always the first
# two path segments (`owner/repo`), while any `/subpath` is a directory inside that repository.
PINNED_ACTION_RE = re.compile(
    r"^(?P<repo>[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)"
    r"(?:/[A-Za-z0-9_./-]+)?"
    r"@(?P<sha>[0-9a-f]{40})"
    r"\s+#\s*(?P<tag>v[0-9A-Za-z_.-]+)$"
)


@dataclass(frozen=True)
class ActionUse:
    """A single pinned external action reference (`uses: owner/repo@<sha> # <tag>`)."""

    file_path: Path
    line_number: int
    repo: str
    sha: str
    tag: str


@click.command()
def main() -> None:
    """Verify all GitHub Actions in `.github/` are pinned to valid upstream release tag SHAs."""
    assert (REPO_ROOT_DIR / ".git").exists(), (
        f"Not a git repository root: {REPO_ROOT_DIR}"
    )

    yaml_files = get_workflow_and_action_files()
    click.echo(
        f"Step 1/2: Scanning {len(yaml_files)} YAML files under .github/workflows/ and "
        f".github/actions/ to verify every 'uses:' statement is pinned to a 40-char commit SHA "
        f"with a '# vX.Y.Z' version tag comment..."
    )

    action_uses, errors = collect_action_uses(yaml_files)
    if errors:
        for err in errors:
            click.secho(f"ERROR: {err}", fg="red", bold=True)
        sys.exit(1)

    # Group uses by (repo, tag, sha) so we only clone each unique upstream (repo, tag) once.
    uses_by_pin: dict[tuple[str, str, str], list[ActionUse]] = {}
    for use in action_uses:
        key = (use.repo, use.tag, use.sha)
        uses_by_pin.setdefault(key, []).append(use)

    click.echo(
        f"  Found {len(action_uses)} pinned external action uses across "
        f"{len(uses_by_pin)} unique (repo, tag, sha) tuples."
    )
    click.echo(
        "Step 2/2: Cloning each upstream repository at its declared tag to verify that the "
        "official tag in the upstream repo resolves to the exact commit SHA pinned in our "
        "workflows..."
    )

    for (repo, tag, sha), uses in sorted(uses_by_pin.items()):
        click.echo(
            f"  - {repo}: checking that upstream tag {tag!r} resolves to {sha} "
            f"(used in {len(uses)} place(s))...",
            nl=False,
        )
        err = verify_action_pin(repo, tag, sha)
        if err is not None:
            click.secho(" FAILED", fg="red", bold=True)
            locations = ", ".join(
                f"{u.file_path.relative_to(REPO_ROOT_DIR)}:{u.line_number}"
                for u in uses
            )
            errors.append(f"{err} (used in {locations})")
        else:
            click.secho(" OK", fg="green")

    if errors:
        click.echo()
        for err in errors:
            click.secho(f"ERROR: {err}", fg="red", bold=True)
        sys.exit(1)

    click.secho(
        "\nSuccess: all GitHub Actions in .github/ are pinned to commit SHAs that match their "
        "official upstream release tags.",
        fg="green",
        bold=True,
    )


def get_workflow_and_action_files() -> list[Path]:
    """Return all YAML files under `.github/workflows/` and `.github/actions/`."""
    files: list[Path] = []
    for sub_dir in (GITHUB_DIR / "workflows", GITHUB_DIR / "actions"):
        if sub_dir.is_dir():
            files.extend(sorted(sub_dir.rglob("*.yml")))
            files.extend(sorted(sub_dir.rglob("*.yaml")))
    return files


def collect_action_uses(yaml_files: list[Path]) -> tuple[list[ActionUse], list[str]]:
    """Parse all `uses:` statements from the given YAML files and validate their syntax.

    Checks that:
    - Local actions/workflows (`uses: ./...`) exist on disk in this repository.
    - External actions follow `uses: owner/repo[/subpath]@<40-char-sha> # vX.Y.Z`.

    Returns a tuple `(action_uses, errors)`, where `action_uses` is the list of
    parsed external `ActionUse` entries and `errors` is a list of error message
    strings (empty if all `uses:` lines are well-formed).
    """
    action_uses: list[ActionUse] = []
    errors: list[str] = []

    for file_path in yaml_files:
        rel_path = file_path.relative_to(REPO_ROOT_DIR)
        for line_number, line in enumerate(
            file_path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            if line.lstrip().startswith("#"):
                continue

            match = USES_LINE_RE.match(line)
            if not match:
                continue

            raw_value = match.group(1)

            # Local action or reusable workflow reference (e.g., `./.github/actions/rust-cache`).
            if raw_value.startswith("./"):
                local_target = raw_value.split("#", maxsplit=1)[0].strip()
                if not (REPO_ROOT_DIR / local_target).exists():
                    errors.append(
                        f"{rel_path}:{line_number}: local reference {local_target!r} does not exist"
                    )
                continue

            pinned_match = PINNED_ACTION_RE.match(raw_value)
            if not pinned_match:
                errors.append(
                    f"{rel_path}:{line_number}: external action must be pinned to a 40-char commit "
                    f"SHA with a version tag comment ('owner/repo@<40-char-sha> # vX.Y.Z'), "
                    f"got: {raw_value!r}"
                )
                continue

            action_uses.append(
                ActionUse(
                    file_path=file_path,
                    line_number=line_number,
                    repo=pinned_match.group("repo"),
                    sha=pinned_match.group("sha"),
                    tag=pinned_match.group("tag"),
                )
            )

    return action_uses, errors


def verify_action_pin(repo: str, tag: str, expected_sha: str) -> str | None:
    """Clone `repo` at `tag` and verify that the upstream tag's commit SHA matches `expected_sha`.

    Unlike raw commit objects (which are shared across all forks of a GitHub repository),
    `refs/tags/<tag>` is controlled by the upstream repository itself. By cloning the upstream
    repository at `<tag>` and resolving `refs/tags/<tag>^{commit}` (which dereferences annotated
    tags to the underlying commit SHA), we confirm that:
    1. `<tag>` is a real git tag in `https://github.com/<repo>.git`.
    2. The commit SHA pinned in our workflow is the exact commit pointed to by `<tag>`.

    Returns `None` if the tag exists in the upstream repository and resolves to
    `expected_sha`, or an error message string otherwise.
    """
    repo_url = f"https://github.com/{repo}.git"

    with tempfile.TemporaryDirectory(prefix="magika-gh-action-") as tmp_dir:
        clone_res = subprocess.run(
            [
                "git",
                "clone",
                "--quiet",
                "-c",
                "advice.detachedHead=false",
                "--depth",
                "1",
                "--branch",
                tag,
                repo_url,
                tmp_dir,
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if clone_res.returncode != 0:
            stderr = clone_res.stderr.strip()
            return f"{repo}@{tag}: failed to clone tag {tag!r} from upstream repo {repo_url}: {stderr}"

        # Explicitly resolve `refs/tags/<tag>^{commit}` so that:
        # - A branch with the same name cannot masquerade as a tag.
        # - Both lightweight tags and annotated tags dereference to the underlying commit SHA.
        rev_res = subprocess.run(
            [
                "git",
                "-C",
                tmp_dir,
                "rev-parse",
                "--verify",
                f"refs/tags/{tag}^{{commit}}",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if rev_res.returncode != 0:
            return (
                f"{repo}@{tag}: {tag!r} is not a valid git tag in upstream repo {repo_url} "
                f"({rev_res.stderr.strip()})"
            )

        actual_sha = rev_res.stdout.strip()
        if actual_sha != expected_sha:
            return (
                f"{repo}@{tag}: tag {tag!r} in upstream repo {repo_url} resolves to commit "
                f"{actual_sha}, which does not match pinned SHA {expected_sha}"
            )

    return None


if __name__ == "__main__":
    main()
