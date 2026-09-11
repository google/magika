# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0
"""Hash-bound HTTPS origins; no credentials, redirects, or unbounded reads."""

import hashlib
import http.client
import threading
from contextlib import contextmanager
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import Request

_connections = threading.local()


@contextmanager
def open_response(request, timeout=30):
    """One reusable TLS connection per downloader thread; never follows redirects."""
    parsed = urlsplit(request.full_url)
    previous = getattr(_connections, "connection", None)
    if previous is None or previous.host != parsed.hostname:
        if previous is not None:
            previous.close()
        previous = http.client.HTTPSConnection(parsed.hostname, timeout=timeout)
        _connections.connection = previous
    response = None
    try:
        path = parsed.path or "/"
        if parsed.query:
            path += "?" + parsed.query
        previous.request("GET", path, headers=dict(request.header_items()))
        response = previous.getresponse()
        if response.status != 200:
            raise HTTPError(
                request.full_url, response.status, response.reason, response.headers, None
            )
        yield response
        if not response.isclosed():
            previous.close()
    except BaseException:
        previous.close()
        _connections.connection = None
        raise
    finally:
        if response is not None:
            response.close()


def validate_url(url):
    parsed = urlsplit(url)
    if (
        parsed.scheme != "https"
        or not parsed.hostname
        or parsed.username
        or parsed.password
        or parsed.fragment
        or parsed.port not in (None, 443)
    ):
        raise ValueError("External origins require an HTTPS URL without credentials or fragments")
    return url


def download(url, limit, expected_size=None, expected_sha256=None):
    validate_url(url)
    request = Request(
        url,
        headers={
            "User-Agent": "magika-datasets/0.1 (evaluation corpus)",
            "Accept-Encoding": "identity",
        },
    )
    with open_response(request, timeout=30) as response:
        if response.headers.get("Content-Encoding", "identity") != "identity":
            raise ValueError("Unexpected HTTP content encoding")
        length = response.headers.get("Content-Length")
        if length is not None and (
            int(length) > limit or (expected_size is not None and int(length) != expected_size)
        ):
            raise ValueError("HTTP size exceeds budget or differs from manifest")
        payload = response.read(limit + 1)
        if len(payload) > limit or (expected_size is not None and len(payload) != expected_size):
            raise ValueError("HTTP body exceeds budget or is incomplete")
        if length is not None and len(payload) != int(length):
            raise ValueError("Truncated HTTP body")
        digest = hashlib.sha256(payload).hexdigest()
        if expected_sha256 is not None and digest != expected_sha256:
            raise ValueError("HTTP SHA-256 mismatch; origin changed")
        return payload, {
            "sha256": digest,
            "etag": response.headers.get("ETag"),
            "last_modified": response.headers.get("Last-Modified"),
        }


class HTTPReader:
    def read_into(self, fixture, output):
        payload, info = download(
            fixture["url"], fixture["size"], fixture["size"], fixture["sha256"]
        )
        output.write(payload)
        return info["sha256"], False

    def close(self):
        pass
