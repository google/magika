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
# dependencies = ["click", "pyyaml"]
#
# [tool.uv]
# exclude-newer = "2 weeks"
# required-version = ">=0.13.4"
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
1. Every YAML file under `.github/workflows/` and `.github/actions/` is parsed structurally with
   PyYAML (`SafeLoader`) so multi-line, flow-style, quoted, or duplicate `uses:` keys cannot bypass
   line-based scanning.
2. Local references (`uses: ./...`) must resolve inside `.github/workflows/` (reusable workflow
   file) or `.github/actions/` (composite action directory with `action.yml`/`action.yaml`) so
   nested `uses:` inside local actions are always covered by the scan.
3. Every external action must belong to a vetted GitHub owner in `ALLOWED_OWNERS` and be pinned on
   a single line to a full 40-character commit SHA with a `# vX.Y.Z` SemVer tag comment
   (`uses: owner/repo[/subpath]@<40-char-sha> # vX.Y.Z`). Floating tag comments (such as `# v7`)
   and unpinned refs are rejected.
4. For each unique `(owner/repo, tag, sha)`, we query `git ls-remote` against
   `https://github.com/owner/repo.git` for `refs/tags/<tag>` and `refs/tags/<tag>^{}` (without
   cloning objects, checking out untrusted working trees, or invoking git smudge/credential
   helpers) and verify that the official upstream tag resolves to the exact pinned commit SHA.

Usage:
    uv run scripts/check_gh_actions.py               # Verify all `.github/` action pins online.
    uv run scripts/check_gh_actions.py --self-test   # Run offline validation self-tests.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile
import time
from collections.abc import Iterator
from dataclasses import dataclass
from pathlib import Path

import click
import yaml

REPO_ROOT_DIR = Path(__file__).resolve().parent.parent
GITHUB_DIR = REPO_ROOT_DIR / ".github"

# Vetted GitHub organizations/owners whose actions are permitted in `.github/`.
# Requiring an explicit allowlist ensures this script never queries arbitrary third-party
# repositories and forces explicit review whenever a new action vendor is introduced.
ALLOWED_OWNERS: frozenset[str] = frozenset(
    {
        "PyO3",
        "actions",
        "coverallsapp",
        "docker",
        "github",
        "ossf",
        "pypa",
    }
)

# Matches single-line block-style `uses: <value>` in YAML files (with or without `- `).
USES_LINE_RE = re.compile(r"^\s*(?:-\s*)?uses:\s*(.+?)\s*$")

# Matches `owner/repo[/subpath]@<40-char-sha> # v<major>.<minor>.<patch>[prerelease]`.
# Note: in GitHub Actions syntax `owner/repo/subpath@ref`, the git repository is always the first
# two path segments (`owner/repo`), while any `/subpath` is a directory inside that repository.
PINNED_ACTION_RE = re.compile(
    r"^(?P<repo>(?P<owner>[A-Za-z0-9_.-]+)/[A-Za-z0-9_.-]+)"
    r"(?:/[A-Za-z0-9_./-]+)?"
    r"@(?P<sha>[0-9a-f]{40})"
    r"\s+#\s*(?P<tag>v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z_.-]+)?)$"
)

SHA_HEX_RE = re.compile(r"^[0-9a-f]{40}$")

LS_REMOTE_TIMEOUT_SECONDS = 30
MAX_LS_REMOTE_ATTEMPTS = 3
PASSTHROUGH_ENV_VARS = (
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
    "SSL_CERT_DIR",
    "SSL_CERT_FILE",
    "SYSTEMROOT",
    "WINDIR",
    "http_proxy",
    "https_proxy",
    "no_proxy",
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
@click.option(
    "--self-test",
    is_flag=True,
    help="Run offline unit self-tests for YAML parsing and tag/SHA verification.",
)
def main(self_test: bool) -> None:
    """Verify all GitHub Actions in `.github/` are pinned to valid upstream release tag SHAs."""
    if self_test:
        sys.exit(_run_self_tests())

    if not (REPO_ROOT_DIR / ".git").exists():
        click.secho(
            f"ERROR: Not a git repository root: {REPO_ROOT_DIR}",
            fg="red",
            bold=True,
            err=True,
        )
        sys.exit(1)

    yaml_files = get_workflow_and_action_files(REPO_ROOT_DIR)
    click.echo(
        f"Step 1/2: Scanning {len(yaml_files)} YAML files under .github/workflows/ and "
        f".github/actions/ to verify every 'uses:' statement is pinned to a 40-char commit SHA "
        f"with a '# vX.Y.Z' SemVer tag comment..."
    )

    action_uses, syntax_errors = collect_action_uses(yaml_files, REPO_ROOT_DIR)
    if syntax_errors:
        for err_msg in syntax_errors:
            click.secho(f"ERROR: {err_msg}", fg="red", bold=True, err=True)
        sys.exit(1)

    # Group uses by (repo, tag, sha) so we only query each unique upstream (repo, tag) once.
    uses_by_pin: dict[tuple[str, str, str], list[ActionUse]] = {}
    for use in action_uses:
        key = (use.repo, use.tag, use.sha)
        uses_by_pin.setdefault(key, []).append(use)

    click.echo(
        f"  Found {len(action_uses)} pinned external action uses across "
        f"{len(uses_by_pin)} unique (repo, tag, sha) tuples."
    )
    click.echo(
        "Step 2/2: Querying each upstream repository via 'git ls-remote' to verify that the "
        "official release tag resolves to the exact commit SHA pinned in our workflows..."
    )

    verification_errors: list[str] = []
    for (repo, tag, sha), uses in sorted(uses_by_pin.items()):
        click.echo(
            f"  - {repo}: checking that upstream tag {tag!r} resolves to {sha} "
            f"(used in {len(uses)} place(s))...",
            nl=False,
        )
        pin_error = verify_action_pin(repo, tag, sha)
        if pin_error is not None:
            click.secho(" FAILED", fg="red", bold=True)
            locations = ", ".join(
                f"{u.file_path.relative_to(REPO_ROOT_DIR)}:{u.line_number}"
                for u in uses
            )
            verification_errors.append(f"{pin_error} (used in {locations})")
        else:
            click.secho(" OK", fg="green")

    if verification_errors:
        click.echo(err=True)
        for err_msg in verification_errors:
            click.secho(f"ERROR: {err_msg}", fg="red", bold=True, err=True)
        sys.exit(1)

    click.secho(
        "\nSuccess: all GitHub Actions in .github/ are pinned to commit SHAs that match their "
        "official upstream release tags.",
        fg="green",
        bold=True,
    )


def get_workflow_and_action_files(repo_root: Path = REPO_ROOT_DIR) -> list[Path]:
    """Return all YAML files under `.github/workflows/` and `.github/actions/`."""
    github_dir = repo_root / ".github"
    files: list[Path] = []
    for sub_dir in (github_dir / "workflows", github_dir / "actions"):
        if sub_dir.is_dir():
            files.extend(sorted(sub_dir.rglob("*.yml")))
            files.extend(sorted(sub_dir.rglob("*.yaml")))
    return files


def collect_action_uses(
    yaml_files: list[Path],
    repo_root: Path = REPO_ROOT_DIR,
    allowed_owners: frozenset[str] = ALLOWED_OWNERS,
) -> tuple[list[ActionUse], list[str]]:
    """Parse all `uses:` statements from `yaml_files` and validate their syntax and allowlisting.

    Checks that:
    - Every YAML file parses cleanly with `yaml.SafeLoader`.
    - Every YAML `uses` key is written as an unquoted single-line `uses: <target>` entry.
    - Local references (`uses: ./...`) resolve inside `.github/workflows/` or `.github/actions/`.
    - External references follow `uses: owner/repo[/subpath]@<40-char-sha> # vX.Y.Z` with
      `owner` in `allowed_owners`.
    """
    action_uses: list[ActionUse] = []
    errors: list[str] = []
    workflows_dir = (repo_root / ".github" / "workflows").resolve()
    actions_dir = (repo_root / ".github" / "actions").resolve()

    for file_path in yaml_files:
        rel_path = file_path.relative_to(repo_root)
        content = file_path.read_text(encoding="utf-8")
        lines = content.splitlines()

        try:
            root_node = yaml.compose(content, Loader=yaml.SafeLoader)
        except yaml.YAMLError as exc:
            errors.append(f"{rel_path}: invalid YAML: {exc}")
            continue

        if root_node is None:
            continue

        for key_node, val_node in _iter_yaml_uses_entries(root_node):
            line_idx = key_node.start_mark.line
            line_number = line_idx + 1

            if (
                key_node.style not in (None, "")
                or not isinstance(val_node, yaml.ScalarNode)
                or val_node.style not in (None, "")
                or key_node.start_mark.line != val_node.start_mark.line
                or val_node.start_mark.line != val_node.end_mark.line
                or line_idx >= len(lines)
            ):
                errors.append(
                    f"{rel_path}:{line_number}: 'uses:' must be an unquoted single-line entry "
                    f"('uses: owner/repo@<40-char-sha> # vX.Y.Z' or 'uses: ./...')"
                )
                continue

            line_match = USES_LINE_RE.match(lines[line_idx])
            if not line_match:
                errors.append(
                    f"{rel_path}:{line_number}: 'uses:' must appear on its own YAML block line, "
                    f"got: {lines[line_idx].strip()!r}"
                )
                continue

            raw_value = line_match.group(1)
            uncommented_value = raw_value.split("#", maxsplit=1)[0].strip()
            if uncommented_value != val_node.value:
                errors.append(
                    f"{rel_path}:{line_number}: parsed YAML value {val_node.value!r} does not "
                    f"match raw 'uses:' line {raw_value!r}"
                )
                continue

            # Local composite action or reusable workflow reference.
            if raw_value.startswith("./"):
                local_error = _validate_local_use(
                    uncommented_value, repo_root, workflows_dir, actions_dir
                )
                if local_error is not None:
                    errors.append(f"{rel_path}:{line_number}: {local_error}")
                continue

            pinned_match = PINNED_ACTION_RE.match(raw_value)
            if not pinned_match:
                errors.append(
                    f"{rel_path}:{line_number}: external action must be pinned to a 40-char "
                    f"commit SHA with a SemVer tag comment "
                    f"('owner/repo@<40-char-sha> # vX.Y.Z'), got: {raw_value!r}"
                )
                continue

            owner = pinned_match.group("owner")
            if owner not in allowed_owners:
                errors.append(
                    f"{rel_path}:{line_number}: action owner {owner!r} is not in ALLOWED_OWNERS "
                    f"({sorted(allowed_owners)}); vet the upstream organization before adding it "
                    f"to scripts/check_gh_actions.py"
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


def _iter_yaml_uses_entries(
    node: yaml.Node,
) -> Iterator[tuple[yaml.ScalarNode, yaml.Node]]:
    """Yield every `(key_node, val_node)` mapping entry with key `'uses'` outside `with`/`env`."""
    if isinstance(node, yaml.MappingNode):
        for key_node, val_node in node.value:
            if isinstance(key_node, yaml.ScalarNode) and key_node.value == "uses":
                yield key_node, val_node
            elif isinstance(key_node, yaml.ScalarNode) and key_node.value in (
                "with",
                "env",
            ):
                # Keys under `with:` or `env:` are action inputs or environment variable names,
                # never GitHub Actions `uses:` directives.
                continue
            else:
                yield from _iter_yaml_uses_entries(val_node)
    elif isinstance(node, yaml.SequenceNode):
        for item_node in node.value:
            yield from _iter_yaml_uses_entries(item_node)


def _validate_local_use(
    local_target: str,
    repo_root: Path,
    workflows_dir: Path,
    actions_dir: Path,
) -> str | None:
    """Validate that a `./...` local `uses:` reference exists inside `.github/{workflows,actions}`."""
    resolved = (repo_root / local_target).resolve()
    if not resolved.exists():
        return f"local reference {local_target!r} does not exist"

    if resolved.is_relative_to(workflows_dir):
        if not resolved.is_file() or resolved.suffix not in (".yml", ".yaml"):
            return (
                f"local workflow reference {local_target!r} must point to a .yml/.yaml file "
                f"under .github/workflows/"
            )
        return None

    if resolved.is_relative_to(actions_dir):
        if not resolved.is_dir() or not (
            (resolved / "action.yml").is_file() or (resolved / "action.yaml").is_file()
        ):
            return (
                f"local action reference {local_target!r} must point to a directory under "
                f".github/actions/ containing action.yml or action.yaml"
            )
        return None

    return (
        f"local reference {local_target!r} must reside under .github/workflows/ or "
        f".github/actions/ so its contents are scanned"
    )


def verify_action_pin(
    repo: str,
    tag: str,
    expected_sha: str,
    *,
    repo_url_template: str = "https://github.com/{repo}.git",
    max_attempts: int = MAX_LS_REMOTE_ATTEMPTS,
    timeout_seconds: int = LS_REMOTE_TIMEOUT_SECONDS,
) -> str | None:
    """Query `git ls-remote` on `repo` and verify `refs/tags/<tag>` resolves to `expected_sha`.

    Unlike raw commit objects (which are shared across all forks of a GitHub repository),
    `refs/tags/<tag>` is controlled by the upstream repository itself. Querying
    `refs/tags/<tag>` and `refs/tags/<tag>^{}` via `git ls-remote`:
    1. Avoids `git clone` entirely (zero git objects downloaded, zero working-tree checkout, and
       zero `.lfsconfig` / smudge filter execution).
    2. Prevents branches with the same name (`refs/heads/<tag>`) from masquerading as tags.
    3. Dereferences annotated tags (including nested annotated tags) server-side via `^{}` to
       their underlying commit SHA while also supporting lightweight tags.

    Returns `None` if `tag` exists in the upstream repository and resolves to `expected_sha`,
    or an error message string otherwise.
    """
    repo_url = repo_url_template.format(repo=repo)
    tag_ref = f"refs/tags/{tag}"
    peeled_tag_ref = f"refs/tags/{tag}^{{}}"

    stdout, ls_error = _run_git_ls_remote(
        repo_url=repo_url,
        refs=[tag_ref, peeled_tag_ref],
        max_attempts=max_attempts,
        timeout_seconds=timeout_seconds,
    )
    if ls_error is not None:
        return f"{repo}@{tag}: failed to query {repo_url} via git ls-remote: {ls_error}"

    resolved_refs: dict[str, str] = {}
    for line in stdout.splitlines():
        if not line.strip():
            continue
        parts = line.split("\t", maxsplit=1)
        if len(parts) != 2 or not SHA_HEX_RE.match(parts[0]):
            return (
                f"{repo}@{tag}: unexpected git ls-remote output line from {repo_url}: "
                f"{line!r}"
            )
        resolved_refs[parts[1].strip()] = parts[0]

    # Annotated tags advertise both `refs/tags/<tag>` (the tag object SHA) and
    # `refs/tags/<tag>^{}` (the peeled commit SHA); lightweight tags advertise only
    # `refs/tags/<tag>` (already the commit SHA).
    actual_sha = resolved_refs.get(peeled_tag_ref) or resolved_refs.get(tag_ref)
    if actual_sha is None:
        return f"{repo}@{tag}: tag {tag!r} was not found in upstream repo {repo_url}"

    if actual_sha != expected_sha:
        return (
            f"{repo}@{tag}: tag {tag!r} in upstream repo {repo_url} resolves to commit "
            f"{actual_sha}, which does not match pinned SHA {expected_sha} "
            f"(WARNING: if this pin was previously verified, confirm upstream did not "
            f"force-push or hijack release tag {tag!r} before updating the SHA)"
        )

    return None


def _run_git_ls_remote(
    repo_url: str,
    refs: list[str],
    max_attempts: int,
    timeout_seconds: int,
) -> tuple[str, str | None]:
    """Run an isolated, non-interactive `git ls-remote` call with bounded retries."""
    env = {
        "PATH": os.environ.get("PATH", ""),
        "GIT_TERMINAL_PROMPT": "0",
        "GIT_CONFIG_GLOBAL": os.devnull,
    }
    for var in PASSTHROUGH_ENV_VARS:
        if var in os.environ:
            env[var] = os.environ[var]

    cmd = [
        "git",
        "-c",
        "protocol.version=2",
        "-c",
        "credential.helper=",
        "ls-remote",
        repo_url,
        *refs,
    ]

    last_error = "unknown git ls-remote failure"
    for attempt in range(1, max_attempts + 1):
        try:
            res = subprocess.run(
                cmd,
                env=env,
                stdin=subprocess.DEVNULL,
                capture_output=True,
                text=True,
                timeout=timeout_seconds,
                check=False,
            )
        except subprocess.TimeoutExpired:
            last_error = f"timed out after {timeout_seconds}s"
        else:
            if res.returncode == 0:
                return res.stdout, None
            last_error = (
                res.stderr.strip() or f"git ls-remote exited with code {res.returncode}"
            )

        if attempt < max_attempts:
            time.sleep(attempt)

    return "", last_error


def _run_self_tests() -> int:
    """Run offline self-tests covering YAML parsing, allowlist checks, and `git ls-remote`."""
    failures: list[str] = []
    sha_a = "1" * 40
    sha_b = "2" * 40

    with tempfile.TemporaryDirectory(prefix="magika-check-gh-actions-test-") as tmp:
        fake_root = Path(tmp) / "workspace"
        wf_dir = fake_root / ".github" / "workflows"
        act_dir = fake_root / ".github" / "actions" / "my-cache"
        wf_dir.mkdir(parents=True)
        act_dir.mkdir(parents=True)
        (wf_dir / "reusable.yml").write_text("name: Reusable\non: workflow_call\n")
        (act_dir / "action.yml").write_text(
            f"name: Cache\nruns:\n  using: composite\n  steps:\n"
            f"    - uses: actions/cache@{sha_a} # v6.1.0\n"
        )

        # 1. Valid workflow with pinned actions, local workflow/action refs, and `uses:` inside
        #    `run: |` and `with:`/`env:` (which must not be treated as action references).
        valid_wf = wf_dir / "ci.yml"
        valid_wf.write_text(
            f"name: CI\n"
            f"on: push\n"
            f"jobs:\n"
            f"  call-reusable:\n"
            f"    uses: ./.github/workflows/reusable.yml\n"
            f"  build:\n"
            f"    runs-on: ubuntu-latest\n"
            f"    env:\n"
            f"      uses: ignored-env-var\n"
            f"    steps:\n"
            f"      - uses: actions/checkout@{sha_a} # v7.0.1\n"
            f"        with:\n"
            f"          uses: ignored-input\n"
            f"      - uses: github/codeql-action/init@{sha_b} # v4.39.1-rc.1\n"
            f"      - uses: ./.github/actions/my-cache\n"
            f"      - name: Shell script mentioning uses\n"
            f"        run: |\n"
            f"          echo 'uses: unpinned/action@v1'\n"
        )
        uses, errs = collect_action_uses(
            get_workflow_and_action_files(fake_root), fake_root
        )
        if errs:
            failures.append(f"valid workflow produced unexpected errors: {errs}")
        elif len(uses) != 3:
            failures.append(f"expected 3 external ActionUse entries, got {len(uses)}")

        # 2. Invalid YAML / pinning / allowlist / local-path cases that must all fail closed.
        invalid_cases: list[tuple[str, str, str]] = [
            (
                "unpinned tag",
                "jobs:\n  b:\n    steps:\n      - uses: actions/checkout@v7.0.1\n",
                "must be pinned to a 40-char commit SHA",
            ),
            (
                "missing tag comment",
                f"jobs:\n  b:\n    steps:\n      - uses: actions/checkout@{sha_a}\n",
                "must be pinned to a 40-char commit SHA",
            ),
            (
                "floating major tag comment",
                f"jobs:\n  b:\n    steps:\n      - uses: actions/checkout@{sha_a} # v7\n",
                "must be pinned to a 40-char commit SHA",
            ),
            (
                "legacy pin@ comment",
                f"jobs:\n  b:\n    steps:\n      - uses: actions/checkout@{sha_a} # pin@v7.0.1\n",
                "must be pinned to a 40-char commit SHA",
            ),
            (
                "multi-line uses value",
                f"jobs:\n  b:\n    steps:\n      - uses:\n          actions/checkout@{sha_a} # v7.0.1\n",
                "unquoted single-line entry",
            ),
            (
                "flow-style step mapping",
                f"jobs:\n  b:\n    steps:\n      - {{ uses: actions/checkout@{sha_a} }} # v7.0.1\n",
                "must appear on its own YAML block line",
            ),
            (
                "quoted uses key",
                f'jobs:\n  b:\n    steps:\n      - "uses": actions/checkout@{sha_a} # v7.0.1\n',
                "unquoted single-line entry",
            ),
            (
                "untrusted owner",
                f"jobs:\n  b:\n    steps:\n      - uses: evil-org/action@{sha_a} # v1.0.0\n",
                "not in ALLOWED_OWNERS",
            ),
            (
                "missing local action",
                "jobs:\n  b:\n    steps:\n      - uses: ./.github/actions/missing\n",
                "does not exist",
            ),
            (
                "local action outside .github",
                "jobs:\n  b:\n    steps:\n      - uses: ./outside-action\n",
                "must reside under .github/workflows/ or .github/actions/",
            ),
        ]
        (fake_root / "outside-action").mkdir()
        (fake_root / "outside-action" / "action.yml").write_text("name: Outside\n")

        bad_wf = wf_dir / "bad.yml"
        for case_name, yaml_body, expected_substr in invalid_cases:
            bad_wf.write_text(yaml_body)
            _, case_errs = collect_action_uses([bad_wf], fake_root)
            if not case_errs:
                failures.append(f"{case_name}: expected error, got none")
            elif not any(expected_substr in e for e in case_errs):
                failures.append(
                    f"{case_name}: expected error containing {expected_substr!r}, got {case_errs}"
                )

        # 3. Offline `verify_action_pin` tests against a real local git repository via `file://`.
        #    Note: use a standard `.git` subdirectory so environments with
        #    `safe.bareRepository = explicit` permit local upload-pack.
        remote_worktree = Path(tmp) / "remotes" / "actions" / "checkout"
        remote_worktree.mkdir(parents=True)
        git_base = ["git", "-C", str(remote_worktree)]
        subprocess.run([*git_base, "init", "-q"], check=True)
        subprocess.run(
            [*git_base, "config", "user.email", "test@example.com"], check=True
        )
        subprocess.run([*git_base, "config", "user.name", "Test"], check=True)

        (remote_worktree / "action.yml").write_text("name: Checkout v1\n")
        subprocess.run([*git_base, "add", "action.yml"], check=True)
        subprocess.run([*git_base, "commit", "-q", "-m", "v1"], check=True)
        commit_1 = subprocess.run(
            [*git_base, "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()

        # Lightweight tag `v1.0.0` -> `commit_1`, annotated tag `v1.0.1` -> `commit_1`,
        # nested annotated tag `v1.0.2` -> `v1.0.1` -> `commit_1`, and a branch named `v9.9.9`.
        subprocess.run([*git_base, "tag", "v1.0.0"], check=True)
        subprocess.run(
            [*git_base, "tag", "-a", "v1.0.1", "-m", "annotated"], check=True
        )
        tag_obj_sha = subprocess.run(
            [*git_base, "rev-parse", "refs/tags/v1.0.1"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            [
                *git_base,
                "-c",
                "advice.nestedTag=false",
                "tag",
                "-a",
                "v1.0.2",
                "refs/tags/v1.0.1",
                "-m",
                "nested",
            ],
            check=True,
        )
        subprocess.run([*git_base, "branch", "v9.9.9"], check=True)

        # Second untagged commit ("impostor commit" reachable in object store but not tagged).
        (remote_worktree / "action.yml").write_text("name: Untagged commit\n")
        subprocess.run([*git_base, "add", "action.yml"], check=True)
        subprocess.run([*git_base, "commit", "-q", "-m", "untagged"], check=True)
        untagged_commit = subprocess.run(
            [*git_base, "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            check=True,
        ).stdout.strip()

        url_tpl = f"file://{Path(tmp) / 'remotes'}/{{repo}}/.git"

        for valid_tag in ("v1.0.0", "v1.0.1", "v1.0.2"):
            err = verify_action_pin(
                "actions/checkout",
                valid_tag,
                commit_1,
                repo_url_template=url_tpl,
                max_attempts=1,
            )
            if err is not None:
                failures.append(
                    f"verify_action_pin({valid_tag}) failed unexpectedly: {err}"
                )

        # Pinning the annotated tag object's own hash instead of the commit hash must fail.
        tag_obj_err = verify_action_pin(
            "actions/checkout",
            "v1.0.1",
            tag_obj_sha,
            repo_url_template=url_tpl,
            max_attempts=1,
        )
        if tag_obj_err is None or "WARNING" not in tag_obj_err:
            failures.append(
                f"expected mismatch warning when pinning tag object SHA, got: {tag_obj_err!r}"
            )

        # Pinning an untagged/impostor commit against v1.0.0 must fail.
        impostor_err = verify_action_pin(
            "actions/checkout",
            "v1.0.0",
            untagged_commit,
            repo_url_template=url_tpl,
            max_attempts=1,
        )
        if impostor_err is None or "does not match pinned SHA" not in impostor_err:
            failures.append(
                f"expected mismatch for untagged commit, got: {impostor_err!r}"
            )

        # A branch named `v9.9.9` must not be accepted as a tag.
        branch_err = verify_action_pin(
            "actions/checkout",
            "v9.9.9",
            commit_1,
            repo_url_template=url_tpl,
            max_attempts=1,
        )
        if branch_err is None or "was not found in upstream repo" not in branch_err:
            failures.append(
                f"expected missing-tag error for branch v9.9.9, got: {branch_err!r}"
            )

    total_checks = 1 + len(invalid_cases) + 6
    for failure in failures:
        click.secho(f"FAIL: {failure}", fg="red", bold=True, err=True)
    click.echo(f"{total_checks - len(failures)}/{total_checks} self-tests passed.")
    return 1 if failures else 0


if __name__ == "__main__":
    main()
