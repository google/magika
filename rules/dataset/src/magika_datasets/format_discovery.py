# Copyright 2026 Google LLC
# SPDX-License-Identifier: Apache-2.0

"""Independent per-format source searches using a shared bounded worker pool."""

import re


def preferred_provider(row):
    """Use GitHub for text/code; VT is the first pass for other whole files."""
    textual = (
        bool(set(row.get("categories", [])) & {"code", "text"})
        or any(m.startswith("text/") for m in row.get("mimes", []))
        or row["format_id"] in {"internetshortcut", "sum", "proteindb", "postscript"}
    )
    return "github" if textual else "virustotal"


def extension_hints(value, class_ids):
    """Validate discovery-only suffixes without changing taxonomy records."""
    if not isinstance(value, dict):
        raise ValueError("github_extensions must be an object keyed by existing format IDs")
    result = {}
    for kind, suffixes in value.items():
        if kind not in class_ids or kind == "invalid":
            raise ValueError(f"github_extensions has an unknown or ineligible format: {kind}")
        if (
            not isinstance(suffixes, list)
            or not suffixes
            or any(
                not isinstance(s, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_+#-]{0,31}", s)
                for s in suffixes
            )
        ):
            raise ValueError(
                f"github_extensions for {kind} must be nonempty bare suffixes, without dots, paths or wildcards"
            )
        result[kind] = sorted({s.lower() for s in suffixes})
    return result
