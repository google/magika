# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""VirusTotal discovery and whole-file downloads; no uploads or sample execution."""

import hashlib
import json
import math
import os
import re
from http.client import HTTPException
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode, urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener

from .acquisition import now

API = "https://www.virustotal.com/api/v3/"
# Provider type identifiers can differ from the frozen canonical class IDs.
TYPE_ALIASES = {"cap": "pcap", "email": "eml", "ps": "postscript", "isoimage": "iso"}


def file_markings(attributes):
    """Keep identification evidence, excluding document authors and macro source."""
    result = {"basis": "VirusTotal static metadata; attributed claims, not ground truth"}
    for key in (
        "type_tag",
        "type_description",
        "type_extension",
        "magic",
        "magika",
        "tlsh",
        "ssdeep",
    ):
        if isinstance(attributes.get(key), str):
            result[key] = attributes[key]
    if isinstance(attributes.get("trid"), list):
        result["trid"] = [
            {"file_type": item["file_type"], "probability": item["probability"]}
            for item in attributes["trid"]
            if isinstance(item, dict)
            and isinstance(item.get("file_type"), str)
            and type(item.get("probability")) in {int, float}
            and math.isfinite(item["probability"])
            and 0 <= item["probability"] <= 100
        ]
    if isinstance(attributes.get("tags"), list):
        result["tags"] = sorted({x for x in attributes["tags"] if isinstance(x, str)})
    for key in ("office_info", "openxml_info"):
        info = attributes.get(key)
        if isinstance(info, dict) and isinstance(info.get("ole"), dict):
            count = info["ole"].get("num_macros")
            if type(count) is int and count >= 0:
                result[key] = {"num_macros": count}
    return result


class VTError(ValueError):
    def __init__(self, message, *, stop_acquisition=False, http_status=None, retry_after=None):
        super().__init__(message)
        self.stop_acquisition = stop_acquisition
        self.http_status = http_status
        self.retry_after = retry_after


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


class Client:
    def __init__(self, api_key=None, *, opener=None):
        self.api_key = api_key if api_key is not None else os.environ.get("VT_API_KEY")
        if not self.api_key:
            raise VTError(
                "Set VT_API_KEY to use VirusTotal search/download.", stop_acquisition=True
            )
        # API calls do not follow redirects carrying x-apikey. Downloads use
        # the separate signed URL endpoint and an unauthenticated request.
        self.opener = opener if opener is not None else build_opener(NoRedirect())

    def open(self, request):
        try:
            return self.opener.open(request, timeout=30)
        except HTTPError as exc:
            status = exc.code
            retry_after = exc.headers.get("Retry-After") if exc.headers else None
            exc.close()
            reason = {
                401: "invalid API key",
                403: "account lacks required access",
                429: "quota/rate limit reached; retry later",
            }.get(status, "request failed")
            raise VTError(
                f"VirusTotal HTTP {status}: {reason}",
                stop_acquisition=status in {401, 403, 429},
                http_status=status,
                retry_after=retry_after,
            ) from None
        except (URLError, OSError, HTTPException):
            # Transport errors can include the signed URL; retain no secrets.
            raise VTError("VirusTotal transport failed; rerun to resume.") from None

    def api(self, endpoint, params=None):
        url = API + endpoint + ("?" + urlencode(params) if params else "")
        request = Request(url, headers={"x-apikey": self.api_key, "Accept": "application/json"})
        try:
            with self.open(request) as response:
                data = response.read(8 * 1024**2 + 1)
        except (OSError, HTTPException):
            raise VTError("VirusTotal metadata stream failed; rerun to resume.") from None
        if len(data) > 8 * 1024**2:
            raise VTError("VirusTotal metadata exceeds the 8 MiB response limit")
        try:
            result = json.loads(data)
        except (ValueError, UnicodeError):
            raise VTError("VirusTotal returned invalid JSON") from None
        if not isinstance(result, dict):
            raise VTError("VirusTotal returned invalid metadata")
        return result

    def search(self, query, *, limit=100, cursor=None):
        """Freeze one bounded search page; callers explicitly request more pages."""
        if not query.strip() or type(limit) is not int or not 1 <= limit <= 300:
            raise ValueError("VT search requires a query and a limit between 1 and 300")
        params = {"query": query, "limit": limit, "descriptors_only": "false"}
        if cursor:
            params["cursor"] = cursor
        result = self.api("intelligence/search", params)
        entries = result.get("data")
        if not isinstance(entries, list) or len(entries) > limit:
            raise VTError("VirusTotal search returned an invalid result page")
        fixtures = {}
        for entry in entries:
            if not isinstance(entry, dict) or not isinstance(entry.get("attributes"), dict):
                raise VTError("VirusTotal search returned invalid file metadata")
            digest = entry.get("id", "")
            attributes = entry.get("attributes", {})
            size = attributes.get("size")
            if (
                entry.get("type") != "file"
                or not isinstance(digest, str)
                or not re.fullmatch(r"[a-f0-9]{64}", digest)
                or type(size) is not int
                or size < 0
            ):
                raise VTError("VirusTotal search requires file results with SHA-256 and size")
            if digest in fixtures and fixtures[digest]["size"] != size:
                raise VTError("VirusTotal returned conflicting sizes for one SHA-256")
            fixtures[digest] = {
                "provider": "virustotal",
                "source": "virustotal",
                "revision": hashlib.sha256(query.encode()).hexdigest(),
                "path": digest,
                "sha256": digest,
                "size": size,
                "query": query,
                "cohort": "virustotal_search",
                "source_group": "collection:virustotal",
                "producer": None,
                "label_status": "unreviewed",
                "claim": file_markings(attributes),
            }
        next_cursor = (
            result.get("meta", {}).get("cursor") if result.get("links", {}).get("next") else None
        )
        if result.get("links", {}).get("next") and (
            not isinstance(next_cursor, str) or not next_cursor
        ):
            raise VTError("VirusTotal continuation page has no cursor")
        return {
            "schema_version": 1,
            "provider": "virustotal",
            "query": query,
            "retrieved_at": now(),
            "cursor": cursor,
            "next_cursor": next_cursor,
            "fixtures": list(fixtures.values()),
            "limitations": [
                "One search page, not the complete query result set.",
                "Search results can change; saved SHA-256 identities pin downloaded bytes.",
                "VT metadata is discovery evidence, not an accepted format label.",
            ],
        }

    def read_into(self, fixture, output):
        digest = fixture["sha256"]
        if not re.fullmatch(r"[a-f0-9]{64}", digest):
            raise ValueError("VirusTotal download requires SHA-256")
        url = self.api(f"files/{digest}/download_url").get("data")
        if not isinstance(url, str):
            raise VTError("VirusTotal returned no download URL")
        parsed = urlsplit(url)
        if parsed.scheme != "https" or not parsed.hostname or parsed.username or parsed.password:
            raise VTError("VirusTotal returned an invalid download URL")
        remaining, content_hash = fixture["size"], hashlib.sha256()
        try:
            with self.open(Request(url)) as response:
                while remaining:
                    chunk = response.read(min(1024 * 1024, remaining))
                    if not chunk:
                        raise VTError("VirusTotal download is truncated")
                    output.write(chunk)
                    content_hash.update(chunk)
                    remaining -= len(chunk)
                if response.read(1):
                    raise VTError("VirusTotal download exceeds declared size")
        except (OSError, HTTPException):
            raise VTError("VirusTotal download stream failed; rerun to resume.") from None
        if content_hash.hexdigest() != digest:
            raise VTError("VirusTotal download SHA-256 mismatch")
        return digest, False

    def close(self):
        pass
