# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Read public repository metadata from GitHub. Nothing here downloads sample bytes."""

import hashlib
import json
import os
import tempfile
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import HTTPRedirectHandler, Request, build_opener

API = "https://api.github.com/"
MAX_BYTES = 8 * 1024 * 1024
NOT_A_LICENSE = frozenset({"NOASSERTION", "", None})


class GitHubStatus(Exception):
    """An HTTP status, and nothing else.

    A response body or a request header could carry the token or arbitrary remote text
    into a log, so only the status travels.
    """


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *_):
        return None


class MetadataClient:
    """Cached, token-scoped metadata reads.

    The cache makes a second run offline and its result byte-identical, which is what lets
    a network-assisted licence resolution still be reproducible.
    """

    def __init__(self, cache=Path("local/github-license-cache"), token=None, offline=False):
        self.cache = Path(cache)
        self.cache.mkdir(parents=True, exist_ok=True)
        self.offline = offline
        if offline:
            # Reading answers already on disk needs no credential and makes the default
            # resolution reproduce the full table instead of regressing to what the
            # frozen inventory alone covers.
            self.token, self.failures = None, {}
            return
        self.token = token if token is not None else os.environ.get("GITHUB_TOKEN")
        if not self.token:
            raise ValueError(
                "GITHUB_TOKEN is required to read repository metadata; a fine-grained token "
                "with public repository read and no other permission is enough"
            )
        self.failures: dict[str, int] = {}

    def _fetch(self, endpoint):
        request = Request(
            API + endpoint,
            headers={
                "Authorization": "Bearer " + self.token,
                "Accept": "application/vnd.github+json",
                "User-Agent": "magika-dataset-builder",
            },
        )
        try:
            with build_opener(NoRedirect()).open(request, timeout=30) as response:
                raw = response.read(MAX_BYTES + 1)
        except HTTPError as error:
            raise GitHubStatus(error.code) from None
        if len(raw) > MAX_BYTES:
            raise ValueError("GitHub metadata response exceeds 8 MiB")
        return json.loads(raw)

    def get(self, endpoint):
        path = self.cache / (hashlib.sha256(endpoint.encode()).hexdigest() + ".json")
        if path.exists():
            return json.loads(path.read_text())
        if self.offline:
            raise GitHubStatus(0)
        value = self._fetch(endpoint)
        with tempfile.NamedTemporaryFile(mode="w", dir=self.cache, delete=False) as handle:
            json.dump(value, handle, sort_keys=True)
            temporary = Path(handle.name)
        temporary.replace(path)
        return value

    def license(self, repository):
        """The repository's declared licence, or None when GitHub has no answer.

        A deleted, renamed or private repository is recorded and skipped: one of them must
        not abort a resolution pass over thousands of others.
        """
        try:
            document = self.get(f"repos/{repository}/license")
        except GitHubStatus as error:
            self.failures[repository] = error.args[0]
            return None
        declared = document.get("license") or {}
        if declared.get("spdx_id") in NOT_A_LICENSE:
            return None
        return {
            "spdx_id": declared["spdx_id"],
            "name": declared.get("name"),
            "permalink": document.get("html_url"),
        }
